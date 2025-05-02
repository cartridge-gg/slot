use colored::*;
use update_informer::{registry, Check};

/// Checks if a new version of the slot CLI is available and notifies the user
pub fn check_for_new_version() {
    let name = "cartridge-gg/slot";
    let current = env!("CARGO_PKG_VERSION");

    let informer = update_informer::new(registry::GitHub, name, current)
        .interval(std::time::Duration::from_secs(60 * 100));

    let check_result = informer.check_version();

    if check_result.is_err() {
        println!("error checking for new version");
    }

    if let Some(version) = check_result.ok().flatten() {
        notify_new_version(current, version.to_string().as_str());
    }
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
