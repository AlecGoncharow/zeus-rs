use std::time;

// fps calculation from https://stackoverflow.com/questions/87304/calculating-frames-per-second-in-a-game
pub const MAX_SAMPLES: usize = 32_768;

/// taken from (ggez)[https://github.com/ggez/ggez/blob/master/src/timer.rs]
#[derive(Debug)]
pub struct TimeContext {
    #[allow(dead_code)]
    init_instant: time::Instant,
    last_instant: time::Instant,
    residual_update_dt: time::Duration,
    delta_since_last_instant: time::Duration,
    samples: [f32; MAX_SAMPLES],
    pub sample_sum: f32,
    sample_cursor: usize,
    pub average_tick: f32,
    pub frame_count: usize,
}

impl TimeContext {
    /// Creates a new `TimeContext` and initializes the start to this instant.
    pub fn new() -> TimeContext {
        TimeContext {
            init_instant: time::Instant::now(),
            last_instant: time::Instant::now(),
            residual_update_dt: time::Duration::from_secs(0),
            delta_since_last_instant: time::Duration::from_secs(0),
            samples: [0.; MAX_SAMPLES],
            sample_sum: 1.,
            sample_cursor: 0,
            average_tick: 1.,
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
        self.delta_since_last_instant = time_since_last;

        // FPS stuff
        let dt = self.delta_time();
        // subtract time falling off buffer
        self.sample_sum -= self.samples[self.sample_cursor];
        // add new time
        self.sample_sum += dt;
        // save value to be subtracted when it falls off
        self.samples[self.sample_cursor] = dt;
        self.sample_cursor = (self.sample_cursor + 1) % MAX_SAMPLES;
        self.average_tick = self.sample_sum / (MAX_SAMPLES as f32);
    }

    pub fn delta_time(&self) -> f32 {
        let seconds = self.delta_since_last_instant.as_secs() as f32;
        let nanos = self.delta_since_last_instant.subsec_nanos() as f32;

        seconds + (nanos * 1e-9)
    }
}

impl Default for TimeContext {
    fn default() -> Self {
        Self::new()
    }
}
