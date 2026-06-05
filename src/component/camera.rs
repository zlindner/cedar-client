use crate::resource::MapRange;

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
        self.top = self.height / 2.0 - y;
        self.bottom = self.top - self.height;
    }

    pub fn center_on_clamped(&mut self, x: f32, y: f32, horizontal: MapRange, vertical: MapRange) {
        let x = clamp_center(x, horizontal, self.width);
        let y = clamp_center(y, vertical, self.height);

        self.center_on(x, y);
    }

    pub fn center_y(&self) -> f32 {
        self.height / 2.0 - self.top
    }

    pub fn center_x(&self) -> f32 {
        self.left + self.width / 2.0
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn visible_top(&self) -> f32 {
        -self.top
    }

    pub fn visible_bottom(&self) -> f32 {
        -self.bottom
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

fn clamp_center(center: f32, bounds: MapRange, viewport_size: f32) -> f32 {
    let min = bounds.min + viewport_size / 2.0;
    let max = bounds.max - viewport_size / 2.0;

    if min > max {
        (bounds.min + bounds.max) / 2.0
    } else {
        center.clamp(min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::Camera;

    #[test]
    fn centering_uses_world_y_down_coordinates() {
        let mut camera = Camera::new(800.0, 600.0);

        camera.center_on(400.0, 350.0);

        assert_eq!(camera.center_y(), 350.0);
        assert_eq!(camera.visible_top(), 50.0);
        assert_eq!(camera.visible_bottom(), 650.0);
    }
}
