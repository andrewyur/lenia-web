use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct ComputeState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pub uniforms: Uniforms<ComputeUniforms>,
    kernel: Storage,
}

#[derive(Copy, Clone, Debug, Default, encase::ShaderType)]
pub struct ComputeUniforms {
    pub height: u32,
    pub width: u32,
    pub time_step: u32,
    pub m: f32,
    pub s: f32,
    pub kernel_size: u32,
    pub kernel_sum: f32,
}

impl ComputeState {
    fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniforms: &Uniforms<ComputeUniforms>,
        grid: &Storage,
        kernel: &Storage,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                uniforms.bind_group_entry(0),
                grid.bind_group_entry(1),
                kernel.bind_group_entry(2),
            ],
        })
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        grid: &Storage,
    ) {
        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, &self.uniforms, grid, &self.kernel);
    }

    pub fn new(
        device: &wgpu::Device,
        grid: &Storage,
        mut uniforms: Uniforms<ComputeUniforms>
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        let kernel_data = generate_kernel(5);

        uniforms.kernel_size = kernel_data.len() as u32;
        uniforms.kernel_sum = kernel_data.iter().map(|r| r.iter().sum::<f32>()).sum();

        let flattened_kernel = kernel_data.into_iter().flatten().collect::<Vec<_>>();
        let kernel = Storage::new(device, "Kernel", &flattened_kernel);


        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute Pipeline Bind Group Layout"),
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                grid.layout_entry(1, wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE, false),
                kernel.layout_entry(2, wgpu::ShaderStages::COMPUTE, true)
            ],
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, &uniforms, grid, &kernel);

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
            bind_group_layout,
            bind_group,
            uniforms,
            kernel,
        }
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroups_x = (self.uniforms.width + 15) / 16;
        let workgroups_y = (self.uniforms.height + 15) / 16;
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}

fn bell(x: f32, m: f32, s: f32) -> f32 {
    (-(((x - m) / s).powi(2)) / 2.0).exp()
}

pub fn generate_kernel(radius: u32) -> Vec<Vec<f32>> {
    let size = (2 * radius + 1) as usize;

    let mut kernel = vec![vec![0f32; size]; size];

    for i in 0..size {
        for j in 0..size {
            let y = (i as i32 - (radius + 1) as i32) as f32;
            let x = (j as i32 - (radius + 1) as i32) as f32;
            
            let d = ((x + 1.0).powi(2) + (y + 1.0).powi(2)).sqrt() / radius as f32;
        
            if d < 1f32 { 
                kernel[i][j] = bell(d, 0.5, 0.15)
            }
        }
    }

    kernel
}