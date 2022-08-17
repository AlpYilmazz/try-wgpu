use super::buffer::MeshVertex;


pub struct Shader {
    pub module: wgpu::ShaderModule,
    pub vertex_buffers: Vec<wgpu::VertexBufferLayout<'static>>, // TODO: lifetime again
    pub fragment_targets: Vec<Option<wgpu::ColorTargetState>>,
}

impl Shader {
    pub const VERTEX_ENTRY_POINT: &'static str = "vs_main";
    pub const FRAGMENT_ENTRY_POINT: &'static str = "fs_main";

    pub fn with(module: wgpu::ShaderModule) -> Self {
        Self {
            module,
            vertex_buffers: Vec::new(),
            fragment_targets: Vec::new(),
        }
    }

    pub fn with_final(
        module: wgpu::ShaderModule,
        vertex_buffers: Vec<wgpu::VertexBufferLayout<'static>>, // TODO: lifetime
        fragment_targets: Vec<Option<wgpu::ColorTargetState>>,
    ) -> Self {
        Self {
            module,
            vertex_buffers,
            fragment_targets,
        }
    }

    pub fn add_vertex<V: MeshVertex>(&mut self) {
        self.vertex_buffers.push(V::layout());
    }

    pub fn add_fragment_target(&mut self, target: wgpu::ColorTargetState) {
        self.fragment_targets.push(Some(target));
    }
}