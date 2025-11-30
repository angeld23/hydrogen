use crate::{gpu_vec::GpuVec, graphics_controller::GraphicsController, pipeline::PipelineBuffers};
use hydrogen_data_structures::indexed_container::IndexedContainer;

pub trait IndexedContainerGraphicsExt {
    fn with_capacity_for_vertices<U>(vertices: &IndexedVertices<U>) -> Self
    where
        U: bytemuck::Pod;
}

impl<T> IndexedContainerGraphicsExt for IndexedContainer<T> {
    fn with_capacity_for_vertices<U>(vertices: &IndexedVertices<U>) -> Self
    where
        U: bytemuck::NoUninit,
    {
        Self::with_capacity(
            vertices.vertices.len() as usize,
            vertices.indices.len() as usize,
        )
    }
}

#[derive(Debug)]
pub struct IndexedVertices<T>
where
    T: bytemuck::NoUninit,
{
    pub vertices: GpuVec<T>,
    pub indices: GpuVec<u32>,
}

impl<T> IndexedVertices<T>
where
    T: bytemuck::NoUninit,
{
    pub fn new(graphics_controller: &GraphicsController) -> Self {
        Self {
            vertices: graphics_controller.vertex_vec(vec![]),
            indices: graphics_controller.index_vec(vec![]),
        }
    }

    pub fn from_contents(
        graphics_controller: &GraphicsController,
        contents: IndexedContainer<T>,
    ) -> Self {
        Self {
            vertices: graphics_controller.vertex_vec(contents.items),
            indices: graphics_controller.index_vec(contents.indices),
        }
    }

    pub fn replace_contents(&mut self, new_contents: IndexedContainer<T>) {
        self.vertices.replace_contents(new_contents.items);
        self.indices.replace_contents(new_contents.indices);
    }

    pub fn as_pipeline_buffers(&'_ self) -> PipelineBuffers<'_, T> {
        PipelineBuffers {
            vertices: &self.vertices,
            instances: None,
            indices: Some(&self.indices),
        }
    }
}
