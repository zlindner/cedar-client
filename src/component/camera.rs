pub struct Camera {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
    pub dpi: f64,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            left: 0.,
            right: width,
            top: 0.,
            bottom: -1. * height,
            near: 0.0,
            far: 100.,
            dpi: 1.0,
        }
    }
}
