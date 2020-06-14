use super::Vertex;
use crate::rendering::prelude::*;

pub trait DrawableDataSource {
    type VertexBuffer: TypedBufferAccess<Content = [Vertex]> + Send + Sync + 'static;
    type IndexBuffer: TypedBufferAccess<Content = [u32]> + Send + Sync + 'static;

    fn vertex_buffer(&self) -> Arc<Self::VertexBuffer>;
    fn index_buffer(&self) -> Arc<Self::IndexBuffer>;
}
