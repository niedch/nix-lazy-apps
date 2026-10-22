use crate::nix::ansi_escape::strip_ansi_escapes;

#[test]
fn test_strip_ansi_escapes() {
    assert_eq!(
        strip_ansi_escapes("\x1b[31;1merror:\x1b[0m package not found"),
        "error: package not found"
    );
    assert_eq!(strip_ansi_escapes("plain text"), "plain text");
    assert_eq!(strip_ansi_escapes("\x1b[2J\x1b[H"), "");
}
