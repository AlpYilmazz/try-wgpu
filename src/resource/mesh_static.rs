use wgpu::util::DeviceExt;

use super::buffer::{MeshVertex, Indices, FromRawVertices};


pub struct Mesh<V: MeshVertex> {
    primitive_topology: wgpu::PrimitiveTopology,
    vertices: Vec<V>,
    indices: Option<Indices>,
}

impl<V: MeshVertex> Mesh<V> {
    pub fn new(primitive_topology: wgpu::PrimitiveTopology) -> Self {
        Self {
            primitive_topology,
            vertices: Default::default(),
            indices: None,
        }
    }

    pub fn with_all(
        primitive_topology: wgpu::PrimitiveTopology,
        vertices: Vec<V>,
        indices: Option<Indices>,
    ) -> Self {
        Self {
            primitive_topology,
            vertices,
            indices,
        }
    }

    pub fn load_obj(filepath: &str) -> Vec<Self>
    where
        V: FromRawVertices,
    {
        let (models, _) = tobj::load_obj(
            filepath,
            &tobj::GPU_LOAD_OPTIONS
        ).expect("Obj file could not be loaded");

        let meshes: Vec<Mesh<V>> = models
            .into_iter()
            .map(|model| {
                let vertices: Vec<V> = V::from_raw(
                    &model.mesh.positions,
                    &model.mesh.texcoords,
                    &model.mesh.normals,
                    &model.mesh.vertex_color
                );
                
                Self::with_all(
                    wgpu::PrimitiveTopology::TriangleList,
                    vertices,
                    Some(Indices::U32(model.mesh.indices)),
                )
            })
            .collect();
    
        meshes
    }

    pub fn get_vertices(&self) -> &[V] {
        &self.vertices
    }

    pub fn set_vertices(&mut self, vertices: Vec<V>) {
        self.vertices = vertices;
    }

    pub fn push_vertices(&mut self, vertices: impl Iterator<Item = V>) {
        self.vertices.extend(vertices);
    }

    pub fn get_indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }

    pub fn set_indices(&mut self, indices: Indices) {
        self.indices = Some(indices);
    }

    pub fn get_primitive_topology(&self) -> wgpu::PrimitiveTopology {
        self.primitive_topology
    }

    pub fn get_index_buffer_bytes(&self) -> Option<&[u8]> {
        self.indices.as_ref().map(|inds| {
            match inds {
                Indices::U16(ivals) => bytemuck::cast_slice(&ivals[..]),
                Indices::U32(ivals) => bytemuck::cast_slice(&ivals[..]),
            }
        })
    }

    pub fn get_vertex_buffer_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.vertices)
    }

    pub fn get_vertex_buffer_layout(&self) -> wgpu::VertexBufferLayout<'static> { // TODO: lifetime
        V::layout()
    }

    pub fn vertex_size(&self) -> u64 {
        V::size()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}

pub enum GpuMeshAssembly {
    Indexed {
        index_buffer: wgpu::Buffer,
        index_count: usize,
        index_format: wgpu::IndexFormat,
    },
    NonIndexed {
        vertex_count: usize,
    }
}

pub struct GpuMesh {
    pub vertex_buffer_layout: wgpu::VertexBufferLayout<'static>, // TODO: lifetime again
    pub vertex_buffer: wgpu::Buffer,
    pub assembly: GpuMeshAssembly,
    pub primitive_topology: wgpu::PrimitiveTopology,
}

impl GpuMesh {
    pub fn from_mesh<V: MeshVertex>(
        mesh: &Mesh<V>,
        device: &wgpu::Device,
    ) -> GpuMesh {
        GpuMesh {
            vertex_buffer_layout: mesh.get_vertex_buffer_layout(),
            vertex_buffer: device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: &mesh.get_vertex_buffer_bytes(),
                    usage: wgpu::BufferUsages::VERTEX,
                }
            ),
            assembly: match mesh.get_index_buffer_bytes() {
                Some(indices) => GpuMeshAssembly::Indexed {
                    index_buffer: device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: indices,
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    ),
                    index_count: mesh.get_indices().unwrap().len(),
                    index_format: mesh.get_indices().unwrap().into(),
                },
                None => GpuMeshAssembly::NonIndexed { vertex_count: mesh.vertex_count() },
            },
            primitive_topology: mesh.get_primitive_topology(),
        }
    }
}
