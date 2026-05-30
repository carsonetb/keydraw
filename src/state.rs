use std::{collections::HashMap, ops::Range, sync::Arc};

use anyhow::Ok;
use wgpu::{
    Backends, BufferUsages, Color, CommandEncoderDescriptor, Device, DeviceDescriptor,
    ExperimentalFeatures, Features, IndexFormat, Instance, InstanceDescriptor, Limits, LoadOp,
    Operations, PowerPreference, PresentMode, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor, Trace,
};

use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, window::Window};

use crate::{Command, DrawKey, buffer::DynBuffer};

pub struct State {
    // High-level window stuff
    surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    pub window: Arc<Window>,

    // Buffers
    vertex_buffer: DynBuffer,
    index_buffer: DynBuffer,
    instance_buffer: DynBuffer,

    // Pipeline and materials
    pub pipeline_db: HashMap<u32, RenderPipeline>,
    pub material_db: HashMap<u32, wgpu::BindGroup>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(&InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::VULKAN,
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::defaults()
                },
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let vertex_buffer = DynBuffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            "MegaVert Buffer",
        );
        let index_buffer = DynBuffer::new(
            &device,
            BufferUsages::INDEX | BufferUsages::COPY_DST,
            "MegaIndex Buffer",
        );
        let instance_buffer = DynBuffer::new(
            &device,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
            "MegaInstance Buffer",
        );

        Ok(Self {
            is_surface_configured: false,
            surface,
            device,
            queue,
            config,
            window,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            pipeline_db: HashMap::new(),
            material_db: HashMap::new(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self, mut commands: Vec<Command>) -> anyhow::Result<()> {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.surface.get_current_texture() {
            std::result::Result::Ok(texture) => texture,
            Err(e) => match e {
                SurfaceError::Timeout | SurfaceError::OutOfMemory | SurfaceError::Other => {
                    return Ok(());
                }
                SurfaceError::Outdated => {
                    self.surface.configure(&self.device, &self.config);
                    return Ok(());
                }
                SurfaceError::Lost => {
                    anyhow::bail!("Lost device.");
                }
            },
        };

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Sort and batch commands
        commands.sort_unstable_by_key(|cmd| cmd.key);

        let mut megaverts = vec![];
        let mut megadices = vec![];
        let mut megainsts = vec![];

        struct BatchRecord {
            key: DrawKey,
            indices: Range<u32>,
            instances: Range<u32>,
            vertex_begin: u32,
        }
        let mut records = Vec::with_capacity(commands.len());

        let mut instance = 0;
        let mut vertex = 0;
        for command in commands {
            let begin = megadices.len() as u32;
            let num_indices = command.indices.len() as u32;

            let num_instances = command.instances();

            records.push(BatchRecord {
                key: command.key,
                indices: begin..(begin + num_indices),
                instances: instance..(instance + num_instances),
                vertex_begin: vertex,
            });

            megaverts.extend_from_slice(&command.vertices);
            megadices.extend_from_slice(&command.indices);
            megainsts.extend_from_slice(&command.instances);

            instance += num_instances;
            vertex += command.vertices.len() as u32;
        }

        if megadices.len() % 2 != 0 {
            megadices.push(0);
        }

        self.vertex_buffer
            .write_data(&self.device, &self.queue, bytemuck::cast_slice(&megaverts));
        self.index_buffer
            .write_data(&self.device, &self.queue, bytemuck::cast_slice(&megadices));
        self.instance_buffer.write_data(
            &self.device,
            &self.queue,
            bytemuck::cast_slice(&megainsts),
        );

        // Here is where most of the drawing gets done, also where the render
        // pipeline is used.
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.buffer.slice(..), IndexFormat::Uint16);

            let mut pipeline_id = None;
            let mut material_id = None;

            for record in records {
                if pipeline_id != Some(record.key.pipeline_id) {
                    let pipeline = self.pipeline_db.get(&record.key.pipeline_id).unwrap();
                    render_pass.set_pipeline(pipeline);
                    pipeline_id = Some(record.key.pipeline_id);
                }

                if record.key.material_id != u32::MAX && material_id != Some(record.key.material_id)
                {
                    let material = self.material_db.get(&record.key.material_id).unwrap();
                    render_pass.set_bind_group(0, material, &[]);
                    material_id = Some(record.key.material_id);
                }

                render_pass.draw_indexed(
                    record.indices,
                    record.vertex_begin as i32,
                    record.instances,
                );
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
}
