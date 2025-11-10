use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;

pub struct ComputeState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group_a: wgpu::BindGroup, // input=buffer_a, output=buffer_b
    bind_group_b: wgpu::BindGroup, // input=buffer_b, output=buffer_a
    pub uniforms_buffer: wgpu::Buffer,
    kernel_buffer: wgpu::Buffer,
    kernel_metadata_buffer: wgpu::Buffer,
}

#[derive(Copy, Clone, Debug, encase::ShaderType, serde::Deserialize, serde::Serialize)]
pub struct ComputeUniforms {
    pub time_step: u32,
    pub m: f32,
    pub s: f32,
}

#[wasm_bindgen(typescript_custom_section)]
const COMPUTE_CONFIG: &'static str = r#"
    type ComputeConfig = {
        time_step: number,
        m: number,
        s: number,
    }
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ComputeConfig")]
    #[derive(Debug)]
    pub type ComputeConfig;
}

impl Default for ComputeUniforms {
    fn default() -> Self {
        Self {
            time_step: 50,
            m: 0.135,
            s: 0.015,
        }
    }
}

#[derive(encase::ShaderType, Debug)]
pub struct KernelMetadata {
    pub size: u32,
    pub sum: f32,
}

impl ComputeState {
    pub fn new(
        device: &wgpu::Device,
        buffer_a: &wgpu::Buffer,
        buffer_b: &wgpu::Buffer,
        globals: &wgpu::Buffer,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Pipeline Bind Group Layout"),
            entries: &[
                // globals
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
                // uniforms
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
                // input
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // output
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // kernel data
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // kernel
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let kernel_metadata = KernelMetadata {
            size: BELL_KERNEL.len() as u32,
            sum: BELL_KERNEL.iter().map(|r| r.iter().sum::<f32>()).sum(),
        };

        let mut kernel_metadata_aligned = encase::UniformBuffer::new(Vec::<u8>::new());
        kernel_metadata_aligned.write(&kernel_metadata).unwrap();

        let kernel_metadata_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Kernel Data Buffer"),
            contents: &kernel_metadata_aligned.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let kernel_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Kernel Buffer"),
            contents: bytemuck::cast_slice(BELL_KERNEL.as_flattened()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let mut uniforms_data = encase::UniformBuffer::new(Vec::<u8>::new());
        uniforms_data.write(&ComputeUniforms::default()).unwrap();

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compute Uniforms Buffer"),
            contents: &uniforms_data.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group (a -> b)"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: kernel_metadata_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        let bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group (b -> a)"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: kernel_metadata_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: kernel_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            pipeline,
            bind_group_a,
            bind_group_b,
            bind_group_layout,
            uniforms_buffer,
            kernel_buffer,
            kernel_metadata_buffer,
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
            label: Some("Compute Bind Group (a -> b)"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffer_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.kernel_metadata_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.kernel_buffer.as_entire_binding(),
                },
            ],
        });

        self.bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group (b -> a)"),
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
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffer_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.kernel_metadata_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.kernel_buffer.as_entire_binding(),
                },
            ],
        });
    }

    pub fn step(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        config: &wgpu::SurfaceConfiguration,
        flip: &mut bool,
    ) {
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Basic Compute Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.pipeline);

            let bind_group = if *flip {
                &self.bind_group_b
            } else {
                &self.bind_group_a
            };
            pass.set_bind_group(0, bind_group, &[]);

            let workgroups_x = (config.width + 15) / 16;
            let workgroups_y = (config.height + 15) / 16;
            pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        *flip = !*flip;
    }
}

pub const BELL_KERNEL: [[f32; 19]; 19] = [
    [
        0., 0., 0., 0., 0., 0.00538165, 0.01140499, 0.01912865, 0.02586927, 0.0285655, 0.02586927,
        0.01912865, 0.01140499, 0.00538165, 0., 0., 0., 0., 0.,
    ],
    [
        0., 0., 0., 0., 0.01266477, 0.03151872, 0.06135352, 0.09615895, 0.12444746, 0.13533528,
        0.12444746, 0.09615895, 0.06135352, 0.03151872, 0.01266477, 0., 0., 0., 0.,
    ],
    [
        0., 0., 0.00482253, 0.01912865, 0.05592624, 0.12444746, 0.21860164, 0.31495946, 0.38551212,
        0.41111229, 0.38551212, 0.31495946, 0.21860164, 0.12444746, 0.05592624, 0.01912865,
        0.00482253, 0., 0.,
    ],
    [
        0., 0., 0.01912865, 0.06724755, 0.17290712, 0.33741597, 0.52286305, 0.67714011, 0.7706448,
        0.8007374, 0.7706448, 0.67714011, 0.52286305, 0.33741597, 0.17290712, 0.06724755,
        0.01912865, 0., 0.,
    ],
    [
        0., 0.01266477, 0.05592624, 0.17290712, 0.38551212, 0.64564743, 0.85775203, 0.9675704,
        0.99782351, 1., 0.99782351, 0.9675704, 0.85775203, 0.64564743, 0.38551212, 0.17290712,
        0.05592624, 0.01266477, 0.,
    ],
    [
        0.00538165, 0.03151872, 0.12444746, 0.33741597, 0.64564743, 0.90857354, 1., 0.93995799,
        0.84292576, 0.8007374, 0.84292576, 0.93995799, 1., 0.90857354, 0.64564743, 0.33741597,
        0.12444746, 0.03151872, 0.00538165,
    ],
    [
        0.01140499, 0.06135352, 0.21860164, 0.52286305, 0.85775203, 1., 0.8803241, 0.64913909,
        0.47213322, 0.41111229, 0.47213322, 0.64913909, 0.8803241, 1., 0.85775203, 0.52286305,
        0.21860164, 0.06135352, 0.01140499,
    ],
    [
        0.01912865, 0.09615895, 0.31495946, 0.67714011, 0.9675704, 0.93995799, 0.64913909,
        0.35065946, 0.1831176, 0.13533528, 0.1831176, 0.35065946, 0.64913909, 0.93995799,
        0.9675704, 0.67714011, 0.31495946, 0.09615895, 0.01912865,
    ],
    [
        0.02586927, 0.12444746, 0.38551212, 0.7706448, 0.99782351, 0.84292576, 0.47213322,
        0.1831176, 0.05742341, 0.0285655, 0.05742341, 0.1831176, 0.47213322, 0.84292576,
        0.99782351, 0.7706448, 0.38551212, 0.12444746, 0.02586927,
    ],
    [
        0.0285655, 0.13533528, 0.41111229, 0.8007374, 1., 0.8007374, 0.41111229, 0.13533528,
        0.0285655, 0.00386592, 0.0285655, 0.13533528, 0.41111229, 0.8007374, 1., 0.8007374,
        0.41111229, 0.13533528, 0.0285655,
    ],
    [
        0.02586927, 0.12444746, 0.38551212, 0.7706448, 0.99782351, 0.84292576, 0.47213322,
        0.1831176, 0.05742341, 0.0285655, 0.05742341, 0.1831176, 0.47213322, 0.84292576,
        0.99782351, 0.7706448, 0.38551212, 0.12444746, 0.02586927,
    ],
    [
        0.01912865, 0.09615895, 0.31495946, 0.67714011, 0.9675704, 0.93995799, 0.64913909,
        0.35065946, 0.1831176, 0.13533528, 0.1831176, 0.35065946, 0.64913909, 0.93995799,
        0.9675704, 0.67714011, 0.31495946, 0.09615895, 0.01912865,
    ],
    [
        0.01140499, 0.06135352, 0.21860164, 0.52286305, 0.85775203, 1., 0.8803241, 0.64913909,
        0.47213322, 0.41111229, 0.47213322, 0.64913909, 0.8803241, 1., 0.85775203, 0.52286305,
        0.21860164, 0.06135352, 0.01140499,
    ],
    [
        0.00538165, 0.03151872, 0.12444746, 0.33741597, 0.64564743, 0.90857354, 1., 0.93995799,
        0.84292576, 0.8007374, 0.84292576, 0.93995799, 1., 0.90857354, 0.64564743, 0.33741597,
        0.12444746, 0.03151872, 0.00538165,
    ],
    [
        0., 0.01266477, 0.05592624, 0.17290712, 0.38551212, 0.64564743, 0.85775203, 0.9675704,
        0.99782351, 1., 0.99782351, 0.9675704, 0.85775203, 0.64564743, 0.38551212, 0.17290712,
        0.05592624, 0.01266477, 0.,
    ],
    [
        0., 0., 0.01912865, 0.06724755, 0.17290712, 0.33741597, 0.52286305, 0.67714011, 0.7706448,
        0.8007374, 0.7706448, 0.67714011, 0.52286305, 0.33741597, 0.17290712, 0.06724755,
        0.01912865, 0., 0.,
    ],
    [
        0., 0., 0.00482253, 0.01912865, 0.05592624, 0.12444746, 0.21860164, 0.31495946, 0.38551212,
        0.41111229, 0.38551212, 0.31495946, 0.21860164, 0.12444746, 0.05592624, 0.01912865,
        0.00482253, 0., 0.,
    ],
    [
        0., 0., 0., 0., 0.01266477, 0.03151872, 0.06135352, 0.09615895, 0.12444746, 0.13533528,
        0.12444746, 0.09615895, 0.06135352, 0.03151872, 0.01266477, 0., 0., 0., 0.,
    ],
    [
        0., 0., 0., 0., 0., 0.00538165, 0.01140499, 0.01912865, 0.02586927, 0.0285655, 0.02586927,
        0.01912865, 0.01140499, 0.00538165, 0., 0., 0., 0., 0.,
    ],
];
