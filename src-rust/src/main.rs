#[cfg(not(target_arch = "wasm32"))]
mod fft_compute;
#[cfg(not(target_arch = "wasm32"))]
mod uniforms_manager;
#[cfg(not(target_arch = "wasm32"))]
mod storage_manager;

#[cfg(target_arch = "wasm32")]
fn main() {
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
        use crate::{fft_compute::{FFTComputeState, FFTState, FFTUniforms, GrowthState, GrowthUniforms, PadWrapState, PadWrapUniforms, TransposeState, TransposeUniforms}, storage_manager::Storage};

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {..Default::default()});
    
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await.unwrap();
    
        let (device, queue) = adapter.request_device(&Default::default()).await.unwrap();
    
        let height: u32 = 13;
        let width: u32 = 25;
        let fft_size: u32 = 32;

        let signal = vec![vec![0f32; height as usize]; width as usize];

        let flattened_signal = signal.into_iter().flatten().collect::<Vec<_>>();
        let input_buffer = Storage::new(&device, "Input", &flattened_signal);
        // let fft_buffer = Storage::new_empty(&device, "FFT buffer", (fft_size * fft_size * 2 * 4).into());

        debug_buffer(&device, &queue, input_buffer.buffer(), |d| display_grid(bytemuck::cast_slice::<_, f32>(&d), height, width)).await;


        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });

        let fft = FFTComputeState::new(&device, &mut encoder, &queue, &input_buffer, width, height);

        queue.submit(Some(encoder.finish()));


        // let pad = PadWrapState::new(&device, &input_buffer, &fft_buffer, PadWrapUniforms { width, height, size: fft_size });

        // let fft = FFTState::new(&device, &fft_buffer, FFTUniforms {
        //     size: fft_size,
        //     num_stages: fft_size.ilog2(),
        // });

        // let growth = GrowthState::new(&device, &fft_buffer, &input_buffer, GrowthUniforms {
        //     time_step: 0,
        //     m: 0.0,
        //     s: 0.0,
        //     fft_size: fft_size,
        //     height, 
        //     width,
        // });

        // let transpose = TransposeState::new(&device, &fft_buffer, TransposeUniforms { size: fft_size });

        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });
       
        // pad.run(&mut encoder, &queue);

        // fft.run_forward(&mut encoder, &queue);
        // transpose.run(&mut encoder, &queue);
        // fft.run_forward(&mut encoder, &queue);

        // fft.run_inverse(&mut encoder, &queue);
        // transpose.run(&mut encoder, &queue);
        // fft.run_inverse(&mut encoder, &queue);

        // growth.run(&mut encoder, &queue);

        fft.run(&mut encoder, &queue);

        queue.submit(Some(encoder.finish()));

        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });
        
        fft.run(&mut encoder, &queue);

        queue.submit(Some(encoder.finish()));
        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });

        fft.run(&mut encoder, &queue);

        queue.submit(Some(encoder.finish()));
        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });

        fft.run(&mut encoder, &queue);

        queue.submit(Some(encoder.finish()));
        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });

        fft.run(&mut encoder, &queue);

        queue.submit(Some(encoder.finish()));

        // debug_buffer(&device, &queue, fft_buffer.buffer(), |d| display_grid(&bytemuck::cast_slice::<_, [f32; 2]>(&d).iter().map(|v| { let mut vc = v.to_owned(); vc[0] /= (fft_size * fft_size) as f32; vc}).collect::<Vec<_>>(), fft_size, fft_size)).await;

        debug_buffer(&device, &queue, input_buffer.buffer(), |d| display_grid(bytemuck::cast_slice::<_, f32>(&d), width, height)).await;

    }

#[cfg(not(target_arch = "wasm32"))]  
pub async fn debug_buffer(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &wgpu::Buffer,
        display: impl FnOnce(&[u8])
    ) {
        let temp_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("temp"),
            size: buffer.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
    
        let mut encoder = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor { label: Some("Debug encoder") });
    
        encoder.copy_buffer_to_buffer(buffer, 0, &temp_buffer, 0, buffer.size());
        queue.submit(Some(encoder.finish()));
    
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        
        temp_buffer.map_async(wgpu::MapMode::Read, .., move |result| {
            tokio::spawn(async move { tx.send(result).await.unwrap()});
        });
    
        println!("wating for poll");
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    
        println!("wating for flag");
        rx.recv().await.unwrap().unwrap();
    
        {
            let output_data = temp_buffer.get_mapped_range(..);
            display(&output_data);
        }
        temp_buffer.unmap();
    }

#[cfg(not(target_arch = "wasm32"))]
fn display_grid<D: std::fmt::Debug>(data: &[D], width: u32, height: u32) {

    let max_len = data
        .iter()
        .map(|v| format!("{v:.3?}").len())
        .max()
        .unwrap_or(0);

    for i in 0..height {
        for j in 0..width {
            let idx = (i * width + j) as usize;
            print!("{:<w$} ", format!("{:.3?}", data[idx]), w = max_len.max(4));
        }
        println!("");
    }
}
