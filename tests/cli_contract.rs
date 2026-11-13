use tmpmail::cli::{Action, Cli};

fn action(args: &[&str]) -> Action {
    Cli::try_parse_from(args)
        .expect("command should parse")
        .action()
}

#[test]
fn root_and_ls_normalize_to_message_list() {
    assert_eq!(action(&["tmpmail"]), Action::MessageList { inbox: None });
    assert_eq!(
        action(&["tmpmail", "ls"]),
        action(&["tmpmail", "message", "list"])
    );
}

#[test]
fn read_and_latest_aliases_match_canonical_commands() {
    assert_eq!(
        action(&["tmpmail", "read", "abc"]),
        action(&["tmpmail", "message", "read", "abc"])
    );
    assert_eq!(
        action(&["tmpmail", "latest"]),
        action(&["tmpmail", "message", "latest"])
    );
    assert_eq!(
        action(&["tmpmail", "read", "abc", "--raw"]),
        action(&["tmpmail", "message", "read", "abc", "--raw"])
    );
    assert!(Cli::try_parse_from(["tmpmail", "read", "abc", "--raw", "--html"]).is_err());
}

#[test]
fn creation_and_new_mail_aliases_do_not_conflict() {
    assert_eq!(
        action(&["tmpmail", "new", "work"]),
        action(&["tmpmail", "inbox", "create", "work"])
    );
    assert_eq!(
        action(&["tmpmail", "new-mail"]),
        action(&["tmpmail", "message", "new"])
    );
    assert_eq!(
        action(&["tmpmail", "unread", "--all-inboxes"]),
        action(&["tmpmail", "message", "new", "--all-inboxes"])
    );
}

#[test]
fn json_mode_rejects_interactive_review() {
    let error = Cli::try_parse_from(["tmpmail", "--json", "message", "new", "--review"])
        .expect("syntax should parse")
        .validate()
        .expect_err("JSON review must be rejected");
    assert_eq!(error.code(), "usage");
}

#[test]
fn review_rejects_all_inboxes_mode() {
    let error = Cli::try_parse_from(["tmpmail", "message", "new", "--review", "--all-inboxes"])
        .expect("syntax should parse")
        .validate()
        .expect_err("modes conflict");
    assert_eq!(error.code(), "usage");
}

#[test]
fn acknowledgement_requires_at_least_one_message() {
    assert!(Cli::try_parse_from(["tmpmail", "message", "ack"]).is_err());
}

#[test]
fn watch_honors_global_inbox_and_explicit_names_take_precedence() {
    for command in [vec!["watch"], vec!["inbox", "watch"]] {
        let mut arguments = vec!["tmpmail", "--inbox", "work"];
        arguments.extend(command);
        assert_eq!(
            action(&arguments),
            Action::InboxWatch {
                names: vec!["work".to_owned()],
                all: false,
                existing: false,
            }
        );
        arguments.push("personal");
        assert_eq!(
            action(&arguments),
            Action::InboxWatch {
                names: vec!["personal".to_owned()],
                all: false,
                existing: false,
            }
        );
    }
}

#[test]
fn watch_rejects_all_with_an_explicit_selection() {
    for arguments in [
        vec!["tmpmail", "watch", "work", "--all"],
        vec!["tmpmail", "--inbox", "work", "inbox", "watch", "--all"],
    ] {
        let error = Cli::try_parse_from(arguments)
            .expect("syntax should parse")
            .validate()
            .expect_err("selection modes conflict");
        assert_eq!(error.code(), "usage");
    }
}
