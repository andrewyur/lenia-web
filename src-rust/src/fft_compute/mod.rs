pub use crate::{fft_compute::{fft::{FFTState, FFTUniforms}, growth::{GrowthState, GrowthUniforms}, kernel::{KernelState, KernelUniforms}, pad_wrap::{PadWrapState, PadWrapUniforms}, transpose::{TransposeState, TransposeUniforms}}, storage_manager::Storage};

mod pad_wrap;
mod fft;
mod transpose;
mod kernel;
mod growth;


pub struct FFTComputeState {
    fft: FFTState,
    pad_wrap: PadWrapState,
    transpose: TransposeState,
    kernel: KernelState,
    growth: GrowthState,
    fft_buffer: Storage
}

impl FFTComputeState {
    pub fn new(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        grid: &Storage,
        width: u32,
        height: u32,
    ) -> Self {
        let kernel_radius= 0;

        let (fft_size, fft_buffer) = Self::create_fft_buffer(device, width, height, kernel_radius);

        let pad_wrap = PadWrapState::new(device, grid, &fft_buffer, PadWrapUniforms {
            width, height, size: fft_size
        });

        let mut fft = FFTState::new(device, &fft_buffer, FFTUniforms {
            size: fft_size,
            num_stages: fft_size.ilog2(),
        });

        let mut transpose = TransposeState::new(device, &fft_buffer, TransposeUniforms { size: fft_size });

        let kernel = KernelState::new(
            device, 
            &fft_buffer, 
            encoder,
            queue,
            40,
            height,
            width,
            &mut fft,
            &mut transpose, 
            KernelUniforms {
                size: fft_size,
            }
        );

        let growth = GrowthState::new(device, &fft_buffer, grid, GrowthUniforms {
            fft_size,
            time_step: 50,
            m: 0.135,
            s: 0.015,
            height,
            width,
        });

        Self {
            fft,
            pad_wrap,
            transpose,
            kernel,
            growth,
            fft_buffer,
        }
    }

    pub fn run(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        self.pad_wrap.run(encoder, queue);
        self.fft.run_forward(encoder, queue);
        self.transpose.run(encoder, queue);
        self.fft.run_forward(encoder, queue);
        self.kernel.run(encoder, queue);
        
        self.fft.run_inverse(encoder, queue);
        self.transpose.run(encoder, queue);
        self.fft.run_inverse(encoder, queue);
        self.growth.run(encoder, queue);
    }

    pub fn create_fft_buffer(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        kernel_radius: u32
    ) -> (u32, Storage) {
        let max_dim = width.max(height);
        let fft_size = (max_dim + 2 * kernel_radius).next_power_of_two().max(256);

        (fft_size, Storage::new_empty(device, "FFT", (fft_size.pow( 2) * 4 * 2) as u64))
    }

    pub fn handle_resize(
        &mut self, 
        device: &wgpu::Device, 
        encoder: &mut wgpu::CommandEncoder, 
        queue: &wgpu::Queue,
        grid: &Storage, 
        height: u32,
        width: u32
    ) {
        let max_dim = width.max(height);
        let (fft_size, fft_buffer) = Self::create_fft_buffer(device, width, height, 0);

        log::info!("height: {}, width: {}, -> max_dim: {} -> FFT buffer size: {}, {} stages", height, width, max_dim, fft_size, fft_size.ilog2());

        // this should probably be handled better... easy to leave stuff out
        self.pad_wrap.uniforms.height = height;
        self.pad_wrap.uniforms.width = width;
        self.pad_wrap.uniforms.size = fft_size;
        self.fft.uniforms.size = fft_size;
        self.fft.uniforms.num_stages = fft_size.ilog2();
        self.transpose.uniforms.size = fft_size;
        self.kernel.uniforms.size = fft_size;
        self.growth.uniforms.fft_size = fft_size;
        self.growth.uniforms.width = width;
        self.growth.uniforms.height = height;


        self.pad_wrap.recreate_bind_groups(device, grid, &fft_buffer);
        self.fft.recreate_bind_groups(device, &fft_buffer);
        self.transpose.recreate_bind_groups(device, &fft_buffer);
        self.growth.recreate_bind_groups(device, grid, &fft_buffer);

        self.kernel.recreate_bind_groups(
            device,
            &fft_buffer,
            encoder,
            queue,
            fft_size,
            height,
            width,
            &mut self.fft,
            &mut self.transpose,
        );
        
        self.fft_buffer = fft_buffer;
    }
}