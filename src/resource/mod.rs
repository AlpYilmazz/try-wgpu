use std::ops::Index;

use self::buffer::MeshVertex;


pub mod buffer;
pub mod shader;
pub mod mesh;
pub mod mesh_static;


#[derive(Default)]
pub struct RenderResources {
    pub render_pipelines: Vec<wgpu::RenderPipeline>,
    pub meshes: Vec<mesh_static::GpuMesh>,
    pub bind_groups: Vec<wgpu::BindGroup>,
}

impl RenderResources {
    pub fn empty() -> Self {
        Default::default()
    }

    pub fn create_gpu_mesh<V: MeshVertex>(
        &mut self,
        device: &wgpu::Device,
        mesh: &mesh_static::Mesh<V>,
    ) -> usize {
        self.meshes.push(mesh_static::GpuMesh::from_mesh(mesh, device));
        self.meshes.len() - 1
    }

    pub fn push_bind_group(
        &mut self,
        bind_group: wgpu::BindGroup,
    ) -> usize {
        self.bind_groups.push(bind_group);
        self.bind_groups.len() - 1
    }

    pub fn create_render_pipeline(
        &mut self,
        device: &wgpu::Device,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
        shader: &shader::Shader,
        primitive_topology: wgpu::PrimitiveTopology,
    ) -> usize {
        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts,
                push_constant_ranges: &[],
            }
        );
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: shader::Shader::VERTEX_ENTRY_POINT,
                    buffers: &shader.vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: shader::Shader::FRAGMENT_ENTRY_POINT,
                    targets: &shader.fragment_targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: primitive_topology,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires
                    // Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        self.render_pipelines.push(render_pipeline);

        self.render_pipelines.len() - 1
    }
}


pub struct RenderPipelineIndex(pub usize);
pub struct MeshIndex(pub usize);
pub struct BindGroupIndex(pub usize);
pub struct BindGroupListIndex(pub Vec<usize>);

impl Index<RenderPipelineIndex> for RenderResources {
    type Output = wgpu::RenderPipeline;

    fn index(&self, index: RenderPipelineIndex) -> &Self::Output {
        self.render_pipelines.get(index.0).unwrap()
    }
}