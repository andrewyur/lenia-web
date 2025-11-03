use wgpu::util::DeviceExt;

pub struct RandomState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RandomUniforms {
    pub seed: u32,
    pub density: f32,
    pub _padding: [u32; 2]
}

impl RandomState {
    pub fn new(device: &wgpu::Device, buffer_a: &wgpu::Buffer, globals: &wgpu::Buffer, uniforms: RandomUniforms) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Randomness Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Random Uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Randomness Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer_a.as_entire_binding(),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("random.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Randomness Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Randomness Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            bind_group,
            uniforms_buffer,
        }
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        buffer_a: &wgpu::Buffer,
        globals: &wgpu::Buffer,
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Randomness Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.uniforms_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffer_a.as_entire_binding(),
                }
            ],
        });
    }

    pub fn randomize(
        &self, 
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        config: &wgpu::SurfaceConfiguration,
        uniforms: Option<RandomUniforms>
    ) {
        if let Some(uniforms) = uniforms {
            queue.write_buffer(
                &self.uniforms_buffer,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }

        let mut encoder = device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("Randomness Compute Encoder"),
                });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Randomness Compute Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);

            let workgroups_x = (config.width + 15) / 16;
            let workgroups_y = (config.height + 15) / 16;
            pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
