use std::fmt;
use std::sync::Mutex;

type Count = Mutex<u32>;

pub struct RequestStatistics {
    failure: Count,
    blocked: Count,
    success: Count,
}

impl RequestStatistics {
    pub fn new() -> RequestStatistics {
        RequestStatistics {
            failure: Mutex::new(0),
            blocked: Mutex::new(0),
            success: Mutex::new(0),
        }
    }

    pub fn add(&self, outcome: RequestOutcome) {
        match outcome {
            RequestOutcome::Fail => {
                let mut val = self.failure.lock().unwrap();
                *val += 1;
            }
            RequestOutcome::Blocked => {
                let mut val = self.blocked.lock().unwrap();
                *val += 1;
            }
            RequestOutcome::Success => {
                let mut val = self.blocked.lock().unwrap();
                *val += 1;
            }
        }
    }

    pub fn get_total_count(&self) -> u32 {
        *self.failure.lock().unwrap()
            + *self.blocked.lock().unwrap()
            + *self.success.lock().unwrap()
    }
}

impl fmt::Display for RequestStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} requests", self.get_total_count())
    }
}

pub enum RequestOutcome {
    Fail,
    Success,
    Blocked,
}

mod tests {
    use super::*;

    #[test]
    fn add_request_statistic() {
        let stats = RequestStatistics::new();
        stats.add(RequestOutcome::Fail);

        assert_eq!(stats.get_total_count(), 1);
    }

    #[test]
    fn fmt_request_statistic() {
        let stats = RequestStatistics::new();
        stats.add(RequestOutcome::Success);
        stats.add(RequestOutcome::Fail);
        stats.add(RequestOutcome::Blocked);

        assert_eq!(format!("{}", stats), "3 requests");
    }
}
