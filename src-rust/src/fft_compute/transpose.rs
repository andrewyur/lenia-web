use crate::{storage_manager::Storage, uniforms_manager::Uniforms};

pub struct TransposeState {
    pipeline_1: wgpu::ComputePipeline,
    pipeline_2: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    scratch: Storage,
    pub uniforms: Uniforms<TransposeUniforms>,
}

#[derive(Clone, Copy, Debug, encase::ShaderType)]
pub struct TransposeUniforms {
    pub size: u32
}

impl TransposeState {
    pub fn new(
        device: &wgpu::Device, 
        fft_buffer: &Storage, 
        uniforms: TransposeUniforms
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("transpose.wgsl"));

        let uniforms = Uniforms::new(device, "Transpose", uniforms);

        let scratch_size = (uniforms.size * (uniforms.size + 1))/2;
        let scratch = Storage::new_empty(device, "Transpose Scratch", (scratch_size * 4 * 2) as u64);
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Transpose Bind Group Layout"), 
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                fft_buffer.layout_entry(1, wgpu::ShaderStages::COMPUTE,false),
                scratch.layout_entry(2, wgpu::ShaderStages::COMPUTE, false)
            ] 
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, &uniforms, fft_buffer, &scratch);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {       
            label: Some("Transpose Pipeline Layout"), 
            bind_group_layouts: &[&bind_group_layout], 
            push_constant_ranges: &[] 
        });

        let pipeline_1 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: Some("Transpose Pipeline"), 
            layout: Some(&pipeline_layout), 
            module: &shader, 
            entry_point: Some("copy_upper"), 
            compilation_options: Default::default(), 
            cache:None 
        });

        let pipeline_2 = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: Some("Transpose Pipeline"), 
            layout: Some(&pipeline_layout), 
            module: &shader, 
            entry_point: Some("transpose_lower"), 
            compilation_options: Default::default(), 
            cache:None 
        });

        Self {
            pipeline_1,
            pipeline_2,
            bind_group,
            bind_group_layout,
            uniforms,
            scratch
        }
    }

    fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniforms: &Uniforms<TransposeUniforms>,
        fft_buffer: &Storage,
        scratch: &Storage,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Transpose Bind Group"), 
            layout: bind_group_layout, 
            entries: &[
                uniforms.bind_group_entry(0),
                fft_buffer.bind_group_entry(1),
                scratch.bind_group_entry(2),
            ] 
        })
    }

    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        fft_buffer: &Storage,
    ) {
        let scratch_size = (&self.uniforms.size * (&self.uniforms.size + 1))/2;
        self.scratch = Storage::new_empty(device, "Transpose Scratch", (scratch_size * 4 * 2) as u64);

        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, &self.uniforms, fft_buffer, &self.scratch)
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);
        let groups = (self.uniforms.size + 15) / 16;

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Transpose Pass 1"), 
                timestamp_writes: None 
            });

            pass.set_pipeline(&self.pipeline_1);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(groups, groups, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
                label: Some("Transpose Pass 2"), 
                timestamp_writes: None 
            });

            pass.set_pipeline(&self.pipeline_2);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.dispatch_workgroups(groups, groups, 1);
        }
    }
}