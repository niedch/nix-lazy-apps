use crate::{gui::loading_view::LoadingDialogApp, nix::tracker::{GlobalProgress, NixProgressSnapshot}};

#[test]
fn test_process_snapshot_normal_progress() {
    let (_tx, rx) = std::sync::mpsc::channel();
    let mut app = LoadingDialogApp {
        header: "Installing...".to_string(),
        detail: "Starting...".to_string(),
        progress: 0.0,
        rx,
        error: None,
        error_sink: std::sync::Arc::new(std::sync::Mutex::new(None)),
    };

    let snapshot = NixProgressSnapshot {
        global: GlobalProgress {
            fraction: Some(0.45),
            ..Default::default()
        },
        individual: None,
        header: "Downloading packages (1/2)...".to_string(),
        detail: "fetching foo".to_string(),
        error: None,
        is_finished: false,
        fraction: Some(0.45),
        builds: (0, 0, 0),
        downloads: (1, 2, 0),
    };

    let should_close = app.process_snapshot(snapshot);
    assert!(!should_close);
    assert_eq!(app.header, "Downloading packages (1/2)...");
    assert_eq!(app.detail, "fetching foo");
    assert_eq!(app.progress, 0.45);
    assert!(app.error.is_none());
}

#[test]
fn test_process_snapshot_normal_finished() {
    let (_tx, rx) = std::sync::mpsc::channel();
    let mut app = LoadingDialogApp {
        header: "Installing...".to_string(),
        detail: "Starting...".to_string(),
        progress: 0.0,
        rx,
        error: None,
        error_sink: std::sync::Arc::new(std::sync::Mutex::new(None)),
    };

    let snapshot = NixProgressSnapshot {
        global: GlobalProgress {
            fraction: Some(1.0),
            ..Default::default()
        },
        individual: None,
        header: "Complete".to_string(),
        detail: "done".to_string(),
        error: None,
        is_finished: true,
        fraction: Some(1.0),
        builds: (0, 0, 0),
        downloads: (0, 0, 0),
    };

    let should_close = app.process_snapshot(snapshot);
    assert!(should_close);
    assert!(app.error.is_none());
}

#[test]
fn test_process_snapshot_error_mode() {
    let (_tx, rx) = std::sync::mpsc::channel();
    let mut app = LoadingDialogApp {
        header: "Installing...".to_string(),
        detail: "Starting...".to_string(),
        progress: 0.2,
        rx,
        error: None,
        error_sink: std::sync::Arc::new(std::sync::Mutex::new(None)),
    };

    let snapshot = NixProgressSnapshot {
        global: GlobalProgress {
            fraction: Some(0.8),
            ..Default::default()
        },
        individual: None,
        header: "Installation failed".to_string(),
        detail: "error: build failed".to_string(),
        error: Some("error: build failed".to_string()),
        is_finished: false,
        fraction: Some(0.8),
        builds: (0, 1, 0),
        downloads: (0, 0, 0),
    };

    let should_close = app.process_snapshot(snapshot);
    assert!(should_close);
    assert_eq!(app.header, "Installation failed");
    assert_eq!(app.error, Some("error: build failed".to_string()));
    assert_eq!(app.progress, 0.2);
}

#[test]
fn test_process_snapshot_error_finished_closes() {
    let (_tx, rx) = std::sync::mpsc::channel();
    let mut app = LoadingDialogApp {
        header: "Installing...".to_string(),
        detail: "Starting...".to_string(),
        progress: 0.0,
        rx,
        error: Some("error: already errored".to_string()),
        error_sink: std::sync::Arc::new(std::sync::Mutex::new(None)),
    };

    let snapshot = NixProgressSnapshot {
        global: GlobalProgress {
            fraction: Some(1.0),
            ..Default::default()
        },
        individual: None,
        header: "Installation failed".to_string(),
        detail: "finished".to_string(),
        error: None,
        is_finished: true,
        fraction: Some(1.0),
        builds: (0, 0, 0),
        downloads: (0, 0, 0),
    };

    let should_close = app.process_snapshot(snapshot);
    assert!(should_close);
    assert_eq!(app.error, Some("error: already errored".to_string()));
    assert_eq!(app.progress, 0.0);
}
