use anyhow::anyhow;
use wgpu::util::DeviceExt;

use crate::{
    compute::{ComputeState, ComputeUniforms},
    random::{RandomState, RandomUniforms},
    render::RenderState,
};

pub struct State {
    surface: wgpu::Surface<'static>,
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
    encoder: wgpu::CommandEncoder,
}

#[derive(Copy, Clone, Debug, encase::ShaderType)]
struct Globals {
    height: u32,
    width: u32,
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

        let buffer_size = (config.width * config.height * 4) as u64;

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
        };

        let mut globals_data = encase::UniformBuffer::new(Vec::<u8>::new());
        globals_data.write(&globals).unwrap();
        
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Globals"),
            contents: &globals_data.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render = RenderState::new(&device, &buffer_a, &buffer_b, &globals_buffer, &config);
        let compute = ComputeState::new(
            &device,
            &buffer_a,
            &buffer_b,
            &globals_buffer
        );
        let random = RandomState::new(
            &device,
            &buffer_a,
            &buffer_b,
            &globals_buffer
        );

        let encoder = device.create_command_encoder(&Default::default());

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render,
            compute,
            random,
            globals,
            buffer_a,
            buffer_b,
            globals_buffer,
            flip: false,
            encoder,
        })
    }

    // pub fn randomize_full(&mut self) {
    //     self.random.randomize(
    //         &mut self.encoder,
    //         &self.queue,
    //         &self.config,
    //         Some(RandomUniforms {
    //             seed: (Math::random() * 100000.) as u32,
    //             ..Default::default()
    //         }),
    //         self.flip,
    //     );
    // }

    pub fn clear(&mut self) {
        let buffer = if self.flip {
            &self.buffer_b
        } else {
            &self.buffer_a
        };

        self.encoder.clear_buffer(buffer, 0, None);
    }

    pub fn write_random_uniforms(&self, uniforms: RandomUniforms) {
        let mut uniforms_data = encase::UniformBuffer::new(Vec::<u8>::new());
        uniforms_data.write(&uniforms).unwrap();
        self.queue.write_buffer(&self.random.uniforms_buffer, 0, &uniforms_data.into_inner());
    }
    pub fn write_compute_uniforms(&self, uniforms: ComputeUniforms) {
        let mut uniforms_data = encase::UniformBuffer::new(Vec::<u8>::new());
        uniforms_data.write(&uniforms).unwrap();
        self.queue.write_buffer(&self.compute.uniforms_buffer, 0, &uniforms_data.into_inner());
    }

    pub fn randomize_area(&mut self, x: u32, y: u32) {
        self.random.randomize(
            &mut self.encoder,
            &self.config,
            &self.queue,
            x, 
            y,
            self.flip,
        );
    }

    pub fn step(&mut self) {
        self.compute
            .step(&mut self.encoder, &self.config, &mut self.flip);
    }

    pub fn render(&mut self) {
        self.render
            .render(&mut self.encoder, &self.surface, self.flip);

        let encoder = std::mem::replace(
            &mut self.encoder,
            self.device.create_command_encoder(&Default::default()),
        );

        self.queue.submit(Some(encoder.finish()));
        let output = self.surface.get_current_texture().unwrap();
        output.present();
    }

    pub fn update(&mut self) {}

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return;
        }

        if self.config.width == width && self.config.height == height {
            return;
        }

        let buffer_size = (width * height * 4) as u64;

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        self.globals = Globals {
            height,
            width,
            ..self.globals
        };

        self.queue.write_buffer( &self.globals_buffer, 0, bytemuck::cast_slice(&[height, width]) );

        self.buffer_a = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer A"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        // self.random.recreate_bind_groups(&self.device, &self.buffer_a, &self.buffer_b, &self.globals_buffer);
        // self.randomize_full();

        self.buffer_b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer B"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        self.flip = false;

        self.random.recreate_bind_groups(
            &self.device,
            &self.buffer_a,
            &self.buffer_b,
            &self.globals_buffer,
        );

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

        self.render()
    }
}
