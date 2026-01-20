use hydrogen_data_structures::indexed_container::IndexedContainer;

use crate::{gpu_vec::GpuVec, pipeline::PipelineBuffers};

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

impl<T> Default for IndexedVertices<T>
where
    T: bytemuck::NoUninit,
{
    fn default() -> Self {
        Self {
            vertices: GpuVec::vertex(vec![]),
            indices: GpuVec::index(vec![]),
        }
    }
}

impl<T> IndexedVertices<T>
where
    T: bytemuck::NoUninit,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_contents(contents: IndexedContainer<T>) -> Self {
        Self {
            vertices: GpuVec::vertex(contents.items),
            indices: GpuVec::index(contents.indices),
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
