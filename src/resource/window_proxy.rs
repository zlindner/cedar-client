use winit::dpi::PhysicalSize;

pub struct WindowProxy {
    pub inner_size: PhysicalSize<u32>,
    pub scale_factor: f64,
}

impl WindowProxy {
    pub fn new(inner_size: PhysicalSize<u32>, scale_factor: f64) -> Self {
        Self {
            inner_size,
            scale_factor,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, new_scale_factor: f64) {
        self.inner_size = new_size;
        self.scale_factor = new_scale_factor;
    }
}
