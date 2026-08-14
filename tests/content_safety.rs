use tmpmail::render::{html_to_text, safe_html_document, terminal_text};

#[test]
fn strips_terminal_controls_carriage_returns_and_bidi_overrides() {
    let hostile = "safe\x1b[31mred\rreplace\u{202e}spoof\nnext";
    assert_eq!(terminal_text(hostile), "safe[31mred\nreplacespoof\nnext");
}

#[test]
fn converts_html_to_plain_text_without_active_content() {
    let html = r#"<p>Hello &amp; welcome</p><script>alert(1)</script><img src="https://tracker.test/pixel">"#;
    let text = html_to_text(html);
    assert!(text.contains("Hello & welcome"));
    assert!(!text.contains("<img"));
    assert!(!text.contains("https://tracker.test"));
}

#[test]
fn cached_html_escapes_content_and_disables_referrers() {
    let document = safe_html_document("<script>alert('x')</script>");
    assert!(!document.contains("<script>"));
    assert!(document.contains("&lt;script&gt;"));
    assert!(document.contains("no-referrer"));
    assert!(!document.contains("<img"));
}
