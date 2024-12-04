use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    proj: Perspective3<f32>
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Point3::origin(),
            target: Point3::new(0., 0., 1.),
            up: Vector3::y(),
            proj: Perspective3::new(aspect, 45. * std::f32::consts::PI / 180., 0.1, 100.)
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.proj.set_aspect(aspect);
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_lh(&self.eye, &self.target, &self.up);

        view * self.proj.as_matrix()
    }
}