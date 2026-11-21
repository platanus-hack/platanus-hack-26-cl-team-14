use bevy::{
    core_pipeline::{
        Core3d, Core3dSystems, FullscreenShader, tonemapping::tonemapping, upscaling::upscaling,
    },
    prelude::*,
    render::{
        RenderApp,
        render_resource::*,
        renderer::{RenderContext, ViewQuery},
        view::{ViewDepthStencilTexture, ViewTarget},
    },
};

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
            ),
        ),
    );
    app.sub_app_mut(RenderApp)
        .insert_resource(VotePass { shader, layout })
        .add_systems(
            Core3d,
            draw.in_set(Core3dSystems::PostProcess)
                .after(tonemapping)
                .before(upscaling),
        );
}

fn draw(
    view: ViewQuery<(&ViewTarget, &ViewDepthStencilTexture)>,
    mut ctx: RenderContext,
    cache: Res<PipelineCache>,
    pass: Res<VotePass>,
    fullscreen: Res<FullscreenShader>,
    mut selected: Local<Option<(TextureFormat, CachedRenderPipelineId)>>,
) {
    let (target, depth) = view.into_inner();
    let format = target.main_texture_format();
    if selected.as_ref().map(|p| p.0) != Some(format) {
        let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("city pixel vote".into()),
            layout: vec![pass.layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: pass.shader.clone(),
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        });
        *selected = Some((format, id));
    }
    let Some(pipeline) = cache.get_render_pipeline(selected.as_ref().unwrap().1) else {
        return;
    };
    let Some(z) = depth.attachment.depth_stencil_views().depth_only_view() else {
        return;
    };
    let post = target.post_process_write();
    let bindings = ctx.render_device().create_bind_group(
        "city pixel vote",
        &cache.get_bind_group_layout(&pass.layout),
        &BindGroupEntries::sequential((post.source, z)),
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
