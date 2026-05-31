use std::u32;

use keydraw::{Command, ComplexCommand, DrawKey, Event, Program, run, state::State};

struct GlyphonCommand<'a> {
    z_index: i32,
    font_system: &'a mut glyphon::FontSystem,
    swash_cache: &'a mut glyphon::SwashCache,
    text_atlas: &'a mut glyphon::TextAtlas,
    text_renderer: &'a mut glyphon::TextRenderer,
    viewport: &'a glyphon::Viewport,
    text_buffer: &'a glyphon::Buffer,
}

impl<'a> ComplexCommand for GlyphonCommand<'a> {
    fn key(&self) -> DrawKey {
        DrawKey::new(self.z_index, u32::MAX, &[])
    }

    fn prepare(&mut self, state: &State) {
        let text_area = glyphon::TextArea {
            buffer: self.text_buffer,
            left: 10.0,
            top: 10.0,
            scale: 1.0,
            bounds: glyphon::TextBounds {
                left: 0,
                top: 0,
                right: 800,
                bottom: 600,
            },
            default_color: glyphon::Color::rgb(255, 255, 255),
            custom_glyphs: &[],
        };

        self.text_renderer
            .prepare(
                &state.device,
                &state.queue,
                self.font_system,
                self.text_atlas,
                self.viewport,
                vec![text_area],
                self.swash_cache,
            )
            .unwrap();
    }

    fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        self.text_renderer
            .render(self.text_atlas, self.viewport, pass)
            .unwrap();
    }
}

struct Game {
    font_system: Option<glyphon::FontSystem>,
    swash_cache: Option<glyphon::SwashCache>,
    text_atlas: Option<glyphon::TextAtlas>,
    text_renderer: Option<glyphon::TextRenderer>,
    viewport: Option<glyphon::Viewport>,
    text_buffer: Option<glyphon::Buffer>,
}

impl Game {
    fn new() -> Self {
        Self {
            font_system: None,
            swash_cache: None,
            text_atlas: None,
            text_renderer: None,
            viewport: None,
            text_buffer: None,
        }
    }
}

impl Program for Game {
    fn init(&mut self, state: &mut State) {
        let mut font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&state.device);
        let mut viewport = glyphon::Viewport::new(&state.device, &cache);
        viewport.update(
            &state.queue,
            glyphon::Resolution {
                width: state.config.width,
                height: state.config.height,
            },
        );
        let mut atlas = glyphon::TextAtlas::new(
            &state.device,
            &state.queue,
            &cache,
            state.config.format.clone(),
        );
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &state.device,
            wgpu::MultisampleState::default(),
            None,
        );
        let mut text_buffer =
            glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(30.0, 42.0));

        let size = state.window.inner_size();
        let physical_width = size.width as f32;
        let physical_height = size.height as f32;

        text_buffer.set_size(
            &mut font_system,
            Some(physical_width),
            Some(physical_height),
        );
        text_buffer.set_text(
            &mut font_system,
            "Hello world!",
            &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Advanced,
            None,
        );
        text_buffer.shape_until_scroll(&mut font_system, false);

        self.font_system = Some(font_system);
        self.swash_cache = Some(swash_cache);
        self.text_atlas = Some(atlas);
        self.text_renderer = Some(text_renderer);
        self.viewport = Some(viewport);
        self.text_buffer = Some(text_buffer);
    }

    fn render(&'_ mut self) -> Vec<Command<'_>> {
        vec![Command::Complex(Box::new(GlyphonCommand {
            z_index: 0,
            font_system: self.font_system.as_mut().unwrap(),
            swash_cache: self.swash_cache.as_mut().unwrap(),
            text_atlas: self.text_atlas.as_mut().unwrap(),
            text_renderer: self.text_renderer.as_mut().unwrap(),
            viewport: self.viewport.as_ref().unwrap(),
            text_buffer: self.text_buffer.as_ref().unwrap(),
        }))]
    }

    fn event(&mut self, event: &Event, state: &mut State) {
        match event {
            Event::Resize(width, height) => {
                self.viewport.as_mut().unwrap().update(
                    &state.queue,
                    glyphon::Resolution {
                        width: *width,
                        height: *height,
                    },
                );
            }
        }
    }
}

fn main() {
    run(Box::new(Game::new())).unwrap();
}
