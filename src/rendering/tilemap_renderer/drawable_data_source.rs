use anyhow::Result;

use super::Vertex;
use crate::rendering::prelude::*;

pub trait DrawableDataSource {
    type VertexBuffer: TypedBufferAccess<Content = [Vertex]> + Send + Sync + 'static;
    type IndexBuffer: TypedBufferAccess<Content = [u32]> + Send + Sync + 'static;

    fn vertex_buffer(&self) -> &Arc<Self::VertexBuffer>;
    fn index_buffer(&self) -> &Arc<Self::IndexBuffer>;
}

pub struct TileMesh {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl DrawableDataSource for TileMesh {
    type VertexBuffer = CpuAccessibleBuffer<[Vertex]>;
    type IndexBuffer = CpuAccessibleBuffer<[u32]>;

    #[inline]
    fn vertex_buffer(&self) -> &Arc<Self::VertexBuffer> {
        &self.vertex_buffer
    }

    #[inline]
    fn index_buffer(&self) -> &Arc<Self::IndexBuffer> {
        &self.index_buffer
    }
}

impl TileMesh {
    pub fn new(queue: Arc<Queue>) -> Result<Self> {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::all(),
            false,
            [
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    texture_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [32.0, 0.0, 0.0],
                    texture_coords: [1.0, 0.0],
                },
                Vertex {
                    position: [32.0, 32.0, 0.0],
                    texture_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [0.0, 32.0, 0.0],
                    texture_coords: [0.0, 1.0],
                },
            ]
            .iter()
            .cloned(),
        )?;

        let index_buffer = CpuAccessibleBuffer::from_iter(
            queue.device().clone(),
            BufferUsage::all(),
            false,
            [0, 1, 2, 0, 2, 3].iter().cloned(),
        )?;

        Ok(Self {
            vertex_buffer,
            index_buffer,
        })
    }
}
