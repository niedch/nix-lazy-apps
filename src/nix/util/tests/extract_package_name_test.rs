use crate::nix::util::package_name::extract_package_name;

#[test]
fn test_extract_package_name() {
    // 1. Store path with .drv
    assert_eq!(
        extract_package_name("/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-hello-2.12.1.drv"),
        "hello-2.12.1"
    );

    // 2. Quoted store path in build text
    assert_eq!(
        extract_package_name("building '/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-hello-2.12.1.drv'"),
        "hello-2.12.1"
    );

    // 3. Store path without .drv
    assert_eq!(
        extract_package_name("/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-curl-8.0.0"),
        "curl-8.0.0"
    );

    // 4. Store path with sub-path
    assert_eq!(
        extract_package_name("copying '/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-source/pkgs/foo' to the store"),
        "source"
    );

    // 5. Bare store path (32-character hash prefix)
    assert_eq!(
        extract_package_name("glw7ngr0dwbp3al07rdlqkrbas7w3a02-curl-8.0.0"),
        "curl-8.0.0"
    );

    // 6. Bare store path with .drv
    assert_eq!(
        extract_package_name("glw7ngr0dwbp3al07rdlqkrbas7w3a02-hello-2.12.1.drv"),
        "hello-2.12.1"
    );

    // 7. URL NAR file
    assert_eq!(
        extract_package_name("https://cache.nixos.org/nar/0xinnn77aclrzj9pj8wjqq6v2lcfzgbga1p86pq443iarmiz20ff.nar.zst"),
        "0xinnn77aclrzj9pj8wjqq6v2lcfzgbga1p86pq443iarmiz20ff.nar.zst"
    );

    // 8. Quoted URL in text
    assert_eq!(
        extract_package_name("downloading 'https://cache.nixos.org/nar/1zikb8mkwxa27c83823g5xg3rl3zj74ivak6qyqzx83kyi45iakh.nar.zst'"),
        "1zikb8mkwxa27c83823g5xg3rl3zj74ivak6qyqzx83kyi45iakh.nar.zst"
    );

    // 9. URL with query parameters
    assert_eq!(
        extract_package_name("https://example.com/downloads/package-1.0.tar.gz?key=value"),
        "package-1.0.tar.gz"
    );

    // 10. URL with trailing slash
    assert_eq!(
        extract_package_name("https://cache.nixos.org/nar/"),
        "nar"
    );

    // 11. Text with store path and URL -> store path takes precedence
    assert_eq!(
        extract_package_name("copying path '/nix/store/i1k4x43j3857yjirh08dinz0zmqryaj6-firefox-unwrapped-158.0' from 'https://cache.nixos.org'"),
        "firefox-unwrapped-158.0"
    );

    // 12. Fallback when neither matches
    assert_eq!(
        extract_package_name("simple-pkg"),
        "simple-pkg"
    );

    // 13. Fallback strips .drv
    assert_eq!(
        extract_package_name("custom-tool.drv"),
        "custom-tool"
    );

    // 14. Fallback with surrounding quotes
    assert_eq!(
        extract_package_name("'quoted-app.drv'"),
        "quoted-app"
    );
}
