#[derive(Copy, Clone)]
pub struct LEDColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct LEDStrip<const NLED: usize> {
    pub colors: [LEDColor; NLED],
}

impl LEDColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn encode(&self) -> u32 {
        ((self.g as u32) << 24) + ((self.r as u32) << 16) + ((self.b as u32) << 8)
    }
}

impl<const NLED: usize> LEDStrip<NLED> {
    pub fn new() -> Self {
        LEDStrip {
            colors: [LEDColor::new(0, 0, 0); NLED],
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.colors.iter().map(|col| col.encode())
    }
}
