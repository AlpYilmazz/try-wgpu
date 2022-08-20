
use camera::{Camera, PerspectiveProjection, CameraView, CameraUniform};
use cgmath::*;
use resource::{buffer::{Vertex, MeshVertex, Indices, InstanceRaw, InstanceUnit, self}, RenderResources, shader::Shader, mesh_static, TypedBindGroupLayout, RenderRef};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{window::Window, event::*};

pub mod texture;
pub mod util;
pub mod resource;
pub mod camera;


// NOTE:
// Traits:
//     - MeshVertex:    VertexBuffer
//     - InstanceUnit:  VertexBuffer
//     - Uniform:       BindGroup
// Structs:
//     - Texture:       BindGroup


pub struct UserState {
    clear_color: wgpu::Color,
    diffuse_blue_noise: texture::Texture,
    airboat_ref: std::ops::Range<usize>,
}

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_resources: RenderResources,
    diffuse_texture: texture::Texture,

    camera_uniform: CameraUniform,
    camera_buffer_id: usize,
    camera: Camera,
    camera_view: CameraView,
    camera_projection: PerspectiveProjection,
    camera_controller: CameraController,
    
    instances: Vec<buffer::Instance>,
    instance_buffer: wgpu::Buffer,
    render_refs: Vec<(usize, Vec<RenderRef>)>,
    
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

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = Vector3::new(
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5,
        0.0,
        Self::NUM_INSTANCES_PER_ROW as f32 * 0.5
    );

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
        
        let mut render_resources = RenderResources::empty();
        
        let bytes = include_bytes!("../res/happy-tree.png");
        let diffuse_texture = texture::Texture::from_bytes(
            &device,
            &queue,
            bytes,
            "Diffuse Texture",
        ).expect("Texture could not be created");

        let noise_bytes = include_bytes!("../res/noise.png");
        let diffuse_blue_noise = texture::Texture::from_bytes(
            &device,
            &queue,
            noise_bytes,
            "Blue Noise Texture"
        ).expect("Blue Noise texture could not be created");

        let texture_layout = render_resources.just_create_texture_layout(&device);

        let tree_texture_bind_id = 
            render_resources.create_texture_bind_group(
                &device,
                &texture_layout,
                &diffuse_texture
            );

        let noise_texture_bind_id = 
            render_resources.create_texture_bind_group(
                &device,
                &texture_layout,
                &diffuse_blue_noise
            );

        let camera_controller = CameraController::new(0.2);
        let mut camera = Camera::new();
        let static_camera = Camera::new();
        let camera_view = CameraView::default();
        let camera_projection = PerspectiveProjection::default();

        camera.view_matrix = camera_view.build_view_matrix();
        camera.projection_matrix = camera_projection.build_projection_matrix();

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update_view_proj(&camera);
        
        let mut static_camera_uniform = CameraUniform::default();
        static_camera_uniform.update_view_proj(&static_camera);

        let camera_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[camera_uniform])
        );
        let camera_layout: TypedBindGroupLayout<CameraUniform> = render_resources.just_create_uniform_layout(&device);
        let camera_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &camera_layout,
                camera_buffer_id
            );
        
        let static_camera_buffer_id = render_resources.create_uniform_buffer_init(
            &device,
            bytemuck::cast_slice(&[static_camera_uniform])
        );
        let static_camera_bind_id = 
            render_resources.create_uniform_bind_group(
                &device,
                &camera_layout,
                static_camera_buffer_id
            );

        let basic_wgsl = device.create_shader_module(include_wgsl!("../res/basic.wgsl"));
        let basic_wgsl_pipeline_id = render_resources.create_render_pipeline(
            &device, 
            &[
                &texture_layout,
                &camera_layout,
            ], 
            &Shader::with_final(
                basic_wgsl,
                vec![Vertex::layout(), InstanceRaw::layout()],
                vec![Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            ), 
            wgpu::PrimitiveTopology::TriangleList
        );

        let pentagon_mesh: mesh_static::Mesh<Vertex> = 
            mesh_static::Mesh::with_all(
                wgpu::PrimitiveTopology::TriangleList,
                Self::VERTICES.to_owned(),
                Some(Indices::U16(Self::INDICES.to_owned())),
            );
        let pentagon_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &pentagon_mesh
        );
        
        let background_mesh: mesh_static::Mesh<Vertex> = 
        mesh_static::Mesh::with_all(
            wgpu::PrimitiveTopology::TriangleList,
            Self::BACKGROUND_VERTICES.to_owned(),
            Some(Indices::U16(Self::BACKGROUND_INDICES.to_owned())),
        );
        let background_mesh_id = render_resources.create_gpu_mesh(
            &device,
            &background_mesh
        );
        
        let model: mesh_static::Model<Vertex> = mesh_static::Mesh::load_obj("res/airboat.obj");
        let model_mesh_count = model.meshes.len();
        let mut maxi = 0;
        for mesh in model.meshes {
            maxi = render_resources.create_gpu_mesh(&device, &mesh);
        }
        let airboat_ref = maxi - model_mesh_count + 1 .. maxi+1;

        let instances = (0..Self::NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..Self::NUM_INSTANCES_PER_ROW).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - Self::INSTANCE_DISPLACEMENT;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                buffer::Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(|ins| {
                ins.to_raw()
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let pentagon = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: pentagon_mesh_id,
            bind_groups: vec![
                tree_texture_bind_id,
                camera_bind_id,
            ],
        };

        let background = RenderRef {
            pipeline: basic_wgsl_pipeline_id,
            mesh: background_mesh_id,
            bind_groups: vec![
                noise_texture_bind_id,
                static_camera_bind_id,
            ],
        };

        let user_state = UserState {
            clear_color: wgpu::Color::BLACK,
            diffuse_blue_noise,
            airboat_ref,
        };

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_resources,
            diffuse_texture,
            
            camera_uniform,
            camera_buffer_id,
            camera,
            camera_view,
            camera_projection,
            camera_controller,

            instances,
            instance_buffer,

            render_refs: vec![
                (
                    basic_wgsl_pipeline_id,
                    vec![
                        // background,
                        pentagon,
                    ]
                )
            ],
            
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
        self.camera_controller.process_events(event);
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
        // self.camera_view.eye += (0.0, 0.1, 0.1).into();
        self.camera_controller.update_camera_view(&mut self.camera_view);

        self.camera.view_matrix = self.camera_view.build_view_matrix();
        self.camera_uniform.update_view_proj(&self.camera);
        let camera_buffer = self.render_resources.buffers.get(self.camera_buffer_id).unwrap();
        self.queue.write_buffer(camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
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

            for (pipeline_id, render_refs) in &self.render_refs {
                self.set_pipeline(&mut render_pass, *pipeline_id);
                for render_ref in render_refs {   
                    self.render_pass_draw(&mut render_pass, render_ref);
                }
            }

        } // drop(render_pass) <- mut borrow encoder <- mut borrow self
            
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn set_pipeline<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_pipeline_id: usize,
    ) {
        let render_pipeline = 
            &self.render_resources.render_pipelines[render_pipeline_id];
            
        render_pass.set_pipeline(render_pipeline);
    }

    fn render_pass_draw<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        render_ref: &RenderRef,
    ) {
        let mesh_0 = 
            &self.render_resources.meshes[render_ref.mesh];

        for (index, bind_group_id) in (&render_ref.bind_groups).into_iter().enumerate() {
            let bind_group = &self.render_resources.bind_groups[*bind_group_id];
            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, mesh_0.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        match &mesh_0.assembly {
            mesh_static::GpuMeshAssembly::Indexed {
                index_buffer,
                index_count,
                index_format
            } => {
                render_pass.set_index_buffer(index_buffer.slice(..), *index_format);
                render_pass.draw_indexed(0..*index_count as u32, 0, 0..self.instances.len() as u32);
            },
            mesh_static::GpuMeshAssembly::NonIndexed {
                vertex_count
            } => {
                render_pass.draw(0..*vertex_count as u32, 0..self.instances.len() as u32);
            },
        }
    }
}


struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn update_camera_view(&self, camera_view: &mut CameraView) {
        use cgmath::InnerSpace;
        let forward = camera_view.target - camera_view.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera_view.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera_view.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera_view.up);

        // Redo radius calc in case the fowrard/backward is pressed.
        let forward = camera_view.target - camera_view.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and eye so 
            // that it doesn't change. The eye therefore still 
            // lies on the circle made by the target and eye.
            camera_view.eye = camera_view.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera_view.eye = camera_view.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
