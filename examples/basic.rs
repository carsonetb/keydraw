use graphics_wgpu::{
    Command, DrawKey, Event, Program,
    data::{CameraUniform, Vertex},
    run,
    state::State,
};
use wgpu::util::DeviceExt;

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
            .create_shader_module(wgpu::include_wgsl!("basic.wgsl"));

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
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
                    }],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: state.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
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
        vec![Command {
            key: DrawKey {
                z_index: 0,
                pipeline_id: 0,
                material_id: 0,
            },
            vertices: vec![
                Vertex {
                    position: [200.0, 100.0, 0.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [100.0, 400.0, 0.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                Vertex {
                    position: [400.0, 400.0, 0.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
            instances: vec![],
            stride: 0,
        }]
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
