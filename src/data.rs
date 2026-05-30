#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new(width: f32, height: f32) -> Self {
        let proj = glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);

        Self {
            proj: proj.to_cols_array_2d(),
        }
    }
}
