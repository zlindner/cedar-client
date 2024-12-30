pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub scale: f32,
}

impl Transform {
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            scale: 1.0,
        }
    }
}