use std::sync::LazyLock;

use regex::Regex;

pub static ANSI_ESCAPE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").expect("valid regex for ANSI_ESCAPE_RE")
});

pub fn strip_ansi_escapes(s: &str) -> String {
    ANSI_ESCAPE_RE.replace_all(s, "").into_owned()
}
