use std::path::Path;
use tmpmail::platform::{browser_command, clipboard_command};

#[test]
fn platform_commands_are_constructed_without_shell_interpolation() {
    let clipboard = clipboard_command();
    assert!(!clipboard.program.is_empty());
    let browser = browser_command(Path::new("/tmp/message with spaces.html"));
    assert!(!browser.program.is_empty());
    assert_eq!(browser.args, ["/tmp/message with spaces.html"]);
    assert!(!browser.args[0].contains("file://"));
}
