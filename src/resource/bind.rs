use std::{num::NonZeroU32, marker::PhantomData};

use bytemuck::{Pod, Zeroable};
use repr_trait::C;
use wgpu::util::DeviceExt;


#[derive(Debug)]
pub struct BindingLayoutEntry {
    pub visibility: wgpu::ShaderStages,
    pub ty: wgpu::BindingType,
    pub count: Option<NonZeroU32>,
}

#[derive(Debug)]
pub struct BindingSetLayoutDescriptor {
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindingLayoutEntry {
    pub fn with_binding(self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: self.visibility,
            ty: self.ty,
            count: self.count,
        }
    }
}

pub trait Binding {
    fn get_layout_entry(&self) -> BindingLayoutEntry;
    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a>;
}

pub trait BindingSet {
    fn layout_desc(&self) -> BindingSetLayoutDescriptor;
    fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup;
}

pub trait AsBindingSet<'a> {
    type Set: BindingSet;

    fn as_binding_set(&'a self) -> Self::Set;
}

#[allow(non_snake_case)]
impl<B0> BindingSet for (&B0,)
where
B0: Binding,
{
    fn layout_desc(&self) -> BindingSetLayoutDescriptor {
        let (B0,) = *self;

        BindingSetLayoutDescriptor {
            entries: vec![
                B0.get_layout_entry().with_binding(0),
            ],
        }
    }

    fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let (B0,) = *self;

        let bs_layout = self.layout_desc();

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &bs_layout.entries,
            }
        );
        
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: B0.get_resource(),
                    },
                ],
            }
        );

        bind_group
    }
}

#[allow(non_snake_case)]
impl<B0, B1> BindingSet for (&B0, &B1,)
where
    B0: Binding,
    B1: Binding,
{
    fn layout_desc(&self) -> BindingSetLayoutDescriptor {
        let (B0, B1,) = *self;

        BindingSetLayoutDescriptor {
            entries: vec![
                B0.get_layout_entry().with_binding(0),
                B1.get_layout_entry().with_binding(1),
            ],
        }
    }

    fn into_bind_group(&self, device: &wgpu::Device) -> wgpu::BindGroup {
        let (B0, B1,) = *self;

        let bs_layout = self.layout_desc();

        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &bs_layout.entries,
            }
        );
        
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: B0.get_resource(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: B1.get_resource(),
                    }
                ],
            }
        );

        bind_group
    }
}

pub trait GpuUniform: C + Pod + Zeroable {

}

pub trait UpdateGpuUniform {
    type GU: GpuUniform;

    fn update_uniform(&self, gpu_uniform: &mut Self::GU);
}

pub struct Uniform<H>
where
    H: UpdateGpuUniform,
{
    pub gpu_uniform: H::GU,
    buffer: UniformBuffer<H::GU>,
    _uniform_repr: PhantomData<H>,
}

impl<H> Uniform<H>
where
    H: UpdateGpuUniform,
{
    pub fn new(device: &wgpu::Device, gpu_uniform: H::GU) -> Self {
        let buffer = 
            UniformBuffer::new_init(device, gpu_uniform);
        Self {
            gpu_uniform,
            buffer,
            _uniform_repr: PhantomData,
        }
    }

    pub fn sync_buffer(&self, queue: &wgpu::Queue) {
        self.buffer.update(queue, self.gpu_uniform);
    }
}

impl<H> Uniform<H>
where
    H: UpdateGpuUniform,
    H::GU: Default
{
    pub fn new_default(device: &wgpu::Device) -> Self {
        Self::new(device, H::GU::default())
    }
}

impl<H> Binding for Uniform<H>
where
    H: UpdateGpuUniform
{
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        self.buffer.get_layout_entry()
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        self.buffer.get_resource()
    }
}

pub struct UniformBuffer<T: GpuUniform> {
    buffer: wgpu::Buffer,
    _marker: PhantomData<T>
}

impl<T: GpuUniform> UniformBuffer<T> {
    pub fn new_init(device: &wgpu::Device, init: T, ) -> Self {
        let buffer1 = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[init]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        Self {
            buffer: buffer1,
            _marker: PhantomData,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, val: T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[val]));
    }
}

impl<T: GpuUniform> Binding for UniformBuffer<T> {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        self.buffer.as_entire_binding()
    }
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Binding for wgpu::TextureView {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::TextureView(self)
    }
}

impl Binding for wgpu::Sampler {
    fn get_layout_entry(&self) -> BindingLayoutEntry {
        BindingLayoutEntry {
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn get_resource<'a>(&'a self) -> wgpu::BindingResource<'a> {
        wgpu::BindingResource::Sampler(self)
    }
}

impl<'a> AsBindingSet<'a> for Texture {
    type Set = (&'a wgpu::TextureView, &'a wgpu::Sampler);

    fn as_binding_set(&'a self) -> Self::Set {
        (&self.view, &self.sampler)
    }
}


#[cfg(test)]
mod tests {
    use cgmath::*;

    use super::*;

    pub struct Camera {
        pub view_matrix: Matrix4<f32>,
        pub projection_matrix: Matrix4<f32>,
    }
    impl UpdateGpuUniform for Camera {
        type GU = CameraUniform;
    
        fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
            gpu_uniform.view_proj = (self.projection_matrix * self.view_matrix).into();
        }
    }
    impl Default for Camera {
        fn default() -> Self {
            Self {
                view_matrix: Matrix4::identity(),
                projection_matrix: Matrix4::identity(),
            }
        }
    }
    
    #[repr(C)]
    #[derive(Debug, Clone, Copy, C, Pod, Zeroable)]
    pub struct CameraUniform {
        pub view_proj: [[f32; 4]; 4],
    }
    impl GpuUniform for CameraUniform {}
    impl Default for CameraUniform {
        fn default() -> Self {
            Self {
                view_proj: Matrix4::identity().into(),
            }
        }
    }

    pub struct Transform {
        pub translation: Vector3<f32>,
        pub scale: Vector3<f32>,
        pub rotation: Quaternion<f32>,
    }
    impl UpdateGpuUniform for Transform {
        type GU = ModelUniform;
    
        fn update_uniform(&self, gpu_uniform: &mut Self::GU) {
            gpu_uniform.model = (
                Matrix4::from_translation(self.translation)
                * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z) 
                * Matrix4::from(self.rotation)
            ).into();
        }
    }
    impl Default for Transform {
        fn default() -> Self {
            Self {
                translation: Vector3::zero(),
                scale: Vector3::new(1.0, 1.0, 1.0),
                rotation: Quaternion::zero(),
            }
        }
    }
    
    #[repr(C)]
    #[derive(Debug, Clone, Copy, C, Pod, Zeroable)]
    pub struct ModelUniform {
        pub model: [[f32; 4]; 4],
    }
    impl GpuUniform for ModelUniform {}
    impl Default for ModelUniform {
        fn default() -> Self {
            Self {
                model: Matrix4::identity().into(),
            }
        }
    }

    fn uniform_usage(device: &wgpu::Device, queue: &wgpu::Queue) {
        // Create high level reprs of uniforms
        let camera = Camera::default();
        let transform = Transform::default();

        // Create uniforms
        let mut camera_uniform: Uniform<Camera> = Uniform::new_default(device);
        let mut model_transform_uniform: Uniform<Transform> = Uniform::new_default(device);

        // Update uniforms
        camera.update_uniform(&mut camera_uniform.gpu_uniform);
        transform.update_uniform(&mut model_transform_uniform.gpu_uniform);

        // Sync Gpu buffers with uniform updates
        camera_uniform.sync_buffer(queue);
        model_transform_uniform.sync_buffer(queue);

        // Create BindingSet
        let mvp_binding_set = (
            &camera_uniform,
            &model_transform_uniform,
        );

        // BindingSet into BindGroup
        let mvp_layout_debug = mvp_binding_set.layout_desc();
        let mvp_bind_group = mvp_binding_set.into_bind_group(device);

        dbg!(mvp_layout_debug);
        dbg!(mvp_bind_group);
    }

}