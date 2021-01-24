use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
pub fn timed<T>(tag: &str, body: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = body();
    let end = Instant::now();
    log::trace!("run of {} {:?}", tag, (end - start));
    result
}

pub struct Tracer {
    last_time: SystemTime,
    report_every: u8,
    counter: Duration,
    tag: String,
}

impl Tracer {
    pub fn new(tag: String, report_every: u8) -> Self {
        log::trace!("insance of {}", std::any::type_name::<Self>());
        Self {
            last_time: SystemTime::now(),
            report_every,
            //Just because!
            counter: Duration::from_nanos(10),
            tag,
        }
    }

    pub fn run<T>(&mut self, body: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let result = body();
        let end = Instant::now();
        let new_ts = SystemTime::now();
        let dur = end - start;
        self.counter = (self.counter + dur) / 2;
        match new_ts.duration_since(self.last_time) {
            Ok(elapsed) => {
                let _result = if elapsed >= Duration::from_secs(self.report_every as u64) {
                    // let fps = (self.counter as f64) / elapsed.as_secs_f64();
                    self.last_time = new_ts;
                    self.counter = dur;
                    log::trace!("run of {} {:?}", self.tag, self.counter);
                };
            }
            Err(_) => (),
        }
        result
    }
}
