use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct FFTState {
    pipeline_forward: wgpu::ComputePipeline,
    pipeline_inverse: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    pub uniforms: Uniforms<FFTUniforms>,
}

#[derive(Clone, Copy, Debug, encase::ShaderType)]
pub struct FFTUniforms {
    pub size: u32,       // power of 2, must be >= 2 * WORKGROUP_SIZE
    pub num_stages: u32, // log2 size
}

impl FFTState {
    pub fn new(
        device: &wgpu::Device, 
        fft_buffer: &Storage, 
        uniforms: FFTUniforms
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("fft.wgsl"));

        let uniforms = Uniforms::new(device, "FFT", uniforms);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("FFT Bind Group Layout"),
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                fft_buffer.layout_entry(1, wgpu::ShaderStages::COMPUTE, false),
            ],
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, &uniforms, fft_buffer);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("FFT Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline_inverse = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("FFT Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("fft_inverse"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        let pipeline_forward = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("FFT Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("fft_forward"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Self {
            pipeline_forward,
            pipeline_inverse,
            bind_group_layout,
            bind_group,
            uniforms,
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniforms: &Uniforms<FFTUniforms>,
        fft_buffer: &Storage,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {   
            label: Some("FFT Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                uniforms.bind_group_entry(0),
                fft_buffer.bind_group_entry(1),
            ],
        })
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        fft_buffer: &Storage,
    ) {
        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, &self.uniforms, fft_buffer)
    }

    pub fn run_forward(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("FFT Forward Compute Pass"), timestamp_writes: None });

        pass.set_pipeline(&self.pipeline_forward);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.uniforms.size, 1, 1);
    }

    pub fn run_inverse(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("FFT Inverse Compute Pass"), timestamp_writes: None });

        pass.set_pipeline(&self.pipeline_inverse);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.uniforms.size, 1, 1);
    }
}
