// The 3D camera keeps its effects, visibility, and input. Only its render target changes.
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat},
    window::PrimaryWindow,
};

const SCALE: u32 = 3;
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

pub struct PixelMode;

impl Plugin for PixelMode {
    fn build(&self, app: &mut App) {
        if std::env::var("SANTIAGO_PIXEL").as_deref() == Ok("1") {
            app.add_systems(PostStartup, setup)
                .add_systems(Update, fit_window);
        }
    }
}

#[derive(Component)]
struct CityCanvas;

fn sample_size(window: &Window) -> UVec2 {
    UVec2::new(
        window.physical_width().div_ceil(SCALE).max(1),
        window.physical_height().div_ceil(SCALE).max(1),
    )
}

fn canvas_size(size: UVec2, window: &Window) -> Vec2 {
    size.as_vec2() * (SCALE as f32 / window.scale_factor())
}

fn setup(
    mut commands: Commands,
    mut camera: Single<(Entity, &mut Camera), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = sample_size(&window);
    let mut target = Image::new_target_texture(size.x, size.y, TextureFormat::Bgra8UnormSrgb, None);
    target.sampler = ImageSampler::nearest();
    let image = images.add(target);

    camera.1.order = -1;
    commands
        .entity(camera.0)
        .insert(RenderTarget::from(image.clone()));
    let mut sprite = Sprite::from_image(image);
    sprite.custom_size = Some(canvas_size(size, &window));
    commands.spawn((sprite, CityCanvas, SCREEN_LAYER));
    commands.spawn((Camera2d, Msaa::Off, SCREEN_LAYER, IsDefaultUiCamera));
}

fn fit_window(
    window: Single<&Window, With<PrimaryWindow>>,
    mut canvas: Single<&mut Sprite, With<CityCanvas>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = sample_size(&window);
    let draw_size = canvas_size(size, &window);
    if canvas.custom_size == Some(draw_size) && images.get(&canvas.image).unwrap().size() == size {
        return;
    }
    canvas.custom_size = Some(draw_size);
    images.get_mut(&canvas.image).unwrap().resize(Extent3d {
        width: size.x,
        height: size.y,
        depth_or_array_layers: 1,
    });
}
