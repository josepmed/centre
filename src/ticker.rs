use std::time::Duration;

/// Default tick interval in milliseconds
pub const DEFAULT_TICK_MS: u64 = 250;

/// Get tick duration
pub fn tick_duration() -> Duration {
    Duration::from_millis(DEFAULT_TICK_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_duration() {
        let duration = tick_duration();
        assert_eq!(duration, Duration::from_millis(250));
    }
}
