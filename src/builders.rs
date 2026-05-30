use std::num::NonZero;

use crate::{data::Vertex, state::State};

pub struct PipelineBuilder<'a> {
    name: &'a str,
    layout: &'a wgpu::PipelineLayout,
    vertex: wgpu::VertexState<'a>,
    fragment: wgpu::FragmentState<'a>,
    primitive: wgpu::PrimitiveState,
    multisample: wgpu::MultisampleState,
}

impl<'a> PipelineBuilder<'a> {
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

    pub fn with_primitive(&mut self, primitive: wgpu::PrimitiveState) -> &mut Self {
        self.primitive = primitive;
        self
    }

    pub fn with_multisample(&mut self, multisample: wgpu::MultisampleState) -> &mut Self {
        self.multisample = multisample;
        self
    }
}

pub struct VertexBuilder<'a> {
    shader: &'a wgpu::ShaderModule,
    entry: &'a str,
    buffers: Vec<wgpu::VertexBufferLayout<'a>>,
    options: wgpu::PipelineCompilationOptions<'a>,
}

impl<'a> VertexBuilder<'a> {
    const BASIC_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4];

    pub fn new(shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            shader,
            entry: "vs_main",
            buffers: vec![],
            options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    pub fn resolve(&'_ self) -> wgpu::VertexState<'_> {
        wgpu::VertexState {
            module: self.shader,
            entry_point: Some(self.entry),
            buffers: self.buffers.as_slice(),
            compilation_options: self.options.clone(),
        }
    }

    pub fn with_entry_point(&mut self, entry: &'a str) -> &mut Self {
        self.entry = entry;
        self
    }

    pub fn with_buffer(&mut self, layout: wgpu::VertexBufferLayout<'a>) -> &mut Self {
        self.buffers.push(layout);
        self
    }

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

pub struct FragmentBuilder<'a> {
    shader: &'a wgpu::ShaderModule,
    entry: &'a str,
    targets: Vec<Option<wgpu::ColorTargetState>>,
    options: wgpu::PipelineCompilationOptions<'a>,
}

impl<'a> FragmentBuilder<'a> {
    pub fn new(shader: &'a wgpu::ShaderModule) -> Self {
        Self {
            shader,
            entry: "fs_main",
            targets: vec![],
            options: wgpu::PipelineCompilationOptions::default(),
        }
    }

    pub fn resolve(&'a self) -> wgpu::FragmentState<'a> {
        wgpu::FragmentState {
            module: &self.shader,
            entry_point: Some(self.entry),
            targets: &self.targets,
            compilation_options: self.options.clone(),
        }
    }

    pub fn with_entry_point(&mut self, entry: &'a str) -> &mut Self {
        self.entry = entry;
        self
    }

    pub fn with_target(&mut self, target: wgpu::ColorTargetState) -> &mut Self {
        self.targets.push(Some(target));
        self
    }

    pub fn with_replace_target(&mut self, state: &State) -> &mut Self {
        self.targets.push(Some(wgpu::ColorTargetState {
            format: state.config.format,
            blend: Some(wgpu::BlendState::REPLACE),
            write_mask: wgpu::ColorWrites::ALL,
        }));
        self
    }

    pub fn with_alpha_target(&mut self, state: &State) -> &mut Self {
        self.targets.push(Some(wgpu::ColorTargetState {
            format: state.config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        }));
        self
    }

    pub fn with_compilation_options(
        &mut self,
        options: wgpu::PipelineCompilationOptions<'a>,
    ) -> &mut Self {
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
