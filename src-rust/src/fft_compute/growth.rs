use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct GrowthState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pub uniforms: Uniforms<GrowthUniforms>,
}

#[derive(Clone, Copy, Debug, encase::ShaderType)]
pub struct GrowthUniforms {
    pub time_step: u32,
    pub m: f32,
    pub s: f32,
    pub fft_size: u32,
    pub height: u32,
    pub width: u32,
}

impl GrowthState {
    fn create_bind_group(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, grid: &Storage, fft_buffer: &Storage, uniforms: &Uniforms<GrowthUniforms>) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Growth Bind Group"), 
            layout, 
            entries: &[
                uniforms.bind_group_entry(0),
                fft_buffer.bind_group_entry(1),
                grid.bind_group_entry(2),
            ] 
        })
    }

    pub fn new(
        device: &wgpu::Device, 
        fft_buffer: &Storage, 
        grid: &Storage,
        uniforms: GrowthUniforms
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("growth.wgsl"));

        let uniforms = Uniforms::new(device, "Growth", uniforms);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Growth Bind Group Layout"), 
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                fft_buffer.layout_entry(1, wgpu::ShaderStages::COMPUTE, true),
                grid.layout_entry(2, wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, false)
            ] 
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, grid, fft_buffer, &uniforms);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Growth Pipeline Layout"), 
            bind_group_layouts: &[&bind_group_layout], 
            push_constant_ranges: &[] 
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: Some("Growth Pipeline"), 
            layout: Some(&pipeline_layout), 
            module: &shader, 
            entry_point: None, 
            compilation_options: Default::default(), 
            cache: None 
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
            uniforms
        }
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device
        , grid: &Storage,
        fft_buffer: &Storage
    ) {
        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, grid, fft_buffer, &self.uniforms);
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
            label: Some("Growth Pass"), 
            timestamp_writes: None 
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let workgroups_x = (self.uniforms.width + 15) / 16;
        let workgroups_y = (self.uniforms.height + 15) / 16;
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
}