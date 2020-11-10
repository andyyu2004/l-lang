use std::time::Instant;

#[derive(Default)]
pub struct Profiler {}

impl Profiler {
    pub fn time<R>(&self, descr: &str, f: impl FnOnce() -> R) -> R {
        let instant = Instant::now();
        let ret = f();
        let elapsed = instant.elapsed();
        info!("[profiler] {}: {}s:{}μs", descr, elapsed.as_secs(), elapsed.subsec_micros());
        ret
    }
}
