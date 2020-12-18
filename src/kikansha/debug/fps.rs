use std::time::{Duration, SystemTime};

pub struct Counter {
    last_time: SystemTime,
    report_every: u8,
    counter: u16,
}

impl Counter {
    pub fn new(report_every: u8) -> Self {
        Self {
            last_time: SystemTime::now(),
            report_every,
            counter: 0,
        }
    }

    pub fn tick(&mut self) -> Option<f64> {
        let new_ts = SystemTime::now();
        match new_ts.duration_since(self.last_time) {
            Ok(elapsed) => {
                let result = if elapsed >= Duration::from_secs(self.report_every as u64) {
                    let fps = (self.counter as f64) / elapsed.as_secs_f64();
                    self.last_time = new_ts;
                    self.counter = 0;
                    Some(fps)
                } else {
                    None
                };
                self.counter += 1;
                result
            }
            Err(_) => None,
        }
    }
}
