use crate::{compute::generate_kernel, fft_compute::{FFTState, TransposeState}, storage_manager::Storage, uniforms_manager::Uniforms};

pub struct KernelState {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    kernel_radius: u32,
    pub uniforms: Uniforms<KernelUniforms>
}

#[derive(Clone, Copy, Debug, encase::ShaderType)]
pub struct KernelUniforms {
    pub size: u32,
}

impl KernelState {
    pub fn new(
        device: &wgpu::Device, 
        fft_buffer: &Storage, 
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        kernel_radius: u32,
        height: u32,
        width: u32,
        fft: &mut FFTState,
        transpose: &mut TransposeState,
        uniforms: KernelUniforms,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("kernel.wgsl"));

        let kernel = Self::create_kernel_buffer(device, encoder, queue, uniforms.size, kernel_radius, height, width, fft_buffer, fft, transpose);

        let uniforms = Uniforms::new(device, "Kernel", uniforms);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: Some("Kernel Bind Group Layout"), 
            entries: &[
                uniforms.layout_entry(0, wgpu::ShaderStages::COMPUTE),
                fft_buffer.layout_entry(1, wgpu::ShaderStages::COMPUTE,  false),
                kernel.layout_entry(2, wgpu::ShaderStages::COMPUTE, true),
            ]
        });

        let bind_group = Self::create_bind_group(device, &bind_group_layout, &uniforms, fft_buffer, &kernel);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
            label: Some("Kernel Pipeline Layout"), 
            bind_group_layouts: &[&bind_group_layout], 
            push_constant_ranges: &[]
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor { 
            label: Some("Kernel Pipeline"), 
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
            uniforms,
            kernel_radius,
        }
    }

    fn create_kernel_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        fft_size: u32,
        kernel_radius: u32,
        height: u32,
        width: u32,
        fft_buffer: &Storage,
        fft: &mut FFTState,
        transpose: &mut TransposeState,
    ) -> Storage {
        let kernel_data = generate_kernel(kernel_radius);
        let mut padded_kernel = vec![vec![[0f32; 2]; fft_size as usize]; fft_size as usize];

        let kernel_sum: f32 = kernel_data.iter().map(|r| r.iter().sum::<f32>()).sum();

        let kernel_size: usize = (kernel_radius * 2 + 1) as usize;
        
        for i in 0..kernel_size {
            for j in 0..kernel_size {
                let pi = (i as i32 - kernel_radius as i32).rem_euclid(fft_size as i32) as usize; 
                let pj = (j as i32 - kernel_radius as i32).rem_euclid(fft_size as i32) as usize; 
                padded_kernel[pi][pj]=[kernel_data[i][j] / kernel_sum, 0.0];
            }
        }

        let flattened_kernel = padded_kernel.into_iter().flatten().collect::<Vec<_>>();
        let kernel_buffer = Storage::new(device, "Kernel", &flattened_kernel);

        // run fft on the kernel
        fft.recreate_bind_groups(device, &kernel_buffer);
        transpose.recreate_bind_groups(device, &kernel_buffer);

        fft.run_forward(encoder, queue);
        transpose.run(encoder, queue);
        fft.run_forward(encoder, queue);

        // reset bind groups
        fft.recreate_bind_groups(device, &fft_buffer);
        transpose.recreate_bind_groups(device, &fft_buffer);

        kernel_buffer
    }


    fn create_bind_group(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniforms: &Uniforms<KernelUniforms>,
        fft_buffer: &Storage,
        kernel: &Storage,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor { 
            label: Some("Kernel Bind Group"), 
            layout: &bind_group_layout, 
            entries: &[
                uniforms.bind_group_entry(0),
                fft_buffer.bind_group_entry(1),
                kernel.bind_group_entry(2),
            ] 
        })
    }
    
    pub fn recreate_bind_groups(
        &mut self,
        device: &wgpu::Device,
        fft_buffer: &Storage,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        fft_size: u32,
        height: u32,
        width: u32,
        fft: &mut FFTState,
        transpose: &mut TransposeState
    ) {
        let kernel = Self::create_kernel_buffer(device, encoder, queue, fft_size, self.kernel_radius, height, width, fft_buffer, fft, transpose);
        self.bind_group = Self::create_bind_group(device, &self.bind_group_layout, &self.uniforms, fft_buffer, &kernel);
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.uniforms.write(queue);

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("Kernel Compute Pass"), timestamp_writes: None });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);

        let groups = (self.uniforms.size + 15) / 16;
        pass.dispatch_workgroups(groups, groups, 1);
    }
}