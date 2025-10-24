use anyhow::{anyhow, Result};
use std::time::Duration;

pub fn parse_duration(duration_str: &str) -> Result<Duration> {
    // Parse duration strings like "1hr", "2min", "24hr", "1day", "1week"
    let duration_str = duration_str.to_lowercase();

    // Extract number and unit
    let (number_str, unit) = if let Some(pos) = duration_str.find(char::is_alphabetic) {
        duration_str.split_at(pos)
    } else {
        return Err(anyhow!("Invalid duration format: {}", duration_str));
    };

    let number: u64 = number_str
        .parse()
        .map_err(|_| anyhow!("Invalid number in duration: {}", number_str))?;

    let duration = match unit {
        "s" | "sec" | "secs" | "second" | "seconds" => Duration::from_secs(number),
        "m" | "min" | "mins" | "minute" | "minutes" => Duration::from_secs(number * 60),
        "h" | "hr" | "hrs" | "hour" | "hours" => Duration::from_secs(number * 3600),
        "d" | "day" | "days" => Duration::from_secs(number * 86400),
        "w" | "week" | "weeks" => {
            if number > 1 {
                return Err(anyhow!("Maximum duration is 1 week"));
            }
            Duration::from_secs(number * 604800)
        }
        _ => {
            return Err(anyhow!(
                "Invalid time unit: {}. Supported units: s, m, h, d, w",
                unit
            ))
        }
    };

    // Check maximum duration (1 week)
    let max_duration = Duration::from_secs(604800); // 1 week in seconds
    if duration > max_duration {
        return Err(anyhow!("Duration exceeds maximum of 1 week"));
    }

    if duration.as_secs() == 0 {
        return Err(anyhow!("Duration must be greater than 0"));
    }

    Ok(duration)
}
