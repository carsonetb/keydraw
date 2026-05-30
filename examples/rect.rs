use graphics_wgpu::{
    Command, DrawKey, Event, Program,
    data::{CameraUniform, Vertex},
    run,
    state::State,
};
use wgpu::util::DeviceExt;

struct RectDrawer {
    command: Command,
}

impl RectDrawer {
    fn new(pipeline_id: u32, material_id: u32) -> Self {
        Self {
            command: Command {
                key: DrawKey {
                    z_index: 0,
                    pipeline_id,
                    material_id,
                },
                vertices: vec![
                    Vertex {
                        position: [0.0, 0.0, 0.0],
                        color: [0.0, 0.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [1.0, 0.0, 0.0],
                        color: [0.0, 0.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [0.0, 1.0, 0.0],
                        color: [0.0, 0.0, 0.0, 0.0],
                    },
                    Vertex {
                        position: [1.0, 1.0, 0.0],
                        color: [0.0, 0.0, 0.0, 0.0],
                    },
                ],
                indices: vec![0, 2, 3, 0, 3, 1],
                instances: vec![],
                stride: size_of::<f32>() as u32 * 8,
            },
        }
    }

    fn draw(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        self.command.instances.append(
            &mut bytemuck::bytes_of(&[x, y, width, height, color[0], color[1], color[2], color[3]])
                .to_vec(),
        );
    }
}

struct Game {
    pub camera_buffer: Option<wgpu::Buffer>,
}

impl Game {
    fn new() -> Self {
        Self {
            camera_buffer: None,
        }
    }
}

impl Program for Game {
    fn init(&mut self, state: &mut State) {
        let shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("rect.wgsl"));

        let camera_uniform = CameraUniform::new(800.0, 600.0);
        let camera_buffer = state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group_layout =
            state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });
        let material = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    immediate_size: 0,
                });
        let pipeline = state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<Vertex>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: size_of::<f32>() as u64 * 8,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview_mask: None,
                cache: None,
            });

        state.pipeline_db.insert(0, pipeline);
        state.material_db.insert(0, material);

        self.camera_buffer = Some(camera_buffer);
    }

    fn render(&mut self) -> Vec<Command> {
        let mut renderer = RectDrawer::new(0, 0);
        renderer.draw(10.0, 10.0, 200.0, 200.0, [1.0, 1.0, 1.0, 0.5]);
        renderer.draw(100.0, 100.0, 200.0, 200.0, [1.0, 1.0, 1.0, 0.5]);
        vec![renderer.command]
    }

    fn event(&mut self, event: &Event, state: &mut State) {
        match event {
            Event::Resize(width, height) => {
                let camera = CameraUniform::new(*width as f32, *height as f32);
                state.queue.write_buffer(
                    &self.camera_buffer.as_ref().unwrap(),
                    0,
                    &bytemuck::cast_slice(&[camera]),
                );
            }
        }
    }
}

fn main() {
    run(Box::new(Game::new())).unwrap();
}
