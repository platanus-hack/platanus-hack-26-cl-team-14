mod vote;
// Keep text at output resolution while the city uses a fixed pixel grid.
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureFormat, TextureUsages},
    window::PrimaryWindow,
};

const SCALE: u32 = 3;
const SAMPLES_PER_AXIS: u32 = 2;
const SCREEN_LAYER: RenderLayers = RenderLayers::layer(1);

pub struct PixelMode;

impl Plugin for PixelMode {
    fn build(&self, app: &mut App) {
        if std::env::var("SANTIAGO_PIXEL").as_deref() != Ok("0") {
            vote::install(app);
            app.add_systems(PostStartup, setup)
                .add_systems(Update, fit_window);
        }
        #[cfg(target_family = "wasm")]
        app.add_systems(Update, fit_panels);
    }
}

#[derive(Component)]
pub(crate) struct CityCanvas;

/// Window pixels per city pixel.
#[cfg(not(target_family = "wasm"))]
fn scale(_: &Window) -> u32 {
    SCALE
}

/// Browser: the window has any size. Keep the grid of the 1920-pixel design.
#[cfg(target_family = "wasm")]
fn scale(window: &Window) -> u32 {
    ((SCALE * window.physical_width()) as f32 / 1920.).round().max(1.) as u32
}

fn sample_size(window: &Window) -> UVec2 {
    UVec2::new(
        window.physical_width().div_ceil(scale(window)).max(1),
        window.physical_height().div_ceil(scale(window)).max(1),
    )
}

fn canvas_size(size: UVec2, window: &Window) -> Vec2 {
    size.as_vec2() * (scale(window) as f32 / window.scale_factor())
}

/// Browser: the panels keep the proportions of the 1920-pixel design in a window of any size.
#[cfg(target_family = "wasm")]
fn fit_panels(window: Single<&Window, With<PrimaryWindow>>, mut ui: ResMut<UiScale>) {
    let fit = window.physical_width() as f32 / 1920.;
    if ui.0 != fit {
        ui.0 = fit;
    }
}

fn setup(
    mut commands: Commands,
    mut camera: Single<(Entity, &mut Camera, &mut Camera3d)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = sample_size(&window);
    let sampled = size * SAMPLES_PER_AXIS;
    let mut target =
        Image::new_target_texture(sampled.x, sampled.y, TextureFormat::Bgra8UnormSrgb, None);
    target.sampler = ImageSampler::nearest();
    let image = images.add(target);

    camera.1.order = -1;
    camera.2.depth_texture_usages =
        (TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING).into();
    commands.entity(camera.0).insert((
        RenderTarget::from(image.clone()),
        Msaa::Off,
        bevy::render::view::DebandDither::Disabled,
    ));
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
    let sampled = size * SAMPLES_PER_AXIS;
    if canvas.custom_size == Some(draw_size) && images.get(&canvas.image).unwrap().size() == sampled
    {
        return;
    }
    canvas.custom_size = Some(draw_size);
    images.get_mut(&canvas.image).unwrap().resize(Extent3d {
        width: sampled.x,
        height: sampled.y,
        depth_or_array_layers: 1,
    });
}
