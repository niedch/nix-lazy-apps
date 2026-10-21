use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::nix::tracker::{strip_ansi_escapes, NixStateTracker};

fn assert_success_parsing<P: AsRef<Path>>(filepath: P) -> std::io::Result<()> {
    let file = File::open(filepath).expect("File need to exisit");
    let reader = BufReader::new(file);

    let mut tracker = NixStateTracker::new();
    let mut prev_fraction = 0.0f32;
    let mut snapshot = tracker.snapshot();

    for line in reader.lines() {
        snapshot = tracker.process_line(line.expect("Line should exist").as_ref());

        if let Some(curr) = snapshot.global.fraction {
            assert!(
                prev_fraction <= curr,
                "progress decreased: {prev_fraction} > {curr}"
            );
            prev_fraction = curr;
        }
    }

    assert_eq!(snapshot.global.fraction, Some(1.0));
    assert!(snapshot.is_finished);

    Ok(())
}

#[test]
fn test_successful_parsing() {
    for path in [
        "./src/nix/tests/success_pull_firefox.log",
        "./src/nix/tests/success_cache_foot.log",
        "./src/nix/tests/success_pull_r2modman.log",
        "./src/nix/tests/success_pull_brave.log",
    ] {
        assert_success_parsing(path).expect("Success");
    }
}

#[test]
fn test_global_progress_foot() -> std::io::Result<()> {
    let file = File::open("./src/nix/tests/success_cache_foot.log")?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    let mut tracker = NixStateTracker::new();
    let mut prev_fraction = 0.0f32;
    let mut snapshot = tracker.snapshot();

    while reader.read_line(&mut line)? > 0 {
        snapshot = tracker.process_line(&line);
        line.clear();

        if let Some(curr) = snapshot.global.fraction {
            assert!(
                prev_fraction <= curr,
                "progress decreased: {prev_fraction} > {curr}"
            );
            prev_fraction = curr;
        }
    }

    assert!(snapshot.is_finished);

    Ok(())
}

#[test]
fn test_determine_header() {
    let mut tracker = NixStateTracker::new();

    // Default: Evaluating Nix expression...
    let mut snap = tracker.snapshot();
    assert_eq!(snap.header, "Evaluating Nix expression...");

    // Fetching inputs (done/expected)
    tracker.process_line(r#"@nix {"action":"start","id":100,"type":103,"text":"copying paths"}"#);
    snap =
        tracker.process_line(r#"@nix {"action":"result","id":100,"type":105,"fields":[1,5,0,0]}"#);
    assert_eq!(snap.header, "Fetching inputs (1/5)...");

    // Downloading packages (done/expected)
    tracker.process_line(r#"@nix {"action":"start","id":101,"type":100,"fields":["/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-curl-8.0.0"]}"#);
    snap = tracker.process_line(
        r#"@nix {"action":"start","id":102,"parent":101,"type":101,"text":"downloading"}"#,
    );
    assert_eq!(snap.header, "Downloading packages (1/5)...");

    // Stop downloading, clear path progress expected -> Building derivations...
    tracker.process_line(r#"@nix {"action":"stop","id":102}"#);
    tracker.process_line(r#"@nix {"action":"stop","id":101}"#);
    tracker.process_line(r#"@nix {"action":"result","id":100,"type":105,"fields":[0,0,0,0]}"#);
    // Building derivations (done/expected) when expected > 0
    tracker.process_line(r#"@nix {"action":"start","id":200,"type":104,"text":"building"}"#);
    snap =
        tracker.process_line(r#"@nix {"action":"result","id":200,"type":105,"fields":[2,8,0,0]}"#);
    assert_eq!(snap.header, "Building derivations (2/8)...");

    // Building with build name
    tracker.process_line(r#"@nix {"action":"start","id":201,"type":105,"text":"building '/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-hello-2.12.1.drv'"}"#);
    snap = tracker.snapshot();
    assert_eq!(snap.header, "Building hello-2.12.1...");

    // Building with name and phase
    tracker.process_line(r#"@nix {"action":"setPhase","phase":"buildPhase"}"#);
    snap = tracker.snapshot();
    assert_eq!(snap.header, "Building hello-2.12.1 (buildPhase)...");

    // Stop the build activity, but phase remains or set phase with no name
    tracker.process_line(r#"@nix {"action":"stop","id":201}"#);
    tracker.process_line(r#"@nix {"action":"result","id":200,"type":105,"fields":[0,0,1,0]}"#);
    tracker.process_line(r#"@nix {"action":"setPhase","phase":"unpackPhase"}"#);
    snap = tracker.snapshot();
    assert_eq!(snap.header, "Building (unpackPhase)...");

    // Clear phase, running > 0, expected == 0 -> Building derivations...
    let mut tracker2 = NixStateTracker::new();
    tracker2.process_line(r#"@nix {"action":"start","id":300,"type":104,"text":"building"}"#);
    tracker2.process_line(r#"@nix {"action":"result","id":300,"type":105,"fields":[0,0,1,0]}"#);
    snap = tracker2.snapshot();
    assert_eq!(snap.header, "Building derivations...");

    // Root finish -> Complete
    let _ = tracker.process_line(
        r#"@nix {"action":"start","id":1,"parent":0,"type":102,"text":"evaluating"}"#,
    );
    snap = tracker.process_line(r#"@nix {"action":"stop","id":1}"#);
    assert_eq!(snap.header, "Complete");
}

#[test]
fn test_determine_individual_progress() {
    let mut tracker = NixStateTracker::new();

    // 1. Initial state: None
    assert_eq!(tracker.snapshot().individual, None);

    // 2. Build progress with no active build name falls back to "derivation"
    tracker.process_line(r#"@nix {"action":"start","id":200,"type":104,"text":"building"}"#);
    tracker.process_line(r#"@nix {"action":"result","id":200,"type":105,"fields":[0,1,1,0]}"#);
    let snap = tracker.snapshot();
    assert!(snap.individual.is_some());
    if let Some(ind) = &snap.individual {
        assert_eq!(ind.name, "derivation");
        assert_eq!(ind.phase, None);
    }

    // 3. Build progress with build name and phase
    tracker.process_line(r#"@nix {"action":"start","id":201,"type":105,"text":"building '/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-hello-2.12.1.drv'"}"#);
    tracker.process_line(r#"@nix {"action":"setPhase","phase":"buildPhase"}"#);
    let snap = tracker.snapshot();
    assert!(snap.individual.is_some());
    if let Some(ind) = &snap.individual {
        assert_eq!(ind.name, "hello-2.12.1");
        assert_eq!(ind.phase.as_deref(), Some("buildPhase"));
    }

    // 4. Transfer progress takes precedence over build progress
    tracker.process_line(r#"@nix {"action":"start","id":10,"type":100,"fields":["/nix/store/glw7ngr0dwbp3al07rdlqkrbas7w3a02-curl-8.0.0"]}"#);
    tracker.process_line(
        r#"@nix {"action":"start","id":11,"parent":10,"type":101,"text":"downloading"}"#,
    );
    tracker.process_line(r#"@nix {"action":"result","id":11,"type":105,"fields":[500,1000,0,0]}"#);
    let snap = tracker.snapshot();
    assert!(snap.individual.is_some());
    if let Some(ind) = &snap.individual {
        assert_eq!(ind.name, "curl-8.0.0");
        assert_eq!(ind.bytes, Some((500, 1000)));
        assert_eq!(ind.fraction, Some(0.5));
    }

    // 5. Transfer stopped -> falls back to build progress
    tracker.process_line(r#"@nix {"action":"stop","id":11}"#);
    tracker.process_line(r#"@nix {"action":"stop","id":10}"#);
    let snap = tracker.snapshot();
    assert!(snap.individual.is_some());
    if let Some(ind) = &snap.individual {
        assert_eq!(ind.name, "hello-2.12.1");
        assert_eq!(ind.phase.as_deref(), Some("buildPhase"));
    }

    // 6. Stop build derivation and clear running build counter -> None
    tracker.process_line(r#"@nix {"action":"stop","id":201}"#);
    tracker.process_line(r#"@nix {"action":"result","id":200,"type":105,"fields":[1,1,0,0]}"#);
    assert_eq!(tracker.snapshot().individual, None);
}

#[test]
fn test_strip_ansi_escapes() {
    assert_eq!(
        strip_ansi_escapes("\x1b[31;1merror:\x1b[0m package not found"),
        "error: package not found"
    );
    assert_eq!(strip_ansi_escapes("plain text"), "plain text");
    assert_eq!(strip_ansi_escapes("\x1b[2J\x1b[H"), "");
}

#[test]
fn test_error_capturing_msg() {
    let mut tracker = NixStateTracker::new();
    let snap = tracker.process_line(
        r#"@nix {"action":"msg","level":0,"msg":"\u001b[31;1merror:\u001b[0m evaluation aborted"}"#,
    );

    assert_eq!(snap.error, Some("error: evaluation aborted".to_string()));
    assert_eq!(snap.header, "Installation failed");
    assert_eq!(snap.detail, "error: evaluation aborted");
}

#[test]
fn test_error_capturing_raw() {
    let mut tracker = NixStateTracker::new();
    let snap = tracker.process_line("\x1b[31;1merror:\x1b[0m builder failed with exit code 1");

    assert_eq!(
        snap.error,
        Some("error: builder failed with exit code 1".to_string())
    );
    assert_eq!(snap.header, "Installation failed");
    assert_eq!(snap.detail, "error: builder failed with exit code 1");
}

#[test]
fn test_error_capturing_fixture() -> std::io::Result<()> {
    let file = File::open("./src/nix/tests/failiure_error_message.log")?;
    let reader = BufReader::new(file);

    let mut tracker = NixStateTracker::new();
    let mut snapshot = tracker.snapshot();

    for line in reader.lines() {
        snapshot = tracker.process_line(line?.as_str());
    }

    assert!(snapshot.error.as_deref().unwrap_or("").starts_with("error:"));
    assert_eq!(snapshot.header, "Installation failed");

    Ok(())
}
