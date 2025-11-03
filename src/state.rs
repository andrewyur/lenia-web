use anyhow::anyhow;
use wgpu::util::DeviceExt;

use crate::{compute::{ComputeState, ComputeUniforms}, random::{RandomState, RandomUniforms}, render::RenderState};

pub struct State {
    surface: wgpu::Surface<'static>,
    canvas: web_sys::HtmlCanvasElement,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render: RenderState,
    compute: ComputeState,
    random: RandomState,
    globals: Globals,
    flip: bool,
    buffer_a: wgpu::Buffer,
    buffer_b: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    // Should be padded to a multiple of 16 bytes for alignment
    height: u32,
    width: u32,
    _padding: [u32; 2],
}

impl State {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });

        let surface = instance.create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter.request_device(&Default::default()).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        if canvas.width() == 0 || canvas.height() == 0 {
            return Err(anyhow!("canvas height or width cannot be 0"));
        }

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps.formats[0],
            width: canvas.width(),
            height: canvas.height(),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let buffer_size = (config.width * config.height * 4 )as u64;

        let buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer A"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer B"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let globals = Globals {
            width: canvas.width(),
            height: canvas.height(),
            _padding: [0; 2],
        };

        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Parameters"),
            contents: bytemuck::cast_slice(&[globals]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render = RenderState::new(&device, &buffer_a, &buffer_b, &globals_buffer, &config);
        let compute = ComputeState::new(&device, &buffer_a, &buffer_b, &globals_buffer, ComputeUniforms {
            time_step: 10,
            _padding: [0; 3]
        });
        let random = RandomState::new(&device, &buffer_a, &globals_buffer, RandomUniforms {
            seed: 12345,
            density: 0.3,
            _padding: [0; 2]
        });

        let created = Self {
            surface,
            device,
            queue,
            config,
            canvas,
            render,
            compute,
            random,
            globals,
            buffer_a,
            buffer_b,
            globals_buffer,
            flip: false,
        };

        created.randomize();

        Ok(created)
    }

    pub fn randomize(&self) {
        self.random.randomize(
            &self.device, 
            &self.queue, 
            &self.config, 
            None,
        );
    }

    pub fn step(&mut self) {
        self.compute.step(
            &self.device, 
            &self.queue, 
            &self.config, 
            &mut self.flip
        );
    }

    pub fn render(&self) {
        self.render.render(
            &self.device,
            &self.queue,
            &self.surface,
            self.flip
        );
    }

    pub fn update(&mut self) {}

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }

        if self.config.width == width && self.config.height == height {
            return;
        }

        let buffer_size = ((width * height * 4 + 3) & !3) as u64;

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.globals = Globals {
            height: height,
            width: width,
            ..self.globals
        };

        self.queue.write_buffer(
            &self.globals_buffer,
            0,
            bytemuck::cast_slice(&[self.globals]),
        );

        self.buffer_a = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer A"),
            size: buffer_size ,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false
        });

        self.random.recreate_bind_groups(&self.device, &self.buffer_a, &self.globals_buffer);
        self.randomize();

        self.buffer_b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer B"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        self.flip = false;

        self.render.recreate_bind_groups(
            &self.device,
            &self.buffer_a,
            &self.buffer_b,
            &self.globals_buffer,
        );
        self.compute.recreate_bind_groups(
            &self.device,
            &self.buffer_a,
            &self.buffer_b,
            &self.globals_buffer,
        );

        _ = self.render()
    }
}
