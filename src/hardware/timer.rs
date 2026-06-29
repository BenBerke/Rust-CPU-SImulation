use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct Timer {
    ticks: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Timer {
    pub fn new(ticks_per_second: u64) -> Self {
        let ticks = Arc::new(AtomicU64::new(0));
        let running = Arc::new(AtomicBool::new(true));

        let thread_ticks = Arc::clone(&ticks);
        let thread_running = Arc::clone(&running);

        let thread = thread::spawn(move || {
            let mut last = Instant::now();
            let mut accumulator = 0.0f64;

            while thread_running.load(Ordering::Relaxed) {
                let now = Instant::now();
                let delta = now.duration_since(last);
                last = now;

                let ticks_to_add =
                    delta.as_secs_f64() * ticks_per_second as f64 + accumulator;

                let whole_ticks = ticks_to_add.floor() as u64;
                accumulator = ticks_to_add - whole_ticks as f64;

                if whole_ticks > 0 {
                    thread_ticks.fetch_add(whole_ticks, Ordering::Relaxed);
                }

                thread::sleep(Duration::from_micros(500));
            }
        });

        Self {
            ticks,
            running,
            thread: Some(thread),
        }
    }

    pub fn read_ticks(&self) -> u64 {
        self.ticks.load(Ordering::Relaxed)
    }

    pub fn read_byte(&self, offset: usize) -> u8 {
        self.read_ticks().to_le_bytes()[offset]
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}