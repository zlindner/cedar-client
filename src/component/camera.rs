#[derive(Clone, Copy)]
pub struct Camera {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
    pub dpi: f64,
    width: f32,
    height: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        let mut camera = Self {
            left: 0.,
            right: 0.,
            top: 0.,
            bottom: 0.,
            near: 0.0,
            far: 100.,
            dpi: 1.0,
            width,
            height,
        };

        camera.resize(width, height);
        camera
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.right = self.left + width;
        self.bottom = self.top - height;
    }

    pub fn center_on(&mut self, x: f32, y: f32) {
        self.left = x - self.width / 2.0;
        self.right = self.left + self.width;
        self.top = y - self.height / 2.0;
        self.bottom = self.top - self.height;
    }

    pub fn screen_space(&self) -> Self {
        let mut camera = *self;
        camera.left = 0.0;
        camera.right = self.width;
        camera.top = 0.0;
        camera.bottom = -self.height;
        camera
    }
}
