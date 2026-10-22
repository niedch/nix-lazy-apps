use std::sync::LazyLock;

use regex::Regex;


#[allow(clippy::expect_used)]
pub static STORE_PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:/nix/store/(?:[0-9a-zA-Z]{32}-)?|\A[0-9a-zA-Z]{32}-)(?P<name>[^/'"\s]+)"#)
        .expect("valid regex for STORE_PATH_RE")
});

#[allow(clippy::expect_used)]
pub static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://(?:[^\s'"?#]+/)*(?P<name>[^/'"\s?#]+)"#)
        .expect("valid regex for URL_RE")
});

/// Extracts a clean package or resource name from a Nix store path or URL.
///
/// Strips `/nix/store/<hash>-` prefixes (and optional `.drv` suffixes),
/// or URL prefixes (e.g. `https://cache.nixos.org/nar/`), returning a clean name.
pub fn extract_package_name(path: &str) -> String {
    let trimmed = path.trim().trim_matches('\'').trim_matches('"');

    let raw_name = STORE_PATH_RE
        .captures(trimmed)
        .or_else(|| URL_RE.captures(trimmed))
        .and_then(|caps| caps.name("name"))
        .map(|m| m.as_str())
        .unwrap_or(trimmed);

    raw_name.strip_suffix(".drv").unwrap_or(raw_name).to_string()
}
