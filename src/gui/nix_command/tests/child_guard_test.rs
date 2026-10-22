use tokio::process::Command;

use crate::gui::nix_command::child_guard::ChildGuard;

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

fn is_process_running(pid: u32) -> bool {
    unsafe { kill(pid as i32, 0) == 0 }
}

fn kill_process(pid: u32) {
    unsafe {
        let _ = kill(pid as i32, 9);
    }
}

#[tokio::test]
async fn test_child_guard_kills_on_drop_when_armed() -> Result<(), Box<dyn std::error::Error>> {
    let child = Command::new("sleep").arg("10").spawn()?;
    let guard = ChildGuard {
        child,
        disarmed: false,
    };
    let pid = guard.child.id().ok_or("expected pid")?;

    drop(guard);

    let mut terminated = false;
    for _ in 0..50 {
        if !is_process_running(pid) {
            terminated = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    assert!(terminated, "process should have been killed on drop");
    Ok(())
}

#[tokio::test]
async fn test_child_guard_does_not_kill_when_disarmed() -> Result<(), Box<dyn std::error::Error>> {
    let child = Command::new("sleep").arg("10").spawn()?;
    let mut guard = ChildGuard {
        child,
        disarmed: false,
    };
    guard.disarmed = true;
    let pid = guard.child.id().ok_or("expected pid")?;

    drop(guard);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let still_running = is_process_running(pid);

    // Clean up child process manually
    kill_process(pid);

    assert!(
        still_running,
        "process should still be running when disarmed"
    );
    Ok(())
}

#[tokio::test]
async fn test_child_guard_already_exited_no_panic() -> Result<(), Box<dyn std::error::Error>> {
    let mut child = Command::new("true").spawn()?;
    let _ = child.wait().await?;
    let guard = ChildGuard {
        child,
        disarmed: false,
    };
    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        drop(guard);
    }));
    assert!(
        res.is_ok(),
        "dropping guard of already exited child should not panic"
    );
    Ok(())
}
