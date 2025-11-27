use anyhow::anyhow;

use crate::{
    Parameters, compute::{ComputeState, ComputeUniforms}, fft_compute::{FFTComputeState}, random::{RandomState, RandomUniforms}, render::{RenderState, RenderUniforms}, storage_manager::Storage, uniforms_manager::Uniforms
};

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render: RenderState,
    compute: ComputeState,
    fft_compute: FFTComputeState,
    random: RandomState,
    grid: Storage,
    encoder: wgpu::CommandEncoder,
}

impl State {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let width = canvas.width();
        let height = canvas.height();

        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        if width == 0 || height == 0 {
            return Err(anyhow!("canvas height or width cannot be 0"));
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let buffer_size = (width * height * 4) as u64;
        let grid = Storage::new_empty(&device, "Grid", buffer_size);

        let render_uniforms = Uniforms::new(&device, "Render", RenderUniforms {
            height, width, ..Default::default()
        });
        let render = RenderState::new(&device, &grid, render_uniforms, &config);

        let compute_uniforms = Uniforms::new(&device, "Compute", ComputeUniforms {
            height, width, ..Default::default()
        });
        let compute = ComputeState::new(&device, &grid, compute_uniforms);

        let mut encoder = device.create_command_encoder(&Default::default());

        let fft_compute= FFTComputeState::new(&device, &mut encoder, &queue, &grid, width, height);

        let random_uniforms = Uniforms::new(&device, "Randomness", RandomUniforms {
            height, width, ..Default::default()
        });
        let random = RandomState::new(&device, &grid, random_uniforms);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render,
            compute,
            fft_compute,
            random,
            grid,
            encoder,
        })
    }

    pub fn clear(&mut self) {
        self.encoder.clear_buffer(self.grid.buffer(), 0, None);
    }

    pub fn parse_parameters(&mut self, parameters: Parameters) {
        self.random.uniforms.density = parameters.random_density;
        self.random.uniforms.size = parameters.random_brush_size;
        self.random.uniforms.seed = parameters.random_seed;

        self.compute.uniforms.m = parameters.compute_m;
        self.compute.uniforms.s = parameters.compute_s;
        self.compute.uniforms.time_step = parameters.compute_time_step;
    }

    pub fn randomize_area(&mut self, x: u32, y: u32) {
        self.random.run(
            &mut self.encoder,
            &self.queue,
            x, 
            y,
        );
    }

    pub fn step(&mut self) {
        self.fft_compute.run(&mut self.encoder, &self.queue);
    }

    pub fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();

        let view = output.texture.create_view(&Default::default());
        
        self.render
            .render_into(&mut self.encoder, &view, &self.queue);
        
        let encoder = std::mem::replace(
            &mut self.encoder,
            self.device.create_command_encoder(&Default::default()),
        );
        
        self.queue.submit(Some(encoder.finish()));

        output.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }

        if self.config.width == width && self.config.height == height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        
        let buffer_size = (width * height * 4) as u64;
        self.grid = Storage::new_empty(&self.device, "Grid", buffer_size);

        self.random.recreate_bind_groups(&self.device, &self.grid);
        self.render.recreate_bind_groups(&self.device, &self.grid);
        // self.compute.recreate_bind_groups(&self.device, &self.grid);

        self.random.uniforms.width = width;
        self.random.uniforms.height = height;
        self.render.uniforms.width = width;
        self.render.uniforms.height = height;
        // self.compute.uniforms.width = width;
        // self.compute.uniforms.height = height;

        self.fft_compute.handle_resize(&self.device, &mut self.encoder, &self.queue, &self.grid, height, width);

        self.render()
    }
}
