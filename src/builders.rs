use std::num::NonZero;

use crate::{data::Vertex, state::State};

/// A helper to build an entire `wgpu::RenderPipeline`.
pub struct PipelineBuilder<'a> {
    name: &'a str,
    layout: &'a wgpu::PipelineLayout,
    vertex: wgpu::VertexState<'a>,
    fragment: wgpu::FragmentState<'a>,
    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,
}

impl<'a> PipelineBuilder<'a> {
    /// Create a new pipeline, given some basic information.
    /// This automatically sets `primitive` and `multisample` to some defaults,
    /// these can be overridden via `with_primitive` and `with_multisample`.
    ///
    /// # Arguments
    ///
    /// * `name`: The name of this pipeline, used in debug and erorr messages.
    /// * `layout`: The pipeline layout.
    /// * `vertex`: Description for the vertex shader.
    /// * `fragment`: Description for the fragment shader.
    pub fn new(
        name: &'a str,
        layout: &'a wgpu::PipelineLayout,
        vertex: &'a VertexBuilder,
        fragment: &'a FragmentBuilder,
    ) -> Self {
        Self {
            name,
            layout,
            vertex: vertex.resolve(),
            fragment: fragment.resolve(),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }

    /// Convert to a `wgpu::RenderPipeline`.
    pub fn resolve(self, state: &State) -> wgpu::RenderPipeline {
        state
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(self.name),
                layout: Some(self.layout),
                vertex: self.vertex,
                primitive: self.primitive,
                depth_stencil: None,
                multisample: self.multisample,
                fragment: Some(self.fragment),
                multiview_mask: None,
                cache: None,
            })
    }

    /// Set a custom `wgpu::PrimitiveState` for the pipeline.
    pub fn with_primitive(&mut self, primitive: wgpu::PrimitiveState) -> &mut Self {
        self.primitive = primitive;
        self
    }

    /// Set a custom `wgpu::MultisampleState` for this pipeline.
    pub fn with_multisample(&mut self, multisample: wgpu::MultisampleState) -> &mut Self {
        self.multisample = multisample;
        self
    }
}

/// A helper to build a `wgpu::VertexState`. This essentially specifies the
/// layout of a vertex shader.
pub struct VertexBuilder<'a> {
    shader: &'a wgpu::ShaderModule,
    entry: &'a str,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    options: wgpu::PipelineCompilationOptions<'a>,
}

impl<'a> VertexBuilder<'a> {
    const BASIC_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    /// From a shader, create a new `VertexBuilder`, with no buffers and
    /// default options. The entry point is 'vs_main' by default.
    pub fn new(shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            shader,
            entry: "vs_main",
            buffers: vec![],
            options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    /// Convert to a `wgpu::VertexState`.
    pub fn resolve(&'_ self) -> wgpu::VertexState<'_> {
        wgpu::VertexState {
            module: self.shader,
            entry_point: Some(self.entry),
            buffers: self.buffers.as_slice(),
            compilation_options: self.options.clone(),
        }
    }

    /// Set a custom entry point name. By default this is 'vs_main'.
    pub fn with_entry_point(&mut self, entry: &'a str) -> &mut Self {
        self.entry = entry;
        self
    }

    /// Add a buffer to the vertex shader. You should use the
    /// `wgpu::vertex_attr_array` macro to create the attributes.
    pub fn with_buffer(&mut self, layout: wgpu::VertexBufferLayout<'a>) -> &mut Self {
        self.buffers.push(layout);
        self
    }

    /// A simple vertex buffer for the builtin `Vertex` struct. Right now this
    /// is probably required, but hopefully this can be changed in the future.
    pub fn with_simple_vertex_buffer(&mut self) -> &mut Self {
        self.with_buffer(wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::BASIC_ATTRIBUTES,
        })
    }

    pub fn with_options(&mut self, options: wgpu::PipelineCompilationOptions<'a>) -> &mut Self {
        self.options = options;
        self
    }
}

/// Similar to a `VertexBuilder`, but for fragment shaders.
pub struct FragmentBuilder<'a> {
    shader: &'a wgpu::ShaderModule,
    entry: &'a str,
    targets: Vec<Option<wgpu::ColorTargetState>>,
    options: wgpu::PipelineCompilationOptions<'a>,
}

impl<'a> FragmentBuilder<'a> {
    /// Create from a shader. The default entry point name is 'fs_main'.
    pub fn new(shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            shader,
            entry: "fs_main",
            targets: vec![],
            options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    /// Convert to a `wgpu::FragmentState`.
    pub fn resolve(&'a self) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: &self.shader,
            entry_point: Some(self.entry),
            targets: &self.targets,
            compilation_options: self.options.clone(),
        }
    }

    /// Set a custom entry point name. The default is 'fs_main'.
    pub fn with_entry_point(&mut self, entry: &'a str) -> &mut Self {
        self.entry = entry;
        self
    }

    /// Add a target. This function mirrors the `with_buffer` function for
    /// `VertexBuilder`.
    pub fn with_target(&mut self, target: wgpu::ColorTargetState) -> &mut Self {
        self.targets.push(Some(target));
        self
    }

    /// Adds a simple target where the color passed replaces whatever color was
    /// previously on the texture.
    pub fn with_replace_target(&mut self, state: &State) -> &mut Self {
        self.targets.push(Some(wgpu::ColorTargetState {
            format: state.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }));
        self
    }

    /// Adds a simple target where the color blends with whatever color was
    /// previously on the texture. This is the recommended target.
    pub fn with_alpha_target(&mut self, state: &State) -> &mut Self {
        self.targets.push(Some(wgpu::ColorTargetState {
            format: state.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }));
        self
    }

    pub fn with_options(&mut self, options: wgpu::PipelineCompilationOptions<'a>) -> &mut Self {
        self.options = options;
        self
    }
}

pub struct BindGroupLayoutEntryBuilder {
    binding: u32,
    visibility: wgpu::ShaderStages,
    ty: wgpu::BindingType,
    count: Option<NonZero<u32>>,
}

impl BindGroupLayoutEntryBuilder {
    pub fn new(binding: u32, ty: wgpu::BindingType) -> Self {
        Self {
            binding,
            visibility: wgpu::ShaderStages::NONE,
            ty,
            count: None,
        }
    }

    pub fn new_uniform_buffer(binding: u32) -> Self {
        Self::new(
            binding,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        )
    }

    pub fn resolve(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.visibility,
            ty: self.ty,
            count: self.count,
        }
    }

    pub fn at_vertex(&mut self) -> &mut Self {
        self.visibility |= wgpu::ShaderStages::VERTEX;
        self
    }

    pub fn at_fragment(&mut self) -> &mut Self {
        self.visibility |= wgpu::ShaderStages::FRAGMENT;
        self
    }
}
