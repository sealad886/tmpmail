use clap::Parser;
use rand::distr::{Alphanumeric, SampleString};
use serde_json::json;
use std::process::ExitCode;
use tmpmail::cli::{Action, Cli};
use tmpmail::config::{Config, ConfigStore};
use tmpmail::core::ProviderId;
use tmpmail::error::AppError;
use tmpmail::paths::AppPaths;
use tmpmail::providers::{MailTm, MailTmCredentials, Message, MessageSummary};
use tmpmail::render::{Envelope, Meta};
use tmpmail::secrets::{KeyringSecretStore, SecretStore};
use tmpmail::state::{AppState, StateStore};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.validate().and_then(|_| run(&cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if cli.json {
                let envelope = Envelope::<serde_json::Value>::failure(&error, Meta::default());
                println!(
                    "{}",
                    serde_json::to_string(&envelope).expect("JSON envelope")
                );
            } else {
                eprintln!("tmpmail: {error}");
            }
            ExitCode::from(error.exit_code())
        }
    }
}

fn run(cli: &Cli) -> Result<(), AppError> {
    let paths = AppPaths::discover()?;
    match cli.action() {
        Action::InboxList => output_inboxes(cli, &StateStore::at(&paths.state).load()?),
        Action::InboxUse { name } => {
            let state_store = StateStore::at(&paths.state);
            let mut state = state_store.load()?;
            state.set_active(&name)?;
            state_store.save(&state)?;
            output(cli, json!({"active_inbox": name}), format!("Using {name}"))
        }
        Action::ConfigShow => {
            let config = ConfigStore::at(&paths.config).load()?;
            let data = serde_json::to_value(&config)
                .map_err(|error| AppError::state("config_encode", error.to_string()))?;
            let human = toml::to_string_pretty(&config)
                .map_err(|error| AppError::state("config_encode", error.to_string()))?;
            output(cli, data, human.trim_end().to_owned())
        }
        Action::ProviderList => output_providers(cli, &ConfigStore::at(&paths.config).load()?),
        Action::ProviderInfo { provider } => output_provider_info(cli, provider),
        Action::ProviderDomains { provider } => {
            run_async(cli, &paths, NetworkAction::Domains(provider))
        }
        Action::InboxCreate {
            name,
            address,
            provider,
        } => run_async(
            cli,
            &paths,
            NetworkAction::Create {
                name,
                address,
                provider,
            },
        ),
        Action::InboxRemove { name } => run_async(cli, &paths, NetworkAction::Remove(name)),
        Action::MessageList { inbox } => run_async(cli, &paths, NetworkAction::List(inbox)),
        Action::MessageLatest { inbox, peek } => {
            run_async(cli, &paths, NetworkAction::Latest { inbox, peek })
        }
        Action::MessageRead {
            message,
            inbox,
            peek,
        } => run_async(
            cli,
            &paths,
            NetworkAction::Read {
                message,
                inbox,
                peek,
            },
        ),
        Action::MessageNew {
            inbox,
            all_inboxes: false,
            review: false,
        } => run_async(cli, &paths, NetworkAction::New(inbox)),
        Action::MessageAck { messages, inbox } => {
            run_async(cli, &paths, NetworkAction::Ack { messages, inbox })
        }
        Action::AttachmentList { message, inbox } => {
            run_async(cli, &paths, NetworkAction::Attachments { message, inbox })
        }
        Action::Doctor => {
            ConfigStore::at(&paths.config).load()?;
            let state = StateStore::at(&paths.state).load()?;
            output(
                cli,
                json!({"config": "ok", "state": "ok", "inboxes": state.inboxes.len()}),
                format!("config ok\nstate ok\n{} inbox(es)", state.inboxes.len()),
            )
        }
        action => Err(AppError::new(
            "not_implemented",
            format!("{} requires a provider milestone", action_name(&action)),
            5,
        )),
    }
}

enum NetworkAction {
    Domains(Option<ProviderId>),
    Create {
        name: Option<String>,
        address: Option<String>,
        provider: Option<ProviderId>,
    },
    Remove(String),
    List(Option<String>),
    Latest {
        inbox: Option<String>,
        peek: bool,
    },
    Read {
        message: String,
        inbox: Option<String>,
        peek: bool,
    },
    New(Option<String>),
    Ack {
        messages: Vec<String>,
        inbox: Option<String>,
    },
    Attachments {
        message: String,
        inbox: Option<String>,
    },
}

fn run_async(cli: &Cli, paths: &AppPaths, action: NetworkAction) -> Result<(), AppError> {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|error| AppError::new("runtime", error.to_string(), 3))?
        .block_on(run_network(cli, paths, action))
}

async fn run_network(cli: &Cli, paths: &AppPaths, action: NetworkAction) -> Result<(), AppError> {
    let provider =
        MailTm::new(parse_duration(&cli.timeout)?).map_err(|error| error.into_app_error())?;
    match action {
        NetworkAction::Domains(selected) => {
            if selected.is_some_and(|value| value != ProviderId::MailTm) {
                return Err(AppError::new(
                    "provider_capability",
                    "DropMail does not provide domain discovery",
                    2,
                ));
            }
            let domains = provider
                .domains()
                .await
                .map_err(|error| error.into_app_error())?;
            output(cli, json!({"domains": domains}), domains.join("\n"))
        }
        NetworkAction::Create {
            name,
            address,
            provider: selected,
        } => {
            if selected.is_some_and(|value| value != ProviderId::MailTm) {
                return Err(AppError::new(
                    "provider_auth",
                    "DropMail token is not configured",
                    4,
                ));
            }
            let name = name.unwrap_or_else(|| "default".to_owned());
            validate_inbox_name(&name)?;
            let mut state = StateStore::at(&paths.state).load()?;
            if state.inboxes.contains_key(&name) {
                return Err(AppError::usage(format!("inbox already exists: {name}")));
            }
            let credentials = match address {
                Some(address) if address.contains('@') => {
                    let password = random_password();
                    provider.create(&address, &password).await
                }
                Some(local) => provider.create_random(Some(&local)).await,
                None => provider.create_random(None).await,
            }
            .map_err(|error| error.into_app_error())?;
            let secret_ref = format!("mailtm:{name}");
            let secret_bytes = serde_json::to_vec(&credentials)
                .map_err(|error| AppError::state("credential_encode", error.to_string()))?;
            let secrets = KeyringSecretStore;
            secrets.put(&secret_ref, &secret_bytes)?;
            state.insert(tmpmail::core::Inbox::new(
                &name,
                &credentials.address,
                ProviderId::MailTm,
                &secret_ref,
                tmpmail::core::Capabilities::mailtm(),
            ));
            if state.active_inbox.is_none() {
                state.set_active(&name)?;
            }
            if let Err(error) = StateStore::at(&paths.state).save(&state) {
                let _ = provider
                    .delete_account(&credentials.token, &credentials.account_id)
                    .await;
                let _ = secrets.remove(&secret_ref);
                return Err(error);
            }
            output(
                cli,
                json!({"name": name, "address": credentials.address, "provider": "mailtm"}),
                format!("{}  ({name}, Mail.tm)", credentials.address),
            )
        }
        NetworkAction::Remove(name) => {
            let store = StateStore::at(&paths.state);
            let mut state = store.load()?;
            let inbox = state.inboxes.get(&name).cloned().ok_or_else(|| {
                AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
            })?;
            let credentials = load_credentials(&inbox.secret_ref)?;
            provider
                .delete_account(&credentials.token, &credentials.account_id)
                .await
                .map_err(|error| error.into_app_error())?;
            state.remove(&name)?;
            store.save(&state)?;
            KeyringSecretStore.remove(&inbox.secret_ref)?;
            output(cli, json!({"removed": name}), format!("Removed {name}"))
        }
        NetworkAction::List(inbox) => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let messages = provider
                .messages_with_refresh(&mut credentials)
                .await
                .map_err(|error| error.into_app_error())?;
            persist_credentials(&selected.secret_ref, &credentials)?;
            output_messages(cli, messages)
        }
        NetworkAction::Latest { inbox, peek } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let messages = provider
                .messages_with_refresh(&mut credentials)
                .await
                .map_err(|error| error.into_app_error())?;
            let newest = messages
                .into_iter()
                .max_by(|a, b| a.created_at.cmp(&b.created_at))
                .ok_or_else(|| AppError::new("message_not_found", "inbox is empty", 8))?;
            let message = provider
                .message_with_refresh(&mut credentials, &newest.id)
                .await
                .map_err(|error| error.into_app_error())?;
            output_message(cli, &message)?;
            if !peek {
                provider
                    .acknowledge_with_refresh(&mut credentials, &message.id)
                    .await
                    .map_err(|error| error.into_app_error())?;
            }
            persist_credentials(&selected.secret_ref, &credentials)
        }
        NetworkAction::Read {
            message,
            inbox,
            peek,
        } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let message = provider
                .message_with_refresh(&mut credentials, &message)
                .await
                .map_err(|error| error.into_app_error())?;
            output_message(cli, &message)?;
            if !peek {
                provider
                    .acknowledge_with_refresh(&mut credentials, &message.id)
                    .await
                    .map_err(|error| error.into_app_error())?;
            }
            persist_credentials(&selected.secret_ref, &credentials)
        }
        NetworkAction::New(inbox) => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let messages = provider
                .messages_with_refresh(&mut credentials)
                .await
                .map_err(|error| error.into_app_error())?
                .into_iter()
                .filter(|message| !message.seen)
                .collect();
            persist_credentials(&selected.secret_ref, &credentials)?;
            output_messages(cli, messages)
        }
        NetworkAction::Ack { messages, inbox } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            for id in &messages {
                provider
                    .acknowledge_with_refresh(&mut credentials, id)
                    .await
                    .map_err(|error| error.into_app_error())?;
            }
            persist_credentials(&selected.secret_ref, &credentials)?;
            output(
                cli,
                json!({"acknowledged": messages}),
                format!("Acknowledged {} message(s)", messages.len()),
            )
        }
        NetworkAction::Attachments { message, inbox } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let message = provider
                .message_with_refresh(&mut credentials, &message)
                .await
                .map_err(|error| error.into_app_error())?;
            persist_credentials(&selected.secret_ref, &credentials)?;
            let human = message
                .attachments
                .iter()
                .map(|item| format!("{}\t{}\t{} bytes", item.id, item.filename, item.size))
                .collect::<Vec<_>>()
                .join("\n");
            output(cli, json!({"attachments": message.attachments}), human)
        }
    }
}

fn selected_credentials(
    paths: &AppPaths,
    selected_name: Option<&str>,
) -> Result<(tmpmail::core::Inbox, MailTmCredentials), AppError> {
    let state = StateStore::at(&paths.state).load()?;
    let name = selected_name
        .or(state.active_inbox.as_deref())
        .ok_or_else(|| AppError::new("inbox_not_found", "no active inbox", 8))?;
    let inbox =
        state.inboxes.get(name).cloned().ok_or_else(|| {
            AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
        })?;
    if inbox.provider != ProviderId::MailTm {
        return Err(AppError::new(
            "provider_unavailable",
            "this inbox requires the DropMail provider",
            5,
        ));
    }
    let credentials = load_credentials(&inbox.secret_ref)?;
    Ok((inbox, credentials))
}

fn load_credentials(reference: &str) -> Result<MailTmCredentials, AppError> {
    let bytes = KeyringSecretStore
        .get(reference)?
        .ok_or_else(|| AppError::new("credential_missing", "inbox credentials are missing", 4))?;
    serde_json::from_slice(&bytes).map_err(|_| {
        AppError::new(
            "credential_invalid",
            "stored inbox credentials are invalid",
            4,
        )
    })
}

fn persist_credentials(reference: &str, credentials: &MailTmCredentials) -> Result<(), AppError> {
    let bytes = serde_json::to_vec(credentials)
        .map_err(|error| AppError::state("credential_encode", error.to_string()))?;
    KeyringSecretStore.put(reference, &bytes)
}

fn parse_duration(value: &str) -> Result<std::time::Duration, AppError> {
    let value = value.trim();
    if let Some(milliseconds) = value.strip_suffix("ms") {
        return milliseconds
            .parse::<u64>()
            .map(std::time::Duration::from_millis)
            .map_err(|_| AppError::usage("invalid timeout duration"));
    }
    if let Some(seconds) = value.strip_suffix('s') {
        return seconds
            .parse::<u64>()
            .map(std::time::Duration::from_secs)
            .map_err(|_| AppError::usage("invalid timeout duration"));
    }
    Err(AppError::usage("timeout must use ms or s suffix"))
}

fn validate_inbox_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(AppError::usage(
            "inbox name must be 1-64 letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}

fn random_password() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 32)
}

fn output_messages(cli: &Cli, messages: Vec<MessageSummary>) -> Result<(), AppError> {
    let human = if messages.is_empty() {
        "No messages.".to_owned()
    } else {
        messages
            .iter()
            .map(|message| {
                format!(
                    "{}\t{}\t{}\t{}",
                    sanitize_terminal(&message.id),
                    sanitize_terminal(&message.created_at),
                    sanitize_terminal(&message.from.address),
                    sanitize_terminal(&message.subject)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    output(cli, json!({"messages": messages}), human)
}

fn output_message(cli: &Cli, message: &Message) -> Result<(), AppError> {
    let human = format!(
        "From: {}\nSubject: {}\nDate: {}\n\n{}",
        sanitize_terminal(&message.from.address),
        sanitize_terminal(&message.subject),
        sanitize_terminal(&message.created_at),
        sanitize_terminal(&message.text)
    );
    output(cli, json!({"message": message}), human)
}

fn sanitize_terminal(value: &str) -> String {
    value
        .chars()
        .filter(|character| matches!(character, '\n' | '\r' | '\t') || !character.is_control())
        .collect()
}

fn output_inboxes(cli: &Cli, state: &AppState) -> Result<(), AppError> {
    if cli.json {
        let rows: Vec<_> = state
            .inboxes
            .values()
            .map(|inbox| {
                json!({
                    "name": inbox.name,
                    "address": inbox.address,
                    "provider": inbox.provider,
                    "active": state.active_inbox.as_deref() == Some(&inbox.name),
                })
            })
            .collect();
        return output(cli, json!({"inboxes": rows}), String::new());
    }
    if cli.quiet {
        return Ok(());
    }
    if state.inboxes.is_empty() {
        println!("No inboxes yet. Create one with `tmpmail new`.");
        return Ok(());
    }
    for inbox in state.inboxes.values() {
        let marker = if state.active_inbox.as_deref() == Some(&inbox.name) {
            "*"
        } else {
            " "
        };
        println!(
            "{marker} {:<16} {:<10} {}",
            inbox.name, inbox.provider, inbox.address
        );
    }
    Ok(())
}

fn output_providers(cli: &Cli, config: &Config) -> Result<(), AppError> {
    let providers: Vec<_> = ProviderId::ALL
        .iter()
        .map(|provider| {
            json!({
                "id": provider,
                "enabled": !config.providers.disabled.contains(provider),
            })
        })
        .collect();
    output(
        cli,
        json!({"providers": providers}),
        "mailtm\ndropmail".to_owned(),
    )
}

fn output_provider_info(cli: &Cli, provider: ProviderId) -> Result<(), AppError> {
    let (name, attribution) = match provider {
        ProviderId::MailTm => ("Mail.tm", "Powered by Mail.tm"),
        ProviderId::DropMail => ("DropMail", "DropMail disposable email"),
    };
    output(
        cli,
        json!({"provider": provider, "name": name, "attribution": attribution}),
        format!("{name}\n{attribution}"),
    )
}

fn output(cli: &Cli, data: serde_json::Value, human: String) -> Result<(), AppError> {
    if cli.json {
        let envelope = Envelope::success(data, Meta::default());
        println!(
            "{}",
            serde_json::to_string(&envelope).expect("JSON envelope")
        );
    } else if !cli.quiet {
        println!("{human}");
    }
    Ok(())
}

fn action_name(action: &Action) -> &'static str {
    match action {
        Action::MessageList { .. } => "message list",
        Action::MessageRead { .. } => "message read",
        Action::MessageLatest { .. } => "message latest",
        Action::MessageNew { .. } => "message new",
        Action::MessageAck { .. } => "message ack",
        Action::InboxCreate { .. } => "inbox create",
        Action::InboxRemove { .. } => "inbox remove",
        Action::InboxWatch { .. } => "inbox watch",
        Action::AttachmentList { .. } => "attachment list",
        Action::AttachmentSave { .. } => "attachment save",
        Action::ProviderDomains { .. } => "provider domains",
        Action::ProviderAuth { .. } => "provider auth",
        Action::ConfigGet { .. } => "config get",
        Action::ConfigSet { .. } => "config set",
        Action::ConfigUnset { .. } => "config unset",
        Action::Copy { .. } => "copy",
        Action::InboxList
        | Action::InboxUse { .. }
        | Action::ProviderList
        | Action::ProviderInfo { .. }
        | Action::ConfigShow
        | Action::Doctor => "local command",
    }
}
