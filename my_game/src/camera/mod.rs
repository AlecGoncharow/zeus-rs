struct Camera {}

struct SmoothFloat {
    pub agility: f64,
    pub actual: f64,
    pub target: f64,
}

impl SmoothFloat {
    pub fn new(init: f64, agility: f64) -> Self {
        Self {
            agility,
            actual: init,
            target: init,
        }
    }

    pub fn update(&mut self, delta: f64) {
        let offset = self.target - self.actual;
        let change = offset * delta * self.agility;
        self.actual += change;
    }
}
