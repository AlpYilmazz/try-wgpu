use bytemuck::{Pod, Zeroable};


pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl Indices {
    pub fn len(&self) -> usize {
        match self {
            Indices::U16(vec) => vec.len(),
            Indices::U32(vec) => vec.len(),
        }
    }
}

impl Into<wgpu::IndexFormat> for &Indices {
    fn into(self) -> wgpu::IndexFormat {
        match self {
            Indices::U16(_) => wgpu::IndexFormat::Uint16,
            Indices::U32(_) => wgpu::IndexFormat::Uint32,
        }
    }
}

pub trait MeshVertex: Sized + Pod + Zeroable {
    const ATTR_NAMES: &'static [&'static str];
    const ATTRIBUTES: &'static [wgpu::VertexAttribute];

    fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: Self::size() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRIBUTES,
        }
    }
}

pub trait FromRawVertices: MeshVertex {
    fn from_raw(
        positions: &[f32],
        texcoords: &[f32],
        normals: &[f32],
        vertex_color: &[f32],
    ) -> Vec<Self>;
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl MeshVertex for Vertex {
    const ATTR_NAMES: &'static [&'static str] = 
        &[
            "Position",
            "Texture Coordinates"
        ];
    
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = 
        &wgpu::vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
        ];
}

impl FromRawVertices for Vertex {
    fn from_raw(
        positions: &[f32],
        texcoords: &[f32],
        _normals: &[f32],
        _vertex_color: &[f32],
    ) -> Vec<Self> {
        (0..positions.len() / 3).into_iter()
            .map(|i| {
                Vertex {
                    position: [
                        positions[i],
                        positions[i+1],
                        positions[i+2],
                    ],
                    tex_coords: [
                        *texcoords.get(i).unwrap_or(&0.0),
                        *texcoords.get(i+1).unwrap_or(&0.0),
                    ]
                }
            })
            .collect()
    }
}
