use clap::Parser;
use serde_json::json;
use std::process::ExitCode;
use tmpmail::cli::{Action, Cli};
use tmpmail::config::{Config, ConfigStore};
use tmpmail::core::ProviderId;
use tmpmail::error::AppError;
use tmpmail::paths::AppPaths;
use tmpmail::render::{Envelope, Meta};
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
