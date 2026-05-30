use crate::{data::Vertex, state::State};

/// Any application must implement the `Program` trait. To run one, you must
/// hand it over, along with control of the thread, to the engine.
pub trait Program {
    /// Here you can initialize the pipelines and textures you're going to use.
    ///
    /// Currently, you are responsible for building the `wgpu` `RenderPipeline`s
    /// and `Texture`s yourself.
    fn init(&mut self, state: &mut State);
    /// Handle other window events, which may modify state.
    fn event(&mut self, event: &Event, state: &mut State);
    /// Here your program may render, by pushing a set of commands.
    fn render(&mut self) -> Vec<Command>;
}

/// A miscelaneous event that should be handled by the program.
pub enum Event {
    /// Emitted when the window is resized. Contains the new width and height
    /// of the window.
    Resize(u32, u32),
}

/// Aids in the sorting of `Command`s.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DrawKey {
    pub z_index: i32,
    /// The shader this object uses.
    /// Every different shader requires a different pipeline.
    pub pipeline_id: u32,
    /// Allows sorting by material, aka a bind group, for aded efficiency.
    /// Can be `u32::MAX` to represent nothing.
    pub material_id: u32,
}

/// Represents a set of similar instances.
#[derive(Debug)]
pub struct Command {
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

impl Command {
    /// Gets the number of instances this command will render.
    pub fn instances(&self) -> u32 {
        if self.stride == 0 {
            1
        } else {
            self.instances.len() as u32 / self.stride
        }
    }
}
