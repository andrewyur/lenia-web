use anyhow::{anyhow};
use rand::{Rng, SeedableRng, rngs::StdRng};
use wgpu::{ComputePassDescriptor, Instance, util::DeviceExt};

use crate::{compute::ComputeState, render::RenderState};

pub struct State {
    surface: wgpu::Surface<'static>,
    canvas: web_sys::HtmlCanvasElement,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render: RenderState,
    compute: ComputeState,
    parameters: Parameters,
    flip: bool,
    buffer_a: wgpu::Buffer,
    buffer_b: wgpu::Buffer,
    parameters_buffer: wgpu::Buffer,
}

// Rust side - create uniform buffer
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Parameters { // Should be padded to a multiple of 16 bytes for alignment
    height: u32,
    width: u32,
    t: u32,
    r: u32,
}

impl State {
    pub async fn new(canvas: web_sys::HtmlCanvasElement) -> anyhow::Result<Self> {
        let instance = Instance::new(&wgpu::InstanceDescriptor {
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

        let mut rng = StdRng::from_os_rng();
        let mut grid_data = vec![0f32; (config.width * config.height) as usize];
        rng.fill(grid_data.as_mut_slice());

        let buffer_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("buffer A"),
            contents: bytemuck::cast_slice(&grid_data),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
        });

        let buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer B"),
            size: buffer_a.size(),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let parameters = Parameters {
            width: canvas.width(),
            height: canvas.height(),
            t: 10,
            r: 5
        };

        let parameters_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Parameters"),
            contents: bytemuck::cast_slice(&[parameters]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let render = RenderState::new(&device, &buffer_a, &buffer_b, &parameters_buffer, &config);
        let compute = ComputeState::new(&device, &buffer_a, &buffer_b, &parameters_buffer);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            canvas,
            render,
            compute,
            parameters,
            buffer_a,
            buffer_b,
            parameters_buffer,
            flip: false,
        })
    }

    pub fn step(&mut self) -> anyhow::Result<()> {
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: Some("Compute Encoder"),
                });

        {
            let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("Basic Compute Pass"),
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.compute.pipeline);

            let bind_group = if self.flip {
                &self.compute.bind_group_b
            } else {
                &self.compute.bind_group_a
            };
            pass.set_bind_group(0, bind_group, &[]);

            let workgroups_x = (self.config.width + 7) / 8;
            let workgroups_y = (self.config.height + 7) / 8;
            pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
        }

        self.queue.submit(Some(encoder.finish()));
        self.flip = !self.flip;

        Ok(())
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Basic Canvas Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.render.pipeline);

            let bind_group = if self.flip {
                &self.render.bind_group_b
            } else {
                &self.render.bind_group_a
            };
            pass.set_bind_group(0, bind_group, &[]);

            pass.draw(0..4, 0..1);
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update(&mut self) {}

    pub fn resize(&mut self, width: u32, height: u32) {
        if width <= 0 || height <= 0 {
            return
        }

        if self.config.width == width && self.config.height == height {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);


        self.parameters = Parameters {
            height: height,
            width: width,
            t: 10,
            r: 5
        };

        self.queue.write_buffer(&self.parameters_buffer, 0, bytemuck::cast_slice(&[self.parameters]));

        let mut rng = StdRng::from_os_rng();
        let mut grid_data = vec![0f32; (self.config.width * self.config.height) as usize];
        rng.fill(grid_data.as_mut_slice());

        self.buffer_a = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("buffer A"),
            contents: bytemuck::cast_slice(&grid_data),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
        });

        self.buffer_b = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer B"),
            size: self.buffer_a.size(),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        self.flip = false;

        self.render.recreate_bind_groups(&self.device, &self.buffer_a, &self.buffer_b, &self.parameters_buffer);
        self.compute.recreate_bind_groups(&self.device, &self.buffer_a, &self.buffer_b, &self.parameters_buffer);

        _ = self.render()
    }
}