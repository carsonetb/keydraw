use keydraw::{
    Command, DrawKey, Program, SimpleCommand,
    builders::{FragmentBuilder, PipelineBuilder, VertexBuilder},
    data::{CameraUniform, Vertex},
    run,
    state::State,
};
use wgpu::util::DeviceExt;

struct RectDrawer {
    command: SimpleCommand,
}

impl RectDrawer {
    fn new(pipeline_id: u32, material_id: u32) -> Self {
        Self {
            command: SimpleCommand {
                key: DrawKey::new(0, pipeline_id, &[material_id]),
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
    camera_buffer: Option<wgpu::Buffer>,
    rect_pipeline_index: u32,
    tri_pipeline_index: u32,
    material_index: u32,
}

impl Game {
    fn new() -> Self {
        Self {
            camera_buffer: None,
            rect_pipeline_index: u32::MAX,
            tri_pipeline_index: u32::MAX,
            material_index: u32::MAX,
        }
    }
}

impl Program for Game {
    fn init(&mut self, state: &mut State) {
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

        let tri_shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("basic.wgsl"));
        let tri_pipeline_layout =
            state.create_simple_layout("Render Pipeline", &[&camera_bind_group_layout]);
        let tri_pipeline = PipelineBuilder::new(
            "Render Pipeline",
            &tri_pipeline_layout,
            &VertexBuilder::new(&tri_shader).with_simple_vertex_buffer(),
            &FragmentBuilder::new(&tri_shader).with_replace_target(state),
        )
        .resolve(state);

        let rect_shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("rect.wgsl"));
        let rect_pipeline_layout =
            state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[&camera_bind_group_layout],
                    immediate_size: 0,
                });
        let rect_pipeline = PipelineBuilder::new(
            "Render Pipeline",
            &rect_pipeline_layout,
            &VertexBuilder::new(&rect_shader)
                .with_simple_vertex_buffer()
                .with_buffer(wgpu::VertexBufferLayout {
                    array_stride: size_of::<f32>() as u64 * 8,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4],
                }),
            &FragmentBuilder::new(&rect_shader).with_alpha_target(state),
        )
        .resolve(state);

        self.rect_pipeline_index = state.get_pipeline();
        self.tri_pipeline_index = state.get_pipeline();
        self.material_index = state.get_material();
        state
            .pipeline_db
            .insert(self.rect_pipeline_index, rect_pipeline);
        state
            .pipeline_db
            .insert(self.tri_pipeline_index, tri_pipeline);
        state.material_db.insert(self.material_index, material);

        self.camera_buffer = Some(camera_buffer);
    }

    fn render(&'_ mut self, _state: &mut State) -> Vec<Command<'_>> {
        let mut renderer = RectDrawer::new(self.rect_pipeline_index, self.material_index);
        renderer.draw(10.0, 10.0, 200.0, 200.0, [1.0, 1.0, 1.0, 0.5]);
        renderer.draw(100.0, 100.0, 200.0, 200.0, [1.0, 1.0, 1.0, 0.5]);
        vec![
            Command::Simple(renderer.command),
            Command::Simple(SimpleCommand {
                key: DrawKey::new(0, self.tri_pipeline_index, &[self.material_index]),
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
            }),
        ]
    }

    fn event(&mut self, event: &winit::event::WindowEvent, state: &mut State) {
        match event {
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                let camera = CameraUniform::new(*width as f32, *height as f32);
                state.queue.write_buffer(
                    &self.camera_buffer.as_ref().unwrap(),
                    0,
                    &bytemuck::cast_slice(&[camera]),
                );
            }
            _ => (),
        }
    }
}

fn main() {
    run(Box::new(Game::new())).unwrap();
}
