use crate::{data::Vertex, state::State};

/// Any application must implement the `Program` trait. To run one, you must
/// hand it over, along with control of the thread, to the engine.
pub trait Program {
    /// Here you can initialize the pipelines and textures you're going to use.
    ///
    /// Currently, you are responsible for building the `wgpu` `RenderPipeline`s
    /// and `Texture`s yourself.
    fn init(&mut self, state: &mut State) {
        let _ = state;
    }

    /// Handle other window events, which may modify state.
    fn event(&mut self, event: &winit::event::WindowEvent, state: &mut State) {
        let _ = (event, state);
    }

    // Handle other device events.
    fn device_event(&mut self, device_event: &winit::event::DeviceEvent, state: &mut State) {
        let _ = (device_event, state);
    }

    /// Here your program may render, by pushing a set of commands.
    fn render(&'_ mut self) -> Vec<Command<'_>> {
        vec![]
    }

    /// Passed on from the [`winit`] event.
    fn new_events(&mut self) {}

    /// Passed on from the [`winit`] event.
    fn about_to_wait(&mut self) {}
}

/// Aids in the sorting of `Command`s.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DrawKey {
    pub z_index: i32,
    /// The shader this object uses.
    /// Every different shader requires a different pipeline.
    /// Complex commands may also set this to u32::MAX if they whish to use
    /// their own pipeline, but the previous one won't be cleared up.
    pub pipeline_id: u32,
    /// Allows sorting by material, aka a bind group, for aded efficiency.
    /// Can be `u32::MAX` to represent nothing.
    /// Up to four materials may be used.
    pub material_ids: [u32; 4],
}

impl DrawKey {
    /// Create a new [`DrawKey`]. You may not use more than four groups.
    pub fn new(z_index: i32, pipeline_id: u32, groups: &[u32]) -> Self {
        let mut bind_groups = [u32::MAX; 4];
        for (i, &g) in groups.iter().enumerate().take(4) {
            bind_groups[i] = g;
        }
        Self {
            z_index,
            pipeline_id,
            material_ids: bind_groups,
        }
    }
}

pub enum Command<'a> {
    Simple(SimpleCommand),
    Complex(Box<dyn ComplexCommand + 'a>),
}

impl<'a> Command<'a> {
    pub fn key(&self) -> DrawKey {
        match self {
            Command::Simple(cmd) => cmd.key,
            Command::Complex(cmd) => cmd.key(),
        }
    }
}

/// Represents a set of similar instances.
#[derive(Debug)]
pub struct SimpleCommand {
    /// Sorted by this key.
    pub key: DrawKey,
    /// The vertices for a single instance. The engine heavily relies on
    /// instancing. For example, if you wanted to render a bunch of rectangles,
    /// instead of sending a bunch of different vertices for each rectangle,
    /// you would send four vertices at (0,0), (1,0), (0,1), and (1,1). Then,
    /// via instance parameters you would supply a position and scale.
    pub vertices: Vec<Vertex>,
    /// Typical indices, indexes the `vertices` vec.
    pub indices: Vec<u16>,
    /// Type-erased instance data.
    pub instances: Vec<u8>,
    /// Width of the data for a single instance, in bytes.
    pub stride: u32,
}

impl SimpleCommand {
    /// Gets the number of instances this command will render.
    pub fn instances(&self) -> u32 {
        if self.stride == 0 {
            1
        } else {
            self.instances.len() as u32 / self.stride
        }
    }
}

pub trait ComplexCommand {
    fn key(&self) -> DrawKey;
    fn prepare(&mut self, state: &State);
    fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>);
}
