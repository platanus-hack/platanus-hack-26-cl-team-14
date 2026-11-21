use bevy::{
    asset::{load_internal_asset, uuid_handle},
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

pub(super) type FerryMaterial = ExtendedMaterial<StandardMaterial, RiverDepth>;

#[derive(Asset, AsBindGroup, Reflect, Clone, Debug)]
pub(super) struct RiverDepth {
    #[texture(100, sample_type = "s_int")]
    pub dem: Handle<Image>,
    #[uniform(101)]
    pub grid: Vec4,
    #[uniform(102)]
    pub aboard: u32,
}

const SHADER: Handle<Shader> = uuid_handle!("1cf92555-60ee-4a00-ad2d-005515dc7c8b");

impl MaterialExtension for RiverDepth {
    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }
}

pub(super) fn install(app: &mut App) {
    load_internal_asset!(app, SHADER, "underpass.wesl", Shader::from_wesl);
    app.add_plugins(MaterialPlugin::<FerryMaterial>::default());
}
