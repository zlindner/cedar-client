#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Colour {
    pub red: u8,
    pub blue: u8,
    pub green: u8,
    pub alpha: u8,
}

impl Colour {
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: 255,
        }
    }

    pub fn white() -> Self {
        Self::rgb(255, 255, 255)
    }
}
