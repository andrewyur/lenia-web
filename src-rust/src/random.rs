use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct RandomState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pub uniforms: Uniforms<RandomUniforms>,
}

#[derive(Copy, Clone, Debug, Default, encase::ShaderType)]
pub struct RandomUniforms {
    pub height: u32,
    pub width: u32,
    pub x: u32,
    pub y: u32,
    pub seed: u32,
    pub density: f32,
    pub use_brush: u32,
    pub size: u32,
}

impl RandomState {
    fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        grid: &Storage,
        uniforms: &Uniforms<RandomUniforms>
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Randomness Bind Group"), 
            layout: &bind_group_layout, 
            entries: &[
                uniforms.bind_group_entry(0),
                grid.bind_group_entry(1),
            ]
        })
    }

    pub fn new(
        device: &wgpu::Device,
        grid: &Storage,
        mut uniforms: Uniforms<RandomUniforms>,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Randomness Bind Group Layout"),
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                grid.layout_entry(1, wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE, false),
            ],
        });

        uniforms.use_brush = 1;

        let bind_group = Self::create_bind_group(device, &bind_group_layout, grid, &uniforms);

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
            uniforms,
        }
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        grid: &Storage,
    ) {
       self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, grid, &self.uniforms)
    }

    pub fn run(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        x: u32,
        y: u32,
    ) {
        self.uniforms.x = x;
        self.uniforms.y = y;
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Randomness Compute Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroups_x = (&self.uniforms.width + 15) / 16;
        let workgroups_y = (&self.uniforms.height + 15) / 16;
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}
