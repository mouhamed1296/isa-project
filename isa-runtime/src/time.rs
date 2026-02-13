use crate::{Result, RuntimeError};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct MonotonicClock {
    last_timestamp: u64,
}

impl MonotonicClock {
    pub fn new() -> Self {
        Self {
            last_timestamp: 0,
        }
    }

    pub fn now(&mut self) -> Result<u64> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| RuntimeError::TimeSourceFailed)?
            .as_millis() as u64;

        if timestamp < self.last_timestamp {
            return Err(RuntimeError::TimeSourceFailed);
        }

        self.last_timestamp = timestamp;
        Ok(timestamp)
    }

    pub fn delta(&mut self, previous: u64) -> Result<u64> {
        let current = self.now()?;
        Ok(current.saturating_sub(previous))
    }
}

impl Default for MonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_monotonic_clock() {
        let mut clock = MonotonicClock::new();
        let t1 = clock.now().unwrap();
        thread::sleep(Duration::from_millis(10));
        let t2 = clock.now().unwrap();
        
        assert!(t2 > t1);
    }

    #[test]
    fn test_delta() {
        let mut clock = MonotonicClock::new();
        let t1 = clock.now().unwrap();
        thread::sleep(Duration::from_millis(10));
        let delta = clock.delta(t1).unwrap();
        
        assert!(delta >= 10);
    }
}
