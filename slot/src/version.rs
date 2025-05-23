use colored::*;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command};
use update_informer::{registry, Check};

/// Repository name for version checking
pub const REPO_NAME: &str = "cartridge-gg/slot";

/// Checks if a new version is available and returns it if so
pub fn get_latest_version() -> Option<String> {
    let current = env!("CARGO_PKG_VERSION");

    let informer = update_informer::new(registry::GitHub, REPO_NAME, current)
        .interval(std::time::Duration::from_secs(60 * 100));

    let check_result = informer.check_version();

    if check_result.is_err() {
        println!("error checking for new version");
        return None;
    }

    check_result.ok().flatten().map(|v| v.to_string())
}

/// Prints a notification about a new version being available
pub fn notify_new_version(current_version: &str, latest_version: &str) {
    let message = format!(
        "\n{} {}{} → {}",
        "Slot CLI update available:".bold(),
        "v".red().bold(),
        current_version.red().bold(),
        latest_version.green().bold()
    );

    let upgrade_message = format!("To upgrade, run: {}", "`slotup`".cyan().bold());

    println!("{}", message);
    println!("{}", upgrade_message);
    println!("\n");
}

/// Checks if auto-update is disabled via environment variable (SLOT_DISABLE_AUTO_UPDATE)
pub fn is_auto_update_disabled() -> bool {
    env::var("SLOT_DISABLE_AUTO_UPDATE").is_ok()
}

/// Checks if auto-update should be forced without confirmation (SLOT_FORCE_AUTO_UPDATE)
pub fn is_auto_update_forced() -> bool {
    env::var("SLOT_FORCE_AUTO_UPDATE").is_ok()
}

/// Detects if the current process is being run via `cargo run`
pub fn is_running_via_cargo_run() -> bool {
    // Get the current executable path
    if let Ok(current_exe) = env::current_exe() {
        // Check if the path contains "target/debug" or "target/release"
        // which would indicate it's being run via cargo
        let path_str = current_exe.to_string_lossy();
        if path_str.contains("/target/debug/") || path_str.contains("/target/release/") {
            return true;
        }
    }

    false
}

/// Checks if slotup binary is available in PATH
pub fn is_slotup_available() -> bool {
    which::which("slotup").is_ok()
}

/// Runs the slotup command to update the CLI
/// If update is successful, re-executes the current command with the updated version
pub fn run_auto_update() -> bool {
    if !is_slotup_available() {
        return false;
    }

    // Run slotup command
    let update_success = match Command::new("slotup").status() {
        Ok(status) => status.success(),
        Err(_) => false,
    };

    if update_success {
        println!("Update successful! Re-executing command with new version...");

        // Re-execute the current command with the updated version
        re_execute_current_command();

        // This line should never be reached as re_execute_current_command will exit
        // But we return true just in case
        return true;
    }

    false
}

/// Re-executes the current command with all its arguments
/// This function will exit the current process
pub fn re_execute_current_command() {
    // Get the path to the installed slot binary in ~/.slot/bin/slot
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_dir = env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir.clone());
    let slot_bin_path = config_dir.join(".slot").join("bin").join("slot");

    // Get all command line arguments
    let args: Vec<String> = env::args().skip(1).collect();

    println!("Re-executing with binary: {:?}", slot_bin_path);

    // Execute the command with the installed binary path
    let result = Command::new(slot_bin_path).args(args).status();

    // Exit with the same status code as the re-executed command
    match result {
        Ok(status) => exit(status.code().unwrap_or(0)),
        Err(e) => {
            println!("Failed to re-execute command: {}", e);
            exit(1);
        }
    };
}

/// Checks for a new version and runs auto-update if needed
///
/// Behavior is controlled by environment variables:
/// - SLOT_DISABLE_AUTO_UPDATE: Disables auto-update completely
/// - SLOT_FORCE_AUTO_UPDATE: Forces auto-update without confirmation
///
/// Returns true if an update was performed
pub fn check_and_auto_update() -> bool {
    // Skip auto-update if disabled or running via cargo
    if is_running_via_cargo_run() {
        return false;
    }

    if is_auto_update_disabled() {
        // Still check for updates to notify the user, but don't auto-update
        if let Some(version) = get_latest_version() {
            let current = env!("CARGO_PKG_VERSION");
            notify_new_version(current, version.as_str());
        }
        return false;
    }

    // Skip if slotup is not available
    if !is_slotup_available() {
        return false;
    }

    // Check for new version and run auto-update if available
    if let Some(version) = get_latest_version() {
        let current = env!("CARGO_PKG_VERSION");

        // If auto-update is forced, run it without confirmation
        if is_auto_update_forced() {
            println!("New version available: {} → {}", current, version);
            println!("Auto-updating (SLOT_FORCE_AUTO_UPDATE is set)...");
            return run_auto_update();
        }

        // Otherwise, prompt for confirmation using dialoguer
        println!(
            "\n{} {}{} → {}",
            "Slot CLI update available:".bold(),
            "v".red().bold(),
            current.red().bold(),
            version.green().bold()
        );

        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to update now (recommended)?")
            .default(true)
            .show_default(true)
            .wait_for_newline(true)
            .interact()
            .unwrap_or(false);

        if confirmation {
            println!("Updating Slot CLI first...");
            return run_auto_update();
        }

        // User declined the update, just show the notification
        println!(
            "Update skipped. You can update manually by running: {}",
            "`slotup`".cyan().bold()
        );
    }

    false
}
