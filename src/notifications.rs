/// Cross-platform notification support
/// Currently only implements macOS notifications

#[cfg(target_os = "macos")]
use std::process::Command;

/// Send a notification when a task is completed
pub fn notify_task_done(task_title: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"display notification "{}" with title "Centre - Task Completed""#,
            task_title.replace('"', "\\\"")
        );

        let _ = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    }

    #[cfg(not(target_os = "macos"))]
    {
        // No-op on other platforms
        let _ = task_title;
    }
}

/// Send a notification when a task estimate is reached
pub fn notify_estimate_reached(task_title: &str) {
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"display notification "⏰ {}" with title "Centre - Estimate Reached""#,
            task_title.replace('"', "\\\"")
        );

        let _ = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    }

    #[cfg(not(target_os = "macos"))]
    {
        // No-op on other platforms
        let _ = task_title;
    }
}
