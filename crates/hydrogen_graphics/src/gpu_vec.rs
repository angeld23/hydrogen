use crate::gpu_handle::GpuHandle;
use std::{mem, ops::Range, sync::Arc};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct GpuVec<T>
where
    T: bytemuck::NoUninit,
{
    handle: Arc<GpuHandle>,

    inner_buffer: wgpu::Buffer,
    inner_vec: Vec<T>,
}

impl<T> GpuVec<T>
where
    T: bytemuck::NoUninit,
{
    fn create_buffer(
        handle: &GpuHandle,
        usage: wgpu::BufferUsages,
        inner_vec: &Vec<T>,
    ) -> wgpu::Buffer {
        handle
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: unsafe {
                    // SAFETY:
                    // - contents of the buffer beyond the range of inner_vec are allowed to be undefined,
                    // as long as there is no public way to retrieve a slice of a GpuVec's inner_buffer that goes
                    // beyond the range of inner_vec
                    // - we're still only getting a slice up to inner_vec's capacity, which means it's allocated
                    // (and that's good i think)

                    bytemuck::cast_slice(inner_vec.get_unchecked(..inner_vec.capacity()))
                },
                usage: usage | wgpu::BufferUsages::COPY_DST,
            })
    }

    pub fn new(handle_arc: Arc<GpuHandle>, usage: wgpu::BufferUsages, contents: Vec<T>) -> Self {
        assert!(
            mem::size_of::<T>() > 0,
            "Element type must not be zero-sized"
        );

        let inner_buffer = Self::create_buffer(&handle_arc, usage, &contents);
        Self {
            handle: handle_arc,

            inner_buffer,
            inner_vec: contents,
        }
    }

    #[inline]
    pub fn capacity(&self) -> wgpu::BufferAddress {
        self.inner_buffer.size() / mem::size_of::<T>() as wgpu::BufferAddress
    }

    #[inline]
    pub fn len(&self) -> wgpu::BufferAddress {
        self.inner_vec.len() as wgpu::BufferAddress
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner_vec.is_empty()
    }

    #[inline]
    pub fn usage(&self) -> wgpu::BufferUsages {
        self.inner_buffer.usage()
    }

    /// Returns [None] if empty
    pub fn borrow_buffer(&self) -> Option<wgpu::BufferSlice> {
        if self.is_empty() {
            return None;
        }

        Some(
            self.inner_buffer
                .slice(0..(self.inner_vec.len() * mem::size_of::<T>()) as wgpu::BufferAddress),
        )
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.inner_buffer
    }

    fn recreate_buffer(&mut self) {
        self.inner_buffer =
            Self::create_buffer(&self.handle, self.inner_buffer.usage(), &self.inner_vec);
    }

    fn match_vec_capacity(&mut self) {
        if self.capacity() != self.inner_vec.capacity() as wgpu::BufferAddress {
            self.recreate_buffer();
        }
    }

    fn expand_if_needed(&mut self) -> bool {
        if self.capacity() < self.inner_vec.capacity() as wgpu::BufferAddress {
            self.recreate_buffer();
            return true;
        }

        false
    }

    fn apply_inner_change(&mut self, mut range: Range<usize>) {
        range.end = range.end.min(self.inner_vec.len());
        if range.start >= range.end {
            return;
        }

        self.handle.queue.write_buffer(
            &self.inner_buffer,
            (range.start * mem::size_of::<T>()) as wgpu::BufferAddress,
            bytemuck::cast_slice(&self.inner_vec[range]),
        );
    }

    /// Note: This has to create an entirely new buffer, because fuck you
    pub fn change_usage(&mut self, new_usage: wgpu::BufferUsages) {
        if self.inner_buffer.usage() != new_usage {
            self.inner_buffer = Self::create_buffer(&self.handle, new_usage, &self.inner_vec);
        };
    }

    pub fn clear(&mut self) {
        self.inner_vec.clear();
    }

    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        let old_len = self.inner_vec.len();
        self.inner_vec.extend(iter);

        let difference = self.inner_vec.len() - old_len;
        if difference > 0 && !self.expand_if_needed() {
            self.apply_inner_change((old_len - 1)..self.inner_vec.len());
        };
    }

    pub fn extend_from_slice(&mut self, slice: &[T]) {
        self.extend(slice.iter().copied());
    }

    pub fn push(&mut self, value: T) {
        self.inner_vec.push(value);
        if !self.expand_if_needed() {
            self.apply_inner_change((self.inner_vec.len() - 1)..self.inner_vec.len())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner_vec.pop()
    }

    pub fn replace_contents(&mut self, new_contents: Vec<T>) {
        self.inner_vec = new_contents;
        if !self.expand_if_needed() {
            self.apply_inner_change(0..self.inner_vec.len());
        }
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.inner_vec[index] = value;
        self.apply_inner_change(index..self.inner_vec.len());
    }

    pub fn overwrite_from_start_index(&mut self, start_index: usize, new_contents: &[T]) {
        // note: an index of exactly inner_vex.len() is allowed because
        // we're only doing this check to avoid having to fill in gaps
        if start_index > self.inner_vec.len() {
            panic!(
                "Index {} is out of range (max is {})",
                start_index,
                self.inner_vec.len()
            );
        }

        if new_contents.is_empty() {
            return;
        }

        let required_length = start_index + new_contents.len();
        if required_length > self.inner_vec.capacity() {
            self.inner_vec
                .reserve(required_length - self.inner_vec.len())
        }

        for (i, value) in new_contents.iter().copied().enumerate() {
            let index = start_index + i;
            if index >= self.inner_vec.len() {
                self.inner_vec.push(value);
            } else {
                self.inner_vec[index] = value;
            }
        }

        if !self.expand_if_needed() {
            self.apply_inner_change(start_index..self.inner_vec.len());
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner_vec.shrink_to_fit();
        self.match_vec_capacity();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner_vec.shrink_to(min_capacity);
        self.match_vec_capacity();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.inner_vec.iter()
    }
}

impl<T> Clone for GpuVec<T>
where
    T: bytemuck::NoUninit,
{
    fn clone(&self) -> Self {
        Self::new(
            Arc::clone(&self.handle),
            self.usage(),
            self.inner_vec.clone(),
        )
    }
}

impl<T> PartialEq for GpuVec<T>
where
    T: bytemuck::NoUninit + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner_vec == other.inner_vec
    }
}
