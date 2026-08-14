use clap::Parser;
use futures_util::StreamExt;
use rand::distr::{Alphanumeric, SampleString};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use tmpmail::cli::{Action, Cli};
use tmpmail::config::{Config, ConfigStore};
use tmpmail::core::ProviderId;
use tmpmail::error::AppError;
use tmpmail::paths::AppPaths;
use tmpmail::providers::{
    DropMail, DropMailCredentials, MailTm, MailTmCredentials, Message, MessageSummary,
    ProviderError, ProviderFailure, creation_fallback_eligible, creation_order,
    remaining_creation_time,
};
use tmpmail::render::{Attempt, Envelope, Meta, html_to_text, safe_html_document, terminal_text};
use tmpmail::secrets::{KeyringSecretStore, SecretStore};
use tmpmail::state::{AppState, StateStore};

fn main() -> ExitCode {
    if matches!(std::env::args_os().nth(1).as_deref(), Some(value) if value == "--help" || value == "-h")
        && std::env::args_os().nth(2).is_none()
    {
        print!("{}", include_str!("help.txt"));
        return ExitCode::SUCCESS;
    }
    if matches!(
        std::env::args_os().nth(1).as_deref(),
        Some(value) if value == "--version" || value == "-V"
    ) && std::env::args_os().nth(2).is_none()
    {
        println!("tmpmail {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let cli = Cli::parse();
    match cli.validate().and_then(|_| run(&cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if error.is_silent() {
                return ExitCode::from(error.exit_code());
            } else if cli.json {
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
        Action::ConfigGet { key } => config_get(cli, &paths, &key),
        Action::ConfigSet { key, value } => config_set(cli, &paths, &key, &value),
        Action::ConfigUnset { key } => config_unset(cli, &paths, &key),
        Action::ProviderList => output_providers(cli, &ConfigStore::at(&paths.config).load()?),
        Action::ProviderInfo { provider } => output_provider_info(cli, provider),
        Action::ProviderAuth {
            provider,
            token_stdin,
        } => provider_auth(cli, &paths, provider, token_stdin),
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
        Action::MessageLatest {
            inbox,
            peek,
            html,
            open,
        } => run_async(
            cli,
            &paths,
            NetworkAction::Latest {
                inbox,
                peek,
                html,
                open,
            },
        ),
        Action::MessageRead {
            message,
            inbox,
            peek,
            html,
            open,
        } => run_async(
            cli,
            &paths,
            NetworkAction::Read {
                message,
                inbox,
                peek,
                html,
                open,
            },
        ),
        Action::MessageNew {
            inbox,
            all_inboxes: false,
            review: false,
        } => run_async(cli, &paths, NetworkAction::New(inbox)),
        Action::MessageNew {
            all_inboxes: true,
            review: false,
            ..
        } => run_async(cli, &paths, NetworkAction::NewAll),
        Action::MessageNew {
            inbox,
            all_inboxes: false,
            review: true,
        } => run_async(cli, &paths, NetworkAction::Review(inbox)),
        Action::MessageAck { messages, inbox } => {
            run_async(cli, &paths, NetworkAction::Ack { messages, inbox })
        }
        Action::AttachmentList { message, inbox } => {
            run_async(cli, &paths, NetworkAction::Attachments { message, inbox })
        }
        Action::AttachmentSave {
            message,
            attachment,
            inbox,
            force,
        } => run_async(
            cli,
            &paths,
            NetworkAction::AttachmentSave {
                message,
                attachment,
                inbox,
                force,
            },
        ),
        Action::Copy { name } => copy_address(cli, &paths, name.as_deref()),
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

fn config_get(cli: &Cli, paths: &AppPaths, key: &str) -> Result<(), AppError> {
    let config = ConfigStore::at(&paths.config).load()?;
    let value = config_value(&config, key)?;
    output(cli, json!({"key": key, "value": value}), value)
}

fn config_set(cli: &Cli, paths: &AppPaths, key: &str, value: &str) -> Result<(), AppError> {
    let store = ConfigStore::at(&paths.config);
    let mut config = store.load()?;
    match key {
        "timeout_seconds" => {
            config.timeout_seconds = value
                .parse()
                .map_err(|_| AppError::usage("timeout_seconds must be an integer"))?
        }
        "color" => config.color = parse_bool(value)?,
        "browser" => config.browser = parse_bool(value)?,
        "max_attachment_bytes" => {
            config.max_attachment_bytes = value
                .parse()
                .map_err(|_| AppError::usage("max_attachment_bytes must be an integer"))?
        }
        "providers.order" => {
            config.providers.order = value
                .split(',')
                .map(str::trim)
                .map(str::parse)
                .collect::<Result<Vec<_>, _>>()
                .map_err(AppError::usage)?;
            if config.providers.order.is_empty() {
                return Err(AppError::usage("providers.order cannot be empty"));
            }
        }
        "providers.disabled" => {
            config.providers.disabled = value
                .split(',')
                .filter(|v| !v.trim().is_empty())
                .map(str::trim)
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(AppError::usage)?
        }
        _ => return Err(AppError::usage(format!("unknown configuration key: {key}"))),
    }
    store.save(&config)?;
    output(
        cli,
        json!({"key": key, "value": config_value(&config, key)?}),
        format!("Set {key}"),
    )
}

fn config_unset(cli: &Cli, paths: &AppPaths, key: &str) -> Result<(), AppError> {
    let defaults = Config::default();
    config_set(cli, paths, key, &config_value(&defaults, key)?)
}

fn config_value(config: &Config, key: &str) -> Result<String, AppError> {
    Ok(match key {
        "timeout_seconds" => config.timeout_seconds.to_string(),
        "color" => config.color.to_string(),
        "browser" => config.browser.to_string(),
        "max_attachment_bytes" => config.max_attachment_bytes.to_string(),
        "providers.order" => config
            .providers
            .order
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(","),
        "providers.disabled" => config
            .providers
            .disabled
            .iter()
            .map(|p| p.as_str())
            .collect::<Vec<_>>()
            .join(","),
        _ => return Err(AppError::usage(format!("unknown configuration key: {key}"))),
    })
}

fn parse_bool(value: &str) -> Result<bool, AppError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(AppError::usage("boolean value must be true or false")),
    }
}

fn provider_auth(
    cli: &Cli,
    paths: &AppPaths,
    provider: ProviderId,
    token_stdin: bool,
) -> Result<(), AppError> {
    if provider != ProviderId::DropMail {
        return Err(AppError::usage(
            "Mail.tm authenticates automatically when an inbox is created",
        ));
    }
    if cli.json && !token_stdin {
        return Err(AppError::usage(
            "JSON mode requires --token-stdin for provider authentication",
        ));
    }
    let token = if token_stdin {
        let mut value = String::new();
        io::stdin()
            .read_to_string(&mut value)
            .map_err(|_| AppError::new("credential_input", "could not read token", 4))?;
        value.trim().to_owned()
    } else {
        rpassword::prompt_password("DropMail af_ token: ")
            .map_err(|_| AppError::new("credential_input", "could not read token", 4))?
            .trim()
            .to_owned()
    };
    DropMail::new(&token, parse_duration(&cli.timeout)?).map_err(|error| error.into_app_error())?;
    KeyringSecretStore.put("provider:dropmail", token.as_bytes())?;
    let store = ConfigStore::at(&paths.config);
    let mut config = store.load()?;
    config.providers.dropmail_ready = true;
    if let Err(error) = store.save(&config) {
        let _ = KeyringSecretStore.remove("provider:dropmail");
        return Err(error);
    }
    output(
        cli,
        json!({"provider": "dropmail", "configured": true}),
        "DropMail credentials stored securely.".to_owned(),
    )
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
        html: bool,
        open: bool,
    },
    Read {
        message: String,
        inbox: Option<String>,
        peek: bool,
        html: bool,
        open: bool,
    },
    New(Option<String>),
    Review(Option<String>),
    NewAll,
    Ack {
        messages: Vec<String>,
        inbox: Option<String>,
    },
    Attachments {
        message: String,
        inbox: Option<String>,
    },
    AttachmentSave {
        message: String,
        attachment: String,
        inbox: Option<String>,
        force: bool,
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
    let timeout = parse_duration(&cli.timeout)?;
    let mailtm = MailTm::new(timeout).map_err(|error| error.into_app_error())?;
    match action {
        NetworkAction::Domains(selected) => {
            let mut domains = Vec::new();
            if selected != Some(ProviderId::DropMail) {
                domains.extend(
                    mailtm
                        .domains()
                        .await
                        .map_err(|error| error.into_app_error())?,
                );
            }
            if selected == Some(ProviderId::DropMail)
                || (selected.is_none()
                    && ConfigStore::at(&paths.config)
                        .load()?
                        .providers
                        .dropmail_ready)
            {
                let token = load_dropmail_token()?;
                domains.extend(
                    DropMail::new(&token, timeout)
                        .map_err(|error| error.into_app_error())?
                        .domains()
                        .await
                        .map_err(|error| error.into_app_error())?,
                );
            }
            output(cli, json!({"domains": domains}), domains.join("\n"))
        }
        NetworkAction::Create {
            name,
            address,
            provider: selected,
        } => create_inbox(cli, paths, &mailtm, timeout, name, address, selected).await,
        NetworkAction::Remove(name) => {
            let store = StateStore::at(&paths.state);
            let mut state = store.load()?;
            let inbox = state.inboxes.get(&name).cloned().ok_or_else(|| {
                AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
            })?;
            let credentials = load_credentials(&inbox)?;
            match &credentials {
                ProviderCredentials::MailTm(value) => mailtm
                    .delete_account(&value.token, &value.account_id)
                    .await
                    .map_err(|error| error.into_app_error())?,
                ProviderCredentials::DropMail(value) => DropMail::new(&value.token, timeout)
                    .map_err(|error| error.into_app_error())?
                    .delete_address(value)
                    .await
                    .map_err(|error| error.into_app_error())?,
            }
            state.remove(&name)?;
            store.save(&state)?;
            KeyringSecretStore.remove(&inbox.secret_ref)?;
            output(cli, json!({"removed": name}), format!("Removed {name}"))
        }
        NetworkAction::List(inbox) => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let messages = fetch_messages(&mailtm, &mut credentials, timeout).await?;
            persist_credentials(&selected.secret_ref, &credentials)?;
            output_messages(cli, messages)
        }
        NetworkAction::Latest {
            inbox,
            peek,
            html,
            open,
        } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let messages = fetch_messages(&mailtm, &mut credentials, timeout).await?;
            let newest = messages
                .into_iter()
                .max_by(|a, b| a.created_at.cmp(&b.created_at))
                .ok_or_else(|| AppError::new("message_not_found", "inbox is empty", 8))?;
            let message = fetch_message(&mailtm, &mut credentials, &newest.id, timeout).await?;
            output_message(cli, paths, &selected.name, &message, html, open)?;
            if !peek {
                acknowledge_message(&mailtm, &mut credentials, &message.id).await?;
                save_acknowledgement(paths, &selected.name, &message.id)?;
            }
            persist_credentials(&selected.secret_ref, &credentials)
        }
        NetworkAction::Read {
            message,
            inbox,
            peek,
            html,
            open,
        } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let message = fetch_message(&mailtm, &mut credentials, &message, timeout).await?;
            output_message(cli, paths, &selected.name, &message, html, open)?;
            if !peek {
                acknowledge_message(&mailtm, &mut credentials, &message.id).await?;
                save_acknowledgement(paths, &selected.name, &message.id)?;
            }
            persist_credentials(&selected.secret_ref, &credentials)
        }
        NetworkAction::New(inbox) => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            let state = StateStore::at(&paths.state).load()?;
            let messages = fetch_messages(&mailtm, &mut credentials, timeout)
                .await?
                .into_iter()
                .filter(|message| {
                    !message.seen && !state.is_acknowledged(&selected.name, &message.id)
                })
                .collect();
            persist_credentials(&selected.secret_ref, &credentials)?;
            output_messages(cli, messages)
        }
        NetworkAction::Review(inbox) => {
            review_messages(cli, paths, &mailtm, timeout, inbox.as_deref()).await
        }
        NetworkAction::NewAll => new_all_inboxes(cli, paths, timeout).await,
        NetworkAction::Ack { messages, inbox } => {
            let (selected, mut credentials) = selected_credentials(paths, inbox.as_deref())?;
            for id in &messages {
                acknowledge_message(&mailtm, &mut credentials, id).await?;
                save_acknowledgement(paths, &selected.name, id)?;
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
            let message = fetch_message(&mailtm, &mut credentials, &message, timeout).await?;
            persist_credentials(&selected.secret_ref, &credentials)?;
            let human = message
                .attachments
                .iter()
                .map(|item| format!("{}\t{}\t{} bytes", item.id, item.filename, item.size))
                .collect::<Vec<_>>()
                .join("\n");
            output(cli, json!({"attachments": message.attachments}), human)
        }
        NetworkAction::AttachmentSave {
            message,
            attachment,
            inbox,
            force,
        } => {
            save_attachment(
                cli,
                paths,
                &mailtm,
                timeout,
                AttachmentSaveRequest {
                    inbox,
                    message,
                    attachment,
                    force,
                },
            )
            .await
        }
    }
}

async fn review_messages(
    cli: &Cli,
    paths: &AppPaths,
    mailtm: &MailTm,
    timeout: std::time::Duration,
    inbox: Option<&str>,
) -> Result<(), AppError> {
    let (selected, mut credentials) = selected_credentials(paths, inbox)?;
    let state = StateStore::at(&paths.state).load()?;
    let messages: Vec<_> = fetch_messages(mailtm, &mut credentials, timeout)
        .await?
        .into_iter()
        .filter(|message| !message.seen && !state.is_acknowledged(&selected.name, &message.id))
        .collect();
    if messages.is_empty() {
        return output(cli, json!({"messages": []}), "No new messages.".to_owned());
    }
    loop {
        for (index, message) in messages.iter().enumerate() {
            println!(
                "{}. {} — {}",
                index + 1,
                terminal_text(&message.from.address),
                terminal_text(&message.subject)
            );
        }
        eprint!("review [r/a/s NUMBER, q]: ");
        io::stderr()
            .flush()
            .map_err(|e| AppError::state("review", e.to_string()))?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| AppError::state("review", e.to_string()))?;
        let trimmed = input.trim();
        if trimmed == "q" {
            break;
        }
        let mut parts = trimmed.split_whitespace();
        let action = parts.next().unwrap_or_default();
        let index = parts
            .next()
            .and_then(|value| value.parse::<usize>().ok())
            .and_then(|value| value.checked_sub(1));
        let Some(summary) = index.and_then(|value| messages.get(value)) else {
            eprintln!("Choose a listed message.");
            continue;
        };
        match action {
            "r" => {
                let message = fetch_message(mailtm, &mut credentials, &summary.id, timeout).await?;
                output_message(cli, paths, &selected.name, &message, false, false)?;
                acknowledge_message(mailtm, &mut credentials, &message.id).await?;
                save_acknowledgement(paths, &selected.name, &message.id)?;
            }
            "a" => {
                acknowledge_message(mailtm, &mut credentials, &summary.id).await?;
                save_acknowledgement(paths, &selected.name, &summary.id)?;
            }
            "s" => {}
            _ => eprintln!("Use r, a, s, or q."),
        }
    }
    persist_credentials(&selected.secret_ref, &credentials)
}

enum CreatedCredentials {
    MailTm(MailTmCredentials),
    DropMail(DropMailCredentials),
}

async fn create_inbox(
    cli: &Cli,
    paths: &AppPaths,
    mailtm: &MailTm,
    timeout: std::time::Duration,
    name: Option<String>,
    address: Option<String>,
    selected: Option<ProviderId>,
) -> Result<(), AppError> {
    let name = name.unwrap_or_else(|| "default".to_owned());
    validate_inbox_name(&name)?;
    let mut state = StateStore::at(&paths.state).load()?;
    if state.inboxes.contains_key(&name) {
        return Err(AppError::usage(format!("inbox already exists: {name}")));
    }
    if selected == Some(ProviderId::DropMail) && address.is_some() {
        return Err(AppError::usage(
            "DropMail does not support custom local-part creation",
        ));
    }
    let config = ConfigStore::at(&paths.config).load()?;
    let providers = creation_order(
        selected,
        &config.providers.order,
        &cli.no_provider,
        &config.providers.disabled,
        config.providers.dropmail_ready,
        address.is_some(),
    );
    if providers.is_empty() {
        return Err(AppError::new(
            "provider_exhausted",
            "no eligible provider is configured",
            5,
        ));
    }
    let started = std::time::Instant::now();
    let mut attempts = Vec::new();
    let mut created = None;
    for provider_id in providers {
        let remaining = remaining_creation_time(timeout, started.elapsed())
            .ok_or_else(|| AppError::new("provider_timeout", "creation deadline expired", 7))?;
        let attempt_started = std::time::Instant::now();
        let result = match provider_id {
            ProviderId::MailTm => create_mailtm(mailtm, address.as_deref())
                .await
                .map(CreatedCredentials::MailTm),
            ProviderId::DropMail => create_dropmail(remaining)
                .await
                .map(CreatedCredentials::DropMail),
        };
        match result {
            Ok(value) => {
                attempts.push(Attempt {
                    provider: provider_id,
                    duration_ms: attempt_started.elapsed().as_millis() as u64,
                    reason: "created".to_owned(),
                    retryable: false,
                });
                created = Some((provider_id, value));
                break;
            }
            Err(error) => {
                let retryable = error.retryable();
                attempts.push(Attempt {
                    provider: provider_id,
                    duration_ms: attempt_started.elapsed().as_millis() as u64,
                    reason: provider_reason(&error.failure).to_owned(),
                    retryable,
                });
                if selected.is_some() || !creation_fallback_eligible(&error.failure) {
                    return Err(error.into_app_error());
                }
            }
        }
    }
    let (provider_id, created) = created
        .ok_or_else(|| AppError::new("provider_exhausted", "all eligible providers failed", 5))?;
    let fallback_used = attempts.len() > 1;
    let (address_value, secret_ref, capabilities, serialized) = match &created {
        CreatedCredentials::MailTm(value) => (
            value.address.clone(),
            format!("mailtm:{name}"),
            tmpmail::core::Capabilities::mailtm(),
            serde_json::to_vec(value),
        ),
        CreatedCredentials::DropMail(value) => (
            value.address.clone(),
            format!("dropmail:{name}"),
            tmpmail::core::Capabilities::dropmail(),
            serde_json::to_vec(value),
        ),
    };
    let serialized =
        serialized.map_err(|error| AppError::state("credential_encode", error.to_string()))?;
    KeyringSecretStore.put(&secret_ref, &serialized)?;
    state.insert(tmpmail::core::Inbox::new(
        &name,
        &address_value,
        provider_id,
        &secret_ref,
        capabilities,
    ));
    if state.active_inbox.is_none() {
        state.set_active(&name)?;
    }
    if let Err(error) = StateStore::at(&paths.state).save(&state) {
        cleanup_created(mailtm, timeout, &created).await;
        let _ = KeyringSecretStore.remove(&secret_ref);
        return Err(error);
    }
    if fallback_used && !cli.quiet {
        let first = &attempts[0];
        eprintln!(
            "{} unavailable ({}) → created via {}",
            provider_display(first.provider),
            first.reason,
            provider_display(provider_id)
        );
    }
    output_with_meta(
        cli,
        json!({"name": name, "address": address_value, "provider": provider_id}),
        format!(
            "{}  ({name}, {})",
            address_value,
            provider_display(provider_id)
        ),
        Meta {
            provider: Some(provider_id),
            fallback_used,
            attempts,
        },
    )
}

async fn create_mailtm(
    provider: &MailTm,
    address: Option<&str>,
) -> Result<MailTmCredentials, ProviderError> {
    match address {
        Some(address) if address.contains('@') => {
            provider.create(address, &random_password()).await
        }
        Some(local) => provider.create_random(Some(local)).await,
        None => provider.create_random(None).await,
    }
}

async fn create_dropmail(
    timeout: std::time::Duration,
) -> Result<DropMailCredentials, ProviderError> {
    let token = load_dropmail_token().map_err(|_| {
        ProviderError::for_provider(
            ProviderId::DropMail,
            ProviderFailure::Authentication,
            "DropMail credentials are unavailable",
        )
    })?;
    DropMail::new(&token, timeout)?.create().await
}

fn load_dropmail_token() -> Result<String, AppError> {
    let token = KeyringSecretStore
        .get("provider:dropmail")?
        .ok_or_else(|| {
            AppError::new("credential_missing", "DropMail token is not configured", 4)
        })?;
    String::from_utf8(token)
        .map_err(|_| AppError::new("credential_invalid", "stored DropMail token is invalid", 4))
}

async fn cleanup_created(
    mailtm: &MailTm,
    timeout: std::time::Duration,
    created: &CreatedCredentials,
) {
    match created {
        CreatedCredentials::MailTm(value) => {
            let _ = mailtm.delete_account(&value.token, &value.account_id).await;
        }
        CreatedCredentials::DropMail(value) => {
            if let Ok(provider) = DropMail::new(&value.token, timeout) {
                let _ = provider.delete_address(value).await;
            }
        }
    }
}

fn provider_reason(failure: &ProviderFailure) -> &'static str {
    match failure {
        ProviderFailure::Authentication => "authentication",
        ProviderFailure::RateLimited { .. } => "rate_limit",
        ProviderFailure::Unavailable => "unavailable",
        ProviderFailure::Timeout => "timeout",
        ProviderFailure::NotFound => "not_found",
        ProviderFailure::Conflict => "rejected",
        ProviderFailure::InvalidResponse => "invalid_response",
    }
}

fn provider_display(provider: ProviderId) -> &'static str {
    match provider {
        ProviderId::MailTm => "Mail.tm",
        ProviderId::DropMail => "DropMail",
    }
}

fn selected_credentials(
    paths: &AppPaths,
    selected_name: Option<&str>,
) -> Result<(tmpmail::core::Inbox, ProviderCredentials), AppError> {
    let state = StateStore::at(&paths.state).load()?;
    let name = selected_name
        .or(state.active_inbox.as_deref())
        .ok_or_else(|| AppError::new("inbox_not_found", "no active inbox", 8))?;
    let inbox =
        state.inboxes.get(name).cloned().ok_or_else(|| {
            AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8)
        })?;
    let credentials = load_credentials(&inbox)?;
    Ok((inbox, credentials))
}

enum ProviderCredentials {
    MailTm(MailTmCredentials),
    DropMail(DropMailCredentials),
}

fn load_credentials(inbox: &tmpmail::core::Inbox) -> Result<ProviderCredentials, AppError> {
    let bytes = KeyringSecretStore
        .get(&inbox.secret_ref)?
        .ok_or_else(|| AppError::new("credential_missing", "inbox credentials are missing", 4))?;
    match inbox.provider {
        ProviderId::MailTm => serde_json::from_slice(&bytes)
            .map(ProviderCredentials::MailTm)
            .map_err(|_| invalid_credentials()),
        ProviderId::DropMail => serde_json::from_slice(&bytes)
            .map(ProviderCredentials::DropMail)
            .map_err(|_| invalid_credentials()),
    }
}

fn persist_credentials(reference: &str, credentials: &ProviderCredentials) -> Result<(), AppError> {
    let bytes = match credentials {
        ProviderCredentials::MailTm(value) => serde_json::to_vec(value),
        ProviderCredentials::DropMail(value) => serde_json::to_vec(value),
    }
    .map_err(|error| AppError::state("credential_encode", error.to_string()))?;
    KeyringSecretStore.put(reference, &bytes)
}

fn invalid_credentials() -> AppError {
    AppError::new(
        "credential_invalid",
        "stored inbox credentials are invalid",
        4,
    )
}

async fn fetch_messages(
    mailtm: &MailTm,
    credentials: &mut ProviderCredentials,
    timeout: std::time::Duration,
) -> Result<Vec<MessageSummary>, AppError> {
    match credentials {
        ProviderCredentials::MailTm(value) => mailtm
            .messages_with_refresh(value)
            .await
            .map_err(|error| error.into_app_error()),
        ProviderCredentials::DropMail(value) => DropMail::new(&value.token, timeout)
            .map_err(|error| error.into_app_error())?
            .messages(value)
            .await
            .map_err(|error| error.into_app_error()),
    }
}

async fn fetch_message(
    mailtm: &MailTm,
    credentials: &mut ProviderCredentials,
    id: &str,
    timeout: std::time::Duration,
) -> Result<Message, AppError> {
    match credentials {
        ProviderCredentials::MailTm(value) => mailtm
            .message_with_refresh(value, id)
            .await
            .map_err(|error| error.into_app_error()),
        ProviderCredentials::DropMail(value) => DropMail::new(&value.token, timeout)
            .map_err(|error| error.into_app_error())?
            .message(value, id)
            .await
            .map_err(|error| error.into_app_error()),
    }
}

async fn acknowledge_message(
    mailtm: &MailTm,
    credentials: &mut ProviderCredentials,
    id: &str,
) -> Result<(), AppError> {
    if let ProviderCredentials::MailTm(value) = credentials {
        mailtm
            .acknowledge_with_refresh(value, id)
            .await
            .map_err(|error| error.into_app_error())?;
    }
    Ok(())
}

fn save_acknowledgement(paths: &AppPaths, inbox: &str, message: &str) -> Result<(), AppError> {
    let store = StateStore::at(&paths.state);
    let mut state = store.load()?;
    state.acknowledge_id(inbox, message);
    store.save(&state)
}

fn copy_address(cli: &Cli, paths: &AppPaths, selected: Option<&str>) -> Result<(), AppError> {
    let state = StateStore::at(&paths.state).load()?;
    let name = selected
        .or(state.active_inbox.as_deref())
        .ok_or_else(|| AppError::new("inbox_not_found", "no active inbox", 8))?;
    let inbox = state
        .inboxes
        .get(name)
        .ok_or_else(|| AppError::new("inbox_not_found", format!("inbox not found: {name}"), 8))?;
    tmpmail::platform::copy(&inbox.address)?;
    output(
        cli,
        json!({"copied": true, "inbox": name}),
        format!("Copied {}", inbox.address),
    )
}

struct AttachmentSaveRequest {
    inbox: Option<String>,
    message: String,
    attachment: String,
    force: bool,
}

async fn save_attachment(
    cli: &Cli,
    paths: &AppPaths,
    mailtm: &MailTm,
    timeout: std::time::Duration,
    request: AttachmentSaveRequest,
) -> Result<(), AppError> {
    let (selected, mut credentials) = selected_credentials(paths, request.inbox.as_deref())?;
    let message = fetch_message(mailtm, &mut credentials, &request.message, timeout).await?;
    let attachment = message
        .attachments
        .iter()
        .find(|item| item.id == request.attachment)
        .ok_or_else(|| AppError::new("attachment_not_found", "attachment not found", 8))?;
    let filename = safe_basename(&attachment.filename)?;
    let destination = PathBuf::from(&filename);
    if destination.exists() && !request.force {
        return Err(AppError::new(
            "unsafe_path",
            "destination exists; use --force to replace it",
            9,
        ));
    }
    if fs::symlink_metadata(&destination).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(AppError::new(
            "unsafe_path",
            "refusing to replace a symbolic link",
            9,
        ));
    }
    let response = match &credentials {
        ProviderCredentials::MailTm(value) => {
            mailtm
                .attachment(&value.token, &attachment.download_url)
                .await
        }
        ProviderCredentials::DropMail(value) => {
            DropMail::new(&value.token, timeout)
                .map_err(|error| error.into_app_error())?
                .attachment(&attachment.download_url)
                .await
        }
    }
    .map_err(|error| error.into_app_error())?;
    let limit = ConfigStore::at(&paths.config).load()?.max_attachment_bytes;
    if attachment.size > limit || response.content_length().is_some_and(|size| size > limit) {
        return Err(AppError::new(
            "attachment_too_large",
            "attachment exceeds configured size limit",
            9,
        ));
    }
    let temporary = destination.with_extension(format!("{}.tmpmail-part", std::process::id()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)
        .map_err(|error| {
            AppError::new(
                "unsafe_path",
                format!("could not create attachment: {error}"),
                9,
            )
        })?;
    let mut received = 0_u64;
    let mut stream = response.bytes_stream();
    let result: Result<(), AppError> = async {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| {
                AppError::new(
                    "attachment_download",
                    "attachment download was interrupted",
                    5,
                )
            })?;
            received = received.saturating_add(chunk.len() as u64);
            if received > limit {
                return Err(AppError::new(
                    "attachment_too_large",
                    "attachment exceeds configured size limit",
                    9,
                ));
            }
            file.write_all(&chunk)
                .map_err(|error| AppError::state("attachment_write", error.to_string()))?;
        }
        file.sync_all()
            .map_err(|error| AppError::state("attachment_write", error.to_string()))?;
        if attachment.size != 0 && received != attachment.size {
            return Err(AppError::new(
                "attachment_truncated",
                "attachment size did not match provider metadata",
                5,
            ));
        }
        Ok(())
    }
    .await;
    if let Err(error) = result {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    fs::rename(&temporary, &destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        AppError::state("attachment_write", error.to_string())
    })?;
    persist_credentials(&selected.secret_ref, &credentials)?;
    output(
        cli,
        json!({"saved": destination, "bytes": received}),
        format!("Saved {filename}"),
    )
}

fn safe_basename(value: &str) -> Result<String, AppError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.chars().any(|character| character.is_control())
    {
        return Err(AppError::new("unsafe_path", "unsafe filename refused", 9));
    }
    let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
    if reserved.iter().any(|name| value.eq_ignore_ascii_case(name)) {
        return Err(AppError::new("unsafe_path", "reserved filename refused", 9));
    }
    Ok(value.to_owned())
}

async fn new_all_inboxes(
    cli: &Cli,
    paths: &AppPaths,
    timeout: std::time::Duration,
) -> Result<(), AppError> {
    let state = StateStore::at(&paths.state).load()?;
    let mut jobs = Vec::new();
    for inbox in state.inboxes.values().cloned() {
        let mut credentials = load_credentials(&inbox)?;
        jobs.push(async move {
            let mailtm = MailTm::new(timeout).map_err(|error| error.into_app_error())?;
            let messages = fetch_messages(&mailtm, &mut credentials, timeout).await?;
            Ok::<_, AppError>((inbox, credentials, messages))
        });
    }
    let results = futures_util::future::join_all(jobs).await;
    let mut rows = Vec::new();
    let mut human = Vec::new();
    let mut failures = Vec::new();
    for result in results {
        match result {
            Ok((inbox, credentials, messages)) => {
                persist_credentials(&inbox.secret_ref, &credentials)?;
                for message in messages.into_iter().filter(|message| {
                    !message.seen && !state.is_acknowledged(&inbox.name, &message.id)
                }) {
                    human.push(format!(
                        "{}\t{}\t{}\t{}",
                        inbox.name, message.id, message.from.address, message.subject
                    ));
                    rows.push(json!({
                        "inbox": inbox.name,
                        "provider": inbox.provider,
                        "message": message,
                    }));
                }
            }
            Err(error) => failures.push(json!({
                "code": error.code(),
                "message": error.to_string(),
                "retryable": error.retryable(),
            })),
        }
    }
    if !failures.is_empty() {
        if cli.json {
            let error = AppError::new(
                "partial_operation",
                format!("{} inbox operation(s) failed", failures.len()),
                10,
            );
            let mut envelope = Envelope::<serde_json::Value>::failure(&error, Meta::default());
            envelope.data = Some(json!({"messages": rows, "failures": failures}));
            println!(
                "{}",
                serde_json::to_string(&envelope).expect("JSON envelope")
            );
            return Err(
                AppError::new("partial_operation", "partial result already rendered", 10).silent(),
            );
        }
        for failure in &failures {
            eprintln!(
                "inbox failed: {}",
                failure["message"].as_str().unwrap_or("provider failure")
            );
        }
        if !human.is_empty() {
            println!("{}", human.join("\n"));
        }
        return Err(AppError::new(
            "partial_operation",
            format!("{} inbox operation(s) failed", failures.len()),
            10,
        )
        .silent());
    }
    output(
        cli,
        json!({"messages": rows}),
        if human.is_empty() {
            "No new messages.".to_owned()
        } else {
            human.join("\n")
        },
    )
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
                    terminal_text(&message.id),
                    terminal_text(&message.created_at),
                    terminal_text(&message.from.address),
                    terminal_text(&message.subject)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    output(cli, json!({"messages": messages}), human)
}

fn output_message(
    cli: &Cli,
    paths: &AppPaths,
    inbox: &str,
    message: &Message,
    html: bool,
    open: bool,
) -> Result<(), AppError> {
    let body = if message.text.trim().is_empty() {
        html_to_text(&message.html.join("\n"))
    } else {
        terminal_text(&message.text)
    };
    let human = format!(
        "From: {}\nSubject: {}\nDate: {}\n\n{}",
        terminal_text(&message.from.address),
        terminal_text(&message.subject),
        terminal_text(&message.created_at),
        body
    );
    output(cli, json!({"message": message}), human)?;
    if html {
        let directory = paths.cache.join("messages");
        fs::create_dir_all(&directory).map_err(|e| AppError::state("html_cache", e.to_string()))?;
        let path = directory.join(format!(
            "{}-{}.html",
            safe_basename(inbox)?,
            safe_basename(&message.id)?
        ));
        fs::write(&path, safe_html_document(&body))
            .map_err(|e| AppError::state("html_cache", e.to_string()))?;
        if open {
            tmpmail::platform::open_browser(&path)?;
        }
    }
    Ok(())
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
                "configured": *provider != ProviderId::DropMail || config.providers.dropmail_ready,
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
    output_with_meta(cli, data, human, Meta::default())
}

fn output_with_meta(
    cli: &Cli,
    data: serde_json::Value,
    human: String,
    meta: Meta,
) -> Result<(), AppError> {
    if cli.json {
        let envelope = Envelope::success(data, meta);
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
