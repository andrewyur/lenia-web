use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct PadWrapState {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pub uniforms: Uniforms<PadWrapUniforms>,
}

#[derive(Clone, Copy, Debug, Default, encase::ShaderType)]
pub struct PadWrapUniforms {
    pub width: u32,
    pub height: u32,
    pub size: u32,
}

impl PadWrapState {
    pub fn new(
        device: &wgpu::Device,
        grid: &Storage,
        fft_buffer: &Storage,
        uniforms: PadWrapUniforms,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("pad_wrap.wgsl"));

        let uniforms = Uniforms::new(device, "Pad Wrap", uniforms);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Pad Wrap Bind Group Layout"),
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                grid.layout_entry(1, wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT, true),
                fft_buffer.layout_entry(2, wgpu::ShaderStages::COMPUTE, false),
            ],
        });

        let bind_group = Self::create_bind_groups(
            device,
            &bind_group_layout,
            grid,
            fft_buffer,
            &uniforms,
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pad Wrap Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Pad Wrap Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: None,
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Self {
            pipeline,
            bind_group,
            bind_group_layout,
            uniforms,
        }
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        grid: &Storage,
        fft_buffer: &Storage,
        uniforms: &Uniforms<PadWrapUniforms>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pad Wrap Bind Group"),
            layout: bind_group_layout,
            entries: &[
                uniforms.bind_group_entry(0),
                grid.bind_group_entry(1),
                fft_buffer.bind_group_entry(2),
            ],
        })
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        grid: &Storage,
        fft_buffer: &Storage,
    ) {
        self.bind_group = Self::create_bind_groups(
            device, 
            &self.bind_group_layout, 
            grid,  
            fft_buffer, 
            &self.uniforms
        );
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Pad + Wrap Compute Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.bind_group, &[]);

        // let workgroups_x = (self.uniforms.width + 15) / 16;
        // let workgroups_y = (self.uniforms.height + 15) / 16;
        // pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);

        let groups = (self.uniforms.size + 15) / 16;

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(groups, groups, 1);
    }
}
