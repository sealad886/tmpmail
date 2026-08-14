use crate::core::ProviderId;
use crate::error::AppError;
use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "tmpmail",
    version,
    about = "Fast, friendly temporary email",
    long_about = "Fast, friendly temporary email\n\nMail.tm functionality is powered by Mail.tm: https://mail.tm",
    subcommand_help_heading = "Commands",
    disable_help_subcommand = true
)]
pub struct Cli {
    #[arg(long, global = true)]
    pub inbox: Option<String>,
    #[arg(long, global = true)]
    pub provider: Option<ProviderId>,
    #[arg(long = "no-provider", global = true)]
    pub no_provider: Vec<ProviderId>,
    #[arg(long, global = true)]
    pub json: bool,
    #[arg(long, global = true)]
    pub no_color: bool,
    #[arg(long, global = true)]
    pub timeout: Option<String>,
    #[arg(long, global = true)]
    pub quiet: bool,
    #[arg(long, global = true)]
    pub verbose: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

impl Cli {
    pub fn try_parse_from<I, T>(values: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(values)
    }

    pub fn validate(&self) -> Result<(), AppError> {
        if self.quiet && self.verbose {
            return Err(AppError::usage("--quiet conflicts with --verbose"));
        }
        if self.json && matches!(self.action(), Action::MessageNew { review: true, .. }) {
            return Err(AppError::usage(
                "--review is interactive and unavailable with --json",
            ));
        }
        if matches!(
            self.action(),
            Action::MessageNew {
                all_inboxes: true,
                review: true,
                ..
            }
        ) {
            return Err(AppError::usage("--review conflicts with --all-inboxes"));
        }
        Ok(())
    }

    pub fn action(&self) -> Action {
        match &self.command {
            None | Some(Command::Ls) => Action::MessageList {
                inbox: self.inbox.clone(),
            },
            Some(Command::Message { command }) => message_action(command, self.inbox.clone()),
            Some(Command::Read {
                message,
                peek,
                html,
                open,
            }) => Action::MessageRead {
                message: message.clone(),
                inbox: self.inbox.clone(),
                peek: *peek,
                html: *html,
                open: *open,
            },
            Some(Command::Latest { peek, html, open }) => Action::MessageLatest {
                inbox: self.inbox.clone(),
                peek: *peek,
                html: *html,
                open: *open,
            },
            Some(Command::NewMail { all_inboxes }) | Some(Command::Unread { all_inboxes }) => {
                Action::MessageNew {
                    inbox: self.inbox.clone(),
                    all_inboxes: *all_inboxes,
                    review: false,
                }
            }
            Some(Command::Inbox { command }) => inbox_action(command, self.provider),
            Some(Command::New { name, address }) => Action::InboxCreate {
                name: name.clone(),
                address: address.clone(),
                provider: self.provider,
            },
            Some(Command::Inboxes) => Action::InboxList,
            Some(Command::Use { name }) => Action::InboxUse { name: name.clone() },
            Some(Command::Watch {
                names,
                all,
                existing,
            }) => Action::InboxWatch {
                names: names.clone(),
                all: *all,
                existing: *existing,
            },
            Some(Command::Attachment { command }) => match command {
                AttachmentCommand::List { message } => Action::AttachmentList {
                    message: message.clone(),
                    inbox: self.inbox.clone(),
                },
                AttachmentCommand::Save {
                    message,
                    attachment,
                    force,
                } => Action::AttachmentSave {
                    message: message.clone(),
                    attachment: attachment.clone(),
                    inbox: self.inbox.clone(),
                    force: *force,
                },
            },
            Some(Command::Provider { command }) => match command {
                ProviderCommand::List => Action::ProviderList,
                ProviderCommand::Domains { provider } => Action::ProviderDomains {
                    provider: *provider,
                },
                ProviderCommand::Auth {
                    provider,
                    token_stdin,
                } => Action::ProviderAuth {
                    provider: *provider,
                    token_stdin: *token_stdin,
                },
                ProviderCommand::Info { provider } => Action::ProviderInfo {
                    provider: *provider,
                },
            },
            Some(Command::Config { command }) => match command {
                ConfigCommand::Show => Action::ConfigShow,
                ConfigCommand::Get { key } => Action::ConfigGet { key: key.clone() },
                ConfigCommand::Set { key, value } => Action::ConfigSet {
                    key: key.clone(),
                    value: value.clone(),
                },
                ConfigCommand::Unset { key } => Action::ConfigUnset { key: key.clone() },
            },
            Some(Command::Copy { name }) => Action::Copy {
                name: name.clone().or_else(|| self.inbox.clone()),
            },
            Some(Command::Doctor) => Action::Doctor,
        }
    }
}

fn message_action(command: &MessageCommand, inbox: Option<String>) -> Action {
    match command {
        MessageCommand::List => Action::MessageList { inbox },
        MessageCommand::Read {
            message,
            peek,
            html,
            open,
        } => Action::MessageRead {
            message: message.clone(),
            inbox,
            peek: *peek,
            html: *html,
            open: *open,
        },
        MessageCommand::Latest { peek, html, open } => Action::MessageLatest {
            inbox,
            peek: *peek,
            html: *html,
            open: *open,
        },
        MessageCommand::New {
            all_inboxes,
            review,
        } => Action::MessageNew {
            inbox,
            all_inboxes: *all_inboxes,
            review: *review,
        },
        MessageCommand::Ack { messages } => Action::MessageAck {
            messages: messages.clone(),
            inbox,
        },
    }
}

fn inbox_action(command: &InboxCommand, provider: Option<ProviderId>) -> Action {
    match command {
        InboxCommand::Create { name, address } => Action::InboxCreate {
            name: name.clone(),
            address: address.clone(),
            provider,
        },
        InboxCommand::List => Action::InboxList,
        InboxCommand::Use { name } => Action::InboxUse { name: name.clone() },
        InboxCommand::Remove { name } => Action::InboxRemove { name: name.clone() },
        InboxCommand::Watch {
            names,
            all,
            existing,
        } => Action::InboxWatch {
            names: names.clone(),
            all: *all,
            existing: *existing,
        },
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Work with messages
    Message {
        #[command(subcommand)]
        command: MessageCommand,
    },
    /// Work with named inboxes
    Inbox {
        #[command(subcommand)]
        command: InboxCommand,
    },
    /// Work with message attachments
    Attachment {
        #[command(subcommand)]
        command: AttachmentCommand,
    },
    /// Inspect and configure providers
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    /// Inspect and update local configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Alias for `message list`
    Ls,
    /// Alias for `message read`
    Read {
        message: String,
        #[arg(long)]
        peek: bool,
        #[arg(long)]
        html: bool,
        #[arg(long, requires = "html")]
        open: bool,
    },
    /// Alias for `message latest`
    Latest {
        #[arg(long)]
        peek: bool,
        #[arg(long)]
        html: bool,
        #[arg(long, requires = "html")]
        open: bool,
    },
    /// Alias for `inbox create`
    New {
        name: Option<String>,
        #[arg(long)]
        address: Option<String>,
    },
    /// Alias for `message new`
    NewMail {
        #[arg(long)]
        all_inboxes: bool,
    },
    /// Alias for `message new`
    Unread {
        #[arg(long)]
        all_inboxes: bool,
    },
    /// Alias for `inbox list`
    Inboxes,
    /// Alias for `inbox use`
    Use { name: String },
    /// Alias for `inbox watch`
    Watch {
        names: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        existing: bool,
    },
    /// Copy an inbox address
    Copy { name: Option<String> },
    /// Diagnose local configuration
    Doctor,
}

#[derive(Debug, Clone, Subcommand)]
enum MessageCommand {
    List,
    Read {
        message: String,
        #[arg(long)]
        peek: bool,
        #[arg(long)]
        html: bool,
        #[arg(long, requires = "html")]
        open: bool,
    },
    Latest {
        #[arg(long)]
        peek: bool,
        #[arg(long)]
        html: bool,
        #[arg(long, requires = "html")]
        open: bool,
    },
    New {
        #[arg(long)]
        all_inboxes: bool,
        #[arg(long)]
        review: bool,
    },
    Ack {
        messages: Vec<String>,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum InboxCommand {
    Create {
        name: Option<String>,
        #[arg(long)]
        address: Option<String>,
    },
    List,
    Use {
        name: String,
    },
    Remove {
        name: String,
    },
    Watch {
        names: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        existing: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum AttachmentCommand {
    List {
        message: String,
    },
    Save {
        message: String,
        attachment: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum ProviderCommand {
    List,
    Domains {
        provider: Option<ProviderId>,
    },
    Auth {
        provider: ProviderId,
        #[arg(long)]
        token_stdin: bool,
    },
    Info {
        provider: ProviderId,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigCommand {
    Show,
    Get { key: String },
    Set { key: String, value: String },
    Unset { key: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    MessageList {
        inbox: Option<String>,
    },
    MessageRead {
        message: String,
        inbox: Option<String>,
        peek: bool,
        html: bool,
        open: bool,
    },
    MessageLatest {
        inbox: Option<String>,
        peek: bool,
        html: bool,
        open: bool,
    },
    MessageNew {
        inbox: Option<String>,
        all_inboxes: bool,
        review: bool,
    },
    MessageAck {
        messages: Vec<String>,
        inbox: Option<String>,
    },
    InboxCreate {
        name: Option<String>,
        address: Option<String>,
        provider: Option<ProviderId>,
    },
    InboxList,
    InboxUse {
        name: String,
    },
    InboxRemove {
        name: String,
    },
    InboxWatch {
        names: Vec<String>,
        all: bool,
        existing: bool,
    },
    AttachmentList {
        message: String,
        inbox: Option<String>,
    },
    AttachmentSave {
        message: String,
        attachment: String,
        inbox: Option<String>,
        force: bool,
    },
    ProviderList,
    ProviderDomains {
        provider: Option<ProviderId>,
    },
    ProviderAuth {
        provider: ProviderId,
        token_stdin: bool,
    },
    ProviderInfo {
        provider: ProviderId,
    },
    ConfigShow,
    ConfigGet {
        key: String,
    },
    ConfigSet {
        key: String,
        value: String,
    },
    ConfigUnset {
        key: String,
    },
    Copy {
        name: Option<String>,
    },
    Doctor,
}
