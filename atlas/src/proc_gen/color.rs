use pantheon::Color;

pub struct ColorGenerator {
    pub spread: f32,
    pub half_spread: f32,
    pub colors: Vec<Color>,
    pub part: f32,
}

impl ColorGenerator {
    pub fn new(colors: Vec<Color>, spread: f32) -> Self {
        Self {
            spread,
            half_spread: spread / 2.,
            part: 1.0 / (colors.len() - 1) as f32,
            colors,
        }
    }

    pub fn generate(&self, heights: &Vec<f32>, amplitude: f32) -> Vec<Color> {
        heights
            .iter()
            .map(|height| self.calc_color(*height, amplitude))
            .collect()
    }

    pub fn calc_color(&self, height: f32, amp: f32) -> Color {
        let mut value = (height + amp) / (amp * 2.);
        value = ((value - self.half_spread) * (1.0 / self.spread)).clamp(0.0, 0.9999);
        let first_color = (value / self.part) as usize;
        let blend = (value - (first_color as f32 * self.part)) / self.part;

        Color::interpolate(
            self.colors[first_color],
            self.colors[first_color + 1],
            blend,
        )
    }
}
