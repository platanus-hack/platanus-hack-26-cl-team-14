use bevy::{
    camera::{MainPassResolutionOverride, Viewport},
    core_pipeline::{Core3d, Core3dSystems, core_3d::CORE_3D_DEPTH_FORMAT},
    mesh::VertexBufferLayout,
    prelude::*,
    render::{
        Extract, ExtractSchedule, Render, RenderApp, RenderSystems,
        camera::ExtractedCamera,
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        view::{ViewDepthStencilTexture, ViewTarget, ViewUniform, ViewUniformOffset, ViewUniforms},
    },
};

type Instance = [f32; 8];

#[derive(Resource, Default)]
pub(super) struct Marks(Vec<Instance>);

impl Marks {
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    pub(super) fn push(&mut self, position: Vec3, radius_px: f32, color: LinearRgba) {
        self.0.push([
            position.x,
            position.y,
            position.z,
            radius_px,
            color.red,
            color.green,
            color.blue,
            color.alpha,
        ]);
    }
}

pub(super) struct MarksPlugin;

impl Plugin for MarksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Marks>();
        let shader = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .add(Shader::from_wesl(SHADER, "overview_marks.wesl"));
        let layout = BindGroupLayoutDescriptor::new(
            "overview camera",
            &BindGroupLayoutEntries::single(
                ShaderStages::VERTEX,
                binding_types::uniform_buffer::<ViewUniform>(true),
            ),
        );
        app.sub_app_mut(RenderApp)
            .insert_resource(MarkPipeline { shader, layout })
            .insert_resource(Instances(RawBufferVec::new(BufferUsages::VERTEX)))
            .add_systems(ExtractSchedule, extract)
            .add_systems(Render, upload.in_set(RenderSystems::PrepareResources))
            .add_systems(
                Core3d,
                draw.after(Core3dSystems::MainPass)
                    .before(Core3dSystems::EarlyPostProcess),
            );
    }
}

#[derive(Resource)]
struct Instances(RawBufferVec<Instance>);

fn extract(marks: Extract<Res<Marks>>, mut instances: ResMut<Instances>) {
    if marks.is_changed() {
        instances.0.values_mut().clone_from(&marks.0);
    }
}

fn upload(mut instances: ResMut<Instances>, device: Res<RenderDevice>, queue: Res<RenderQueue>) {
    if instances.is_changed() && !instances.0.is_empty() {
        let buffer = &mut instances.0;
        if buffer.len() > buffer.capacity() {
            buffer.reserve(buffer.len().next_power_of_two(), &device);
        }
        buffer.write_buffer(&device, &queue);
    }
}

#[derive(Resource)]
struct MarkPipeline {
    shader: Handle<Shader>,
    layout: BindGroupLayoutDescriptor,
}

impl MarkPipeline {
    fn descriptor(&self, format: TextureFormat, samples: u32) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("overview mover marks".into()),
            layout: vec![self.layout.clone()],
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![VertexBufferLayout {
                    array_stride: size_of::<Instance>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: vec![
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 0,
                            shader_location: 0,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 16,
                            shader_location: 1,
                        },
                    ],
                }],
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_compare: Some(CompareFunction::GreaterEqual),
                depth_write_enabled: Some(false),
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: samples,
                ..default()
            },
            ..default()
        }
    }
}

fn draw(
    view: ViewQuery<(
        &ViewTarget,
        &ViewDepthStencilTexture,
        &ViewUniformOffset,
        &Msaa,
        &ExtractedCamera,
        Option<&MainPassResolutionOverride>,
    )>,
    mut ctx: RenderContext,
    instances: Res<Instances>,
    marks: Res<MarkPipeline>,
    uniforms: Res<ViewUniforms>,
    cache: Res<PipelineCache>,
    mut selected: Local<Option<((TextureFormat, u32), CachedRenderPipelineId)>>,
) {
    if instances.0.is_empty() {
        return;
    }
    let (target, depth, offset, msaa, camera, resolution) = view.into_inner();
    let key = (target.main_texture_format(), msaa.samples());
    if selected.as_ref().map(|(previous, _)| *previous) != Some(key) {
        *selected = Some((
            key,
            cache.queue_render_pipeline(marks.descriptor(key.0, key.1)),
        ));
    }
    let Some(pipeline) = cache.get_render_pipeline(selected.as_ref().unwrap().1) else {
        return;
    };
    let Some(binding) = uniforms.uniforms.binding() else {
        return;
    };
    let group = ctx.render_device().create_bind_group(
        "overview camera",
        &cache.get_bind_group_layout(&marks.layout),
        &BindGroupEntries::single(binding),
    );
    let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("overview mover marks"),
        color_attachments: &[Some(target.get_color_attachment())],
        depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
        ..default()
    });
    if let Some(viewport) =
        Viewport::from_viewport_and_override(camera.viewport.as_ref(), resolution)
    {
        pass.set_camera_viewport(&viewport);
    }
    pass.set_render_pipeline(pipeline);
    pass.set_bind_group(0, &group, &[offset.offset]);
    pass.set_vertex_buffer(0, instances.0.buffer().unwrap().slice(..));
    pass.draw(0..4, 0..instances.0.len() as u32);
}

// All instances use the same four-vertex billboard. Its radius stays in pixels as the camera zooms.
const SHADER: &str = r#"
import bevy_render::view::View;

@group(0) @binding(0) var<uniform> camera: View;

struct MarkVertex {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
    @location(1) corner: vec2f,
}

@vertex
fn vertex(@builtin(vertex_index) index: u32, @location(0) mark: vec4f, @location(1) color: vec4f) -> MarkVertex {
    let corners = array<vec2f, 4>(vec2f(-1., -1.), vec2f(1., -1.), vec2f(-1., 1.), vec2f(1., 1.));
    let corner = corners[index];
    let clip = camera.clip_from_world * vec4f(mark.xyz, 1.);
    let delta = corner * (2. * mark.w) / camera.main_pass_viewport.zw * clip.w;
    return MarkVertex(vec4f(clip.xy + delta, clip.zw), color, corner);
}

@fragment
fn fragment(mark: MarkVertex) -> @location(0) vec4f {
    if dot(mark.corner, mark.corner) > 1. { discard; }
    return mark.color;
}
"#;
