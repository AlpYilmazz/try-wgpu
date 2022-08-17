use resource::{buffer::{Vertex, MeshVertex, Indices}, RenderResources, shader::Shader, mesh_static};
use wgpu::include_wgsl;
use winit::{window::Window, event::*};

pub mod texture;
pub mod util;
pub mod resource;
pub mod camera;


pub struct UserState {
    clear_color: wgpu::Color,
    diffuse_blue_noise: texture::Texture,
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_resources: RenderResources,
    diffuse_texture: texture::Texture,
    
    pub user_state: UserState,
}

impl State {
    const VERTICES: &'static [Vertex] = &[
        Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614], }, // A
        Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354], }, // B
        Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397], }, // C
        Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914], }, // D
        Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641], }, // E
    ];

    const INDICES: &'static [u16] = &[
        0, 1, 4,
        1, 2, 4,
        2, 3, 4,
    ];

    const BACKGROUND_VERTICES: &'static [Vertex] = &[
        Vertex { position: [-0.5, 0.5, 0.0], tex_coords: [0.0, 0.0] }, // A
        Vertex { position: [-0.5, -0.5, 0.0], tex_coords: [0.0, 1.0] }, // B
        Vertex { position: [0.5, -0.5, 0.0], tex_coords: [1.0, 1.0] }, // C
        Vertex { position: [0.5, 0.5, 0.0], tex_coords: [1.0, 0.0] }, // E
    ];

    const BACKGROUND_INDICES: &'static [u16] = &[
        0, 1, 2,
        0, 2, 3,
    ];

    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None, // trace_path
        ).await.unwrap();
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    }
                ],
            }
        );
        
        let bytes = include_bytes!("../res/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            bytes,
            "Diffuse Texture",
        ).expect("Texture could not be created");
        
        let diffuse_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                    }
                ],
            }
        );

        let noise_bytes = include_bytes!("../res/noise.png");
        let diffuse_blue_noise = texture::Texture::from_bytes(
            &device,
            &queue,
            noise_bytes,
            "Blue Noise Texture"
        ).expect("Blue Noise texture could not be created");

        let diffuse_blue_noise_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&diffuse_blue_noise.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&diffuse_blue_noise.sampler),
                    }
                ],
            }
        );

        let mut render_resources = RenderResources::empty();
        
        let basic_wgsl = device.create_shader_module(include_wgsl!("../res/basic.wgsl"));
        render_resources.create_render_pipeline(
            &device, 
            &[&texture_bind_group_layout], 
            &Shader::with_final(
                basic_wgsl,
                vec![Vertex::layout()],
                vec![Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            ), 
            wgpu::PrimitiveTopology::TriangleList
        );
        
        render_resources.push_bind_group(diffuse_bind_group);
        render_resources.push_bind_group(diffuse_blue_noise_bind_group);

        let mesh: mesh_static::Mesh<Vertex> = 
            mesh_static::Mesh::with_all(
                wgpu::PrimitiveTopology::TriangleList,
                Self::VERTICES.to_owned(),
                Some(Indices::U16(Self::INDICES.to_owned())),
            );
        render_resources.create_gpu_mesh(
            &device,
            &mesh
        );

        let background_mesh: mesh_static::Mesh<Vertex> = 
            mesh_static::Mesh::with_all(
                wgpu::PrimitiveTopology::TriangleList,
                Self::BACKGROUND_VERTICES.to_owned(),
                Some(Indices::U16(Self::BACKGROUND_INDICES.to_owned())),
            );
        render_resources.create_gpu_mesh(
            &device,
            &background_mesh
        );

        let user_state = UserState {
            clear_color: wgpu::Color::BLACK,
            diffuse_blue_noise,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_resources,
            diffuse_texture,
            // main_render_pipeline,
            // vertex_buffer,
            // index_buffer,
            // diffuse_bind_group,
            user_state,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size; // Copy
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.user_state.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };

                true
            },
            WindowEvent::KeyboardInput {
                input:
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                },
                ..
            } => {
                // self.user_state.render_pipeline_switch.cycle();

                true
            },
            _ => false,
        }
    }

    pub fn update(&mut self) {

    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        {
            let mut render_pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.user_state.clear_color), // Copy
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                }
            );

            self.render_pass_draw(&mut render_pass, 0, 1, 1);
            self.render_pass_draw(&mut render_pass, 0, 0, 0);

        } // drop(render_pass) <- mut borrow encoder <- mut borrow self
            
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_pass_draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_pipeline_id: usize,
        bind_group_id: usize,
        mesh_id: usize,
    ) {
        let render_pipeline = 
            &self.render_resources.render_pipelines[render_pipeline_id];
        let bind_group_0 = 
            &self.render_resources.bind_groups[bind_group_id];
        let mesh_0 = 
            &self.render_resources.meshes[mesh_id];

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(0, bind_group_0, &[]);
        render_pass.set_vertex_buffer(0, mesh_0.vertex_buffer.slice(..));
        match &mesh_0.assembly {
            mesh_static::GpuMeshAssembly::Indexed {
                index_buffer,
                index_count,
                index_format
            } => {
                render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
                render_pass.draw_indexed(0..*index_count as u32, 0, 0..1);
            },
            mesh_static::GpuMeshAssembly::NonIndexed {
                vertex_count
            } => {
                render_pass.draw(0..*vertex_count as u32, 0..1);
            },
        }
    }
}
