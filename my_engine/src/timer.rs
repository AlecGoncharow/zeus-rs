use std::time;

/// taken from (ggez)[https://github.com/ggez/ggez/blob/master/src/timer.rs]
#[derive(Debug)]
pub struct TimeContext {
    init_instant: time::Instant,
    last_instant: time::Instant,
    residual_update_dt: time::Duration,
    frame_count: usize,
}

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        TimeContext {
            init_instant: time::Instant::now(),
            last_instant: time::Instant::now(),
            residual_update_dt: time::Duration::from_secs(0),
            frame_count: 0,
        }
    }

    /// Update the state of the `TimeContext` to record that
    /// another frame has taken place.  Necessary for the FPS
    /// tracking and [`check_update_time()`](fn.check_update_time.html)
    /// functions to work.
    ///
    /// It's usually not necessary to call this function yourself,
    /// [`event::run()`](../event/fn.run.html) will do it for you.
    pub fn tick(&mut self) {
        let now = time::Instant::now();
        let time_since_last = now - self.last_instant;
        self.last_instant = now;
        self.frame_count += 1;

        self.residual_update_dt += time_since_last;
    }
}
