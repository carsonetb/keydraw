use keydraw::{
    Command, DrawKey, Event, Program, SimpleCommand,
    builders::{BindGroupLayoutEntryBuilder, FragmentBuilder, PipelineBuilder, VertexBuilder},
    data::{CameraUniform, Vertex},
    run,
    state::State,
};

struct Game {
    camera_buffer: Option<wgpu::Buffer>,
    pipeline_index: u32,
    material_index: u32,
}

impl Game {
    fn new() -> Self {
        Self {
            camera_buffer: None,
            pipeline_index: u32::MAX,
            material_index: u32::MAX,
        }
    }
}

impl Program for Game {
    fn init(&mut self, state: &mut State) {
        let shader = state
            .device
            .create_shader_module(wgpu::include_wgsl!("basic.wgsl"));

        let camera_uniform = CameraUniform::new(800.0, 600.0);
        let camera_buffer =
            state.create_uniform_buffer("Camera Buffer", bytemuck::cast_slice(&[camera_uniform]));

        let camera_bind_group_layout = state.create_bind_group_layout(
            "camera_bind_group_layout",
            &[BindGroupLayoutEntryBuilder::new_uniform_buffer(0)
                .at_vertex()
                .resolve()],
        );
        let material = state.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            state.create_simple_layout("Render Pipeline", &[&camera_bind_group_layout]);

        let pipeline = PipelineBuilder::new(
            "Render Pipeline",
            &render_pipeline_layout,
            &VertexBuilder::new(&shader).with_simple_vertex_buffer(),
            &FragmentBuilder::new(&shader).with_replace_target(state),
        )
        .resolve(state);

        self.pipeline_index = state.get_pipeline();
        self.material_index = state.get_material();
        state.pipeline_db.insert(self.pipeline_index, pipeline);
        state.material_db.insert(self.material_index, material);

        self.camera_buffer = Some(camera_buffer);
    }

    fn render(&'_ mut self) -> Vec<Command<'_>> {
        vec![Command::Simple(SimpleCommand {
            key: DrawKey::new(0, self.pipeline_index, &[self.material_index]),
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
        })]
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
