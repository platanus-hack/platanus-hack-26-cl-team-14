use bevy::{
    core_pipeline::{
        Core3d, Core3dSystems, FullscreenShader,
        core_3d::{main_opaque_pass_3d, main_transparent_pass_3d},
    },
    prelude::*,
    render::{
        Extract, ExtractSchedule, RenderApp,
        render_resource::*,
        renderer::{RenderContext, RenderQueue, ViewQuery},
        view::{ExtractedView, ViewDepthStencilTexture, ViewTarget},
    },
};

#[derive(Resource, Default)]
struct DarkPaper {
    night: bool,
    title: Option<f32>,
    to_sun: Vec3,
}

#[derive(Default, Clone, Copy, PartialEq, ShaderType)]
struct PrintView {
    up_and_focal: Vec4,
    sun_and_title: Vec4,
}

fn extract_paper(
    night: Extract<Res<crate::Night>>,
    title: Extract<Res<crate::boot::TitleLight>>,
    subway: Extract<Res<crate::city::SubwayView>>,
    sun: Extract<Query<&GlobalTransform, With<DirectionalLight>>>,
    mut paper: ResMut<DarkPaper>,
) {
    paper.night = night.0 || subway.0;
    paper.title = title.0;
    if let Ok(sun) = sun.single() {
        paper.to_sun = sun.back().into();
    }
}

#[derive(Resource)]
struct VotePass {
    shader: Handle<Shader>,
    layout: BindGroupLayoutDescriptor,
}

pub fn install(app: &mut App) {
    let shader = app
        .world_mut()
        .resource_mut::<Assets<Shader>>()
        .add(Shader::from_wgsl(include_str!("vote.wgsl"), "vote.wgsl"));
    let layout = BindGroupLayoutDescriptor::new(
        "city pixel color and depth",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                binding_types::texture_2d(TextureSampleType::Float { filterable: false }),
                binding_types::texture_depth_2d(),
                binding_types::uniform_buffer::<PrintView>(false),
            ),
        ),
    );
    app.sub_app_mut(RenderApp)
        .insert_resource(VotePass { shader, layout })
        .init_resource::<DarkPaper>()
        .add_systems(ExtractSchedule, extract_paper)
        .add_systems(
            Core3d,
            // Without this order the print can run before the opaque pass, which then covers the ink.
            draw.in_set(Core3dSystems::MainPass)
                .after(main_opaque_pass_3d)
                .before(main_transparent_pass_3d),
        );
}

fn draw(
    view: ViewQuery<(&ViewTarget, &ViewDepthStencilTexture, &ExtractedView)>,
    mut ctx: RenderContext,
    cache: Res<PipelineCache>,
    pass: Res<VotePass>,
    fullscreen: Res<FullscreenShader>,
    paper: Res<DarkPaper>,
    queue: Res<RenderQueue>,
    mut uniform: Local<UniformBuffer<PrintView>>,
    mut selected: Local<Option<((TextureFormat, &'static str), CachedRenderPipelineId)>>,
) {
    let (target, depth, camera) = view.into_inner();
    let format = target.main_texture_format();
    let entry = if paper.title.is_some() {
        "title"
    } else if paper.night {
        "night"
    } else {
        "day"
    };
    let key = (format, entry);
    if selected.as_ref().map(|p| p.0) != Some(key) {
        let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("city pixel vote".into()),
            layout: vec![pass.layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: pass.shader.clone(),
                entry_point: Some(entry.into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        });
        *selected = Some((key, id));
    }
    let Some(pipeline) = cache.get_render_pipeline(selected.as_ref().unwrap().1) else {
        return;
    };
    let Some(z) = depth.attachment.depth_stencil_views().depth_only_view() else {
        return;
    };
    let view_rotation = camera.world_from_view.rotation().inverse();
    let focal = 0.5 * camera.viewport.w as f32 * camera.clip_from_view.y_axis.y;
    let value = PrintView {
        up_and_focal: (view_rotation * Vec3::Y).extend(focal),
        sun_and_title: (view_rotation * paper.to_sun).extend(paper.title.unwrap_or_default()),
    };
    if uniform.binding().is_none() || *uniform.get() != value {
        uniform.set(value);
        uniform.write_buffer(ctx.render_device(), &queue);
    }
    let post = target.post_process_write();
    let bindings = ctx.render_device().create_bind_group(
        "city pixel vote",
        &cache.get_bind_group_layout(&pass.layout),
        &BindGroupEntries::sequential((post.source, z, uniform.binding().unwrap())),
    );
    let mut render = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("city pixel vote"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            ..default()
        });
    render.set_pipeline(pipeline);
    render.set_bind_group(0, &bindings, &[]);
    render.draw(0..3, 0..1);
}
