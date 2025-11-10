use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

pub struct RandomState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group_a: wgpu::BindGroup,
    bind_group_b: wgpu::BindGroup,
    pub uniforms_buffer: wgpu::Buffer,
}

#[derive(Copy, Clone, Debug, encase::ShaderType, serde::Deserialize, serde::Serialize)]
pub struct RandomUniforms {
    pub x: u32,
    pub y: u32,
    pub seed: u32,
    pub density: f32,
    pub use_brush: u32,
    pub size: u32,
}
impl Default for RandomUniforms {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            seed: (web_sys::js_sys::Math::random() * 100000.) as u32,
            density: 0.5,
            use_brush: 1,
            size: 10
        }
    }
}

#[wasm_bindgen(typescript_custom_section)]
const RANDOM_CONFIG: &'static str = r#"
    type RandomConfig = {
        x: number,
        y: number,
        seed: number,
        density: number,
        use_brush: number,
        size: number,
    }
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "RandomConfig")]
    #[derive(Debug)]
    pub type RandomConfig;
}


impl RandomState {
    pub fn new(
        device: &wgpu::Device,
        buffer_a: &wgpu::Buffer,
        buffer_b: &wgpu::Buffer,
        globals: &wgpu::Buffer,
    ) -> Self {
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

        let mut uniforms_aligned = encase::UniformBuffer::new(Vec::<u8>::new());
        uniforms_aligned.write(&RandomUniforms::default()).unwrap();

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Random Uniforms Buffer"),
            contents: &uniforms_aligned.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                    resource: buffer_b.as_entire_binding(),
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
            bind_group_a,
            bind_group_b,
            uniforms_buffer,
        }
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        buffer_a: &wgpu::Buffer,
        buffer_b: &wgpu::Buffer,
        globals: &wgpu::Buffer,
    ) {
        self.bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                },
            ],
        });
        self.bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                    resource: buffer_b.as_entire_binding(),
                },
            ],
        });
    }

    pub fn randomize(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
        flip: bool,
    ) {
        queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::cast_slice(&[x, y]));

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Randomness Compute Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);

        let bind_group = if flip {
            &self.bind_group_b
        } else {
            &self.bind_group_a
        };
        pass.set_bind_group(0, bind_group, &[]);

        let workgroups_x = (config.width + 15) / 16;
        let workgroups_y = (config.height + 15) / 16;
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}
