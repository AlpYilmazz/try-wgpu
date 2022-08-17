use cgmath::*;


pub struct Camera {
    pub projection_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection_matrix: Matrix4::identity(),
        }
    }
}

pub struct PerspectiveProjection {
    // pub eye: Point3<f32>,
    // pub target: Point3<f32>,
    // pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl PerspectiveProjection {
    pub fn build_projection_matrix(&self) -> Matrix4<f32> {
        cgmath::perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar)
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            aspect: 1.0,
            fovy: std::f32::consts::PI / 4.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
}