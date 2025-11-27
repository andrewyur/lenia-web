use std::ops::{Deref, DerefMut};
use wgpu::util::DeviceExt;

pub struct Uniforms<T: encase::ShaderType + encase::internal::WriteInto> {
    buffer: wgpu::Buffer,
    data: T,
}

impl<T: encase::ShaderType + encase::internal::WriteInto> Uniforms<T> {
    pub fn new(
        device: &wgpu::Device,
        label: &str,
        data: T,
    ) -> Self {
        let mut buffer_data = encase::UniformBuffer::new(Vec::new());
        buffer_data.write(&data).unwrap();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} Uniform Buffer", label)),
            contents: &buffer_data.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            buffer,
            data,
        }
    }

    pub fn write(
        &self,
        queue: &wgpu::Queue,
    ) {
        let mut encoded = encase::UniformBuffer::new(Vec::new());
        encoded.write(&self.data).unwrap();
        queue.write_buffer(&self.buffer, 0, &encoded.into_inner());
    }

    // pub fn buffer(&self) -> &wgpu::Buffer {
    //     &self.buffer
    // }

    pub fn layout_entry(&self, binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn bind_group_entry(&'_ self, binding: u32) -> wgpu::BindGroupEntry<'_> {
        wgpu::BindGroupEntry {
            binding,
            resource: self.buffer.as_entire_binding(),
        }
    }
}

impl<T: encase::ShaderType + encase::internal::WriteInto> Deref for Uniforms<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: encase::ShaderType + encase::internal::WriteInto> DerefMut for Uniforms<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}