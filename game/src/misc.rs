#[derive(Default, Debug)]
pub struct Timer {
    pub timer_sec: f32,
    pub timer_start_sec: f32,
}

impl Timer {
    pub fn start(&mut self, timer_sec: f32) {
        self.timer_sec = timer_sec;
        self.timer_start_sec = timer_sec;
    }

    pub fn restart(&mut self) {
        self.timer_sec = self.timer_start_sec;
    }


    pub fn alpha(&self) -> f32 {
        if self.timer_start_sec > 0.0 {
            let elapsed = self.timer_start_sec - self.timer_sec;
            elapsed / self.timer_start_sec
        } else {
            1.0
        }
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if self.timer_sec == 0.0 {
            return false;
        }
        self.timer_sec -= dt;
        if self.timer_sec <= 0.0 {
            self.timer_sec = 0.0;
            return true;
        }

        false
    }

    pub fn done(&self) -> bool {
        self.timer_sec == 0.0
    }
}