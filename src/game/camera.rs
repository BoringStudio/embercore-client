use once_cell::sync::OnceCell;
use winit::dpi::PhysicalSize;

pub struct Camera {
    view: glm::Mat4,
    scale: u32,

    projection: glm::Mat4,
}

impl Camera {
    pub fn new(size: PhysicalSize<u32>) -> Self {
        let mut camera = Self {
            view: glm::identity(),
            projection: glm::identity(),
            scale: 2,
        };
        camera.update_projection(size);
        camera
    }

    #[inline]
    pub fn set_view(&mut self, view: glm::Mat4) {
        self.view = view;
    }

    #[inline]
    pub fn view(&self) -> &glm::Mat4 {
        &self.view
    }

    #[inline]
    pub fn update_projection(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = (size.width, size.height);
        let factor = 2.0 * self.scale as f32;

        self.projection = glm::ortho(
            -(width as f32 / factor),
            width as f32 / factor,
            -(height as f32 / factor),
            height as f32 / factor,
            -10.0,
            10.0,
        );
        self.projection.m22 *= -1.0;
    }

    #[inline]
    pub fn projection(&self) -> &glm::Mat4 {
        &self.projection
    }
}
