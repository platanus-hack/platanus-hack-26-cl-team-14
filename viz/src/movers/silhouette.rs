//! People are flat one-tone silhouettes in profile, drawn once into an atlas at startup.
//! Each figure is a 1.75 m card that turns to the camera and mirrors to show its walk direction.
use super::motion::{GAIT_FRAMES, walk_pose};
use super::*;
use bevy::{
    image::{ImageFilterMode, ImageSamplerDescriptor},
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    transform::TransformSystems,
};

// One cell is 1 m wide and 2 m tall at 128 pixels per metre.
const PX: usize = 128;
const STAND: usize = GAIT_FRAMES as usize;
const SIT: usize = STAND + 1;
const PAIR: usize = SIT + 1;
const CELLS: usize = PAIR + 2;
const MIPS: u32 = 8;

/// Feet are at the entity's translation. Halos and flows read only these people.
#[derive(Component)]
pub(crate) struct Silhouette;

#[derive(Resource)]
pub(crate) struct Silhouettes {
    pub atlas: Handle<Image>,
    pub walk: Vec<Handle<Mesh>>,
    pub stand: Handle<Mesh>,
    pub sit: Handle<Mesh>,
    #[allow(dead_code)] // Copihue's gathering halos use the talking pair.
    pub pair: Handle<Mesh>,
}

impl FromWorld for Silhouettes {
    fn from_world(world: &mut World) -> Self {
        let atlas = world.resource_mut::<Assets<Image>>().add(atlas());
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            atlas,
            walk: (0..GAIT_FRAMES).map(|f| meshes.add(walk(f))).collect(),
            stand: meshes.add(stand()),
            sit: meshes.add(quad(SIT, 1)),
            pair: meshes.add(quad(PAIR, 2)),
        }
    }
}

pub(super) struct SilhouettePlugin;

impl Plugin for SilhouettePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Silhouettes>().add_systems(
            PostUpdate,
            (tag, face_camera).chain().before(TransformSystems::Propagate),
        );
    }
}

pub(super) fn walk(frame: u8) -> Mesh {
    quad(frame as usize, 1)
}

pub(super) fn stand() -> Mesh {
    quad(STAND, 1)
}

/// Turn the card to the eye. `dir` is the walk direction; the profile faces it on screen.
pub(crate) fn face(transform: &mut Transform, dir: Vec3, eye: Vec3) {
    transform.look_to(transform.translation - eye, Vec3::Y);
    let right = transform.rotation * Vec3::X;
    transform.scale.x = transform.scale.x.abs() * if dir.dot(right) < 0. { -1. } else { 1. };
}

// The pool spawns mover figures; mark the people among them.
fn tag(mut commands: Commands, models: Res<Models>, added: Query<(Entity, &Mesh3d), Added<Figure>>) {
    for (entity, mesh) in &added {
        let person = models.mesh.iter().any(|(model, handle)| {
            matches!(model, Model::Person | Model::Walking(_)) && handle.id() == mesh.id()
        });
        if person {
            commands.entity(entity).insert(Silhouette);
        }
    }
}

// Placement sets the rotation with +X along the walk each frame; replace it with the camera turn.
fn face_camera(
    camera: Single<&Transform, (With<Camera3d>, Without<Silhouette>)>,
    mut people: Query<&mut Transform, (With<Silhouette>, Changed<Transform>)>,
) {
    for mut transform in &mut people {
        let dir = transform.rotation * Vec3::X;
        face(&mut transform, dir, camera.translation);
    }
}

fn quad(cell: usize, cells: usize) -> Mesh {
    let half = cells as f32 * 0.5;
    let (u0, u1) = (cell as f32 / CELLS as f32, (cell + cells) as f32 / CELLS as f32);
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-half, 0., 0.], [half, 0., 0.], [half, 2., 0.], [-half, 2., 0.]],
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; 4])
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[u0, 1.], [u1, 1.], [u1, 0.], [u0, 0.]])
        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]))
}

// A capsule whose radius tapers from the first point to the second.
type Stroke = (Vec2, Vec2, f32, f32);

struct Body {
    hip: Vec2,
    shoulder: Vec2,
    legs: [[Vec2; 2]; 2], // knee, ankle
    arms: [[Vec2; 2]; 2], // elbow, wrist
}

// Proportions of a 1.75 m adult: the head is about 1/7.5 of the height. +x is forward.
fn strokes(b: &Body) -> Vec<Stroke> {
    let up = (b.shoulder - b.hip).normalize();
    let fwd = Vec2::new(up.y, -up.x);
    let neck = b.shoulder + up * 0.06;
    let mid = b.hip.lerp(b.shoulder, 0.45);
    let mut s = vec![
        (neck + up * 0.12 + fwd * 0.012, neck + up * 0.185, 0.088, 0.088),
        (b.shoulder, neck + up * 0.09, 0.05, 0.05),
        (b.shoulder - up * 0.05 + fwd * 0.02, mid + fwd * 0.01, 0.125, 0.11),
        (mid + fwd * 0.01, b.hip + up * 0.03, 0.11, 0.12),
    ];
    for [knee, ankle] in b.legs {
        s.push((b.hip, knee, 0.085, 0.065));
        s.push((knee, ankle, 0.062, 0.045));
        s.push((ankle, ankle + Vec2::new(0.15, -0.045), 0.045, 0.035));
    }
    for [elbow, wrist] in b.arms {
        let shoulder = b.shoulder - up * 0.03;
        s.push((shoulder, elbow, 0.055, 0.045));
        s.push((elbow, wrist, 0.045, 0.038));
        s.push((wrist, wrist + (wrist - elbow).normalize() * 0.07, 0.042, 0.034));
    }
    s
}

fn knee(hip: Vec2, ankle: Vec2) -> Vec2 {
    let leg = ankle - hip;
    let bend = (0.43_f32.powi(2) - leg.length_squared() * 0.25).max(0.).sqrt();
    (hip + ankle) * 0.5 + Vec2::new(-leg.y, leg.x).normalize() * bend
}

fn arm(shoulder: Vec2, swing: f32) -> [Vec2; 2] {
    let at = |angle: f32| Vec2::new(angle.sin(), -angle.cos());
    let elbow = shoulder + at(swing * 0.7) * 0.29;
    [elbow, elbow + at(swing * 1.3 + 0.12) * 0.26]
}

fn walker(frame: u8) -> Body {
    let lift = walk_pose(frame, true).hip_lift;
    let hip = Vec2::new(0., 0.9 + lift);
    let shoulder = Vec2::new(0.03, 1.42 + lift);
    let side = |left| {
        let pose = walk_pose(frame, left);
        let ankle = Vec2::new(pose.foot_x, 0.085 + pose.foot_lift);
        ([knee(hip, ankle), ankle], arm(shoulder, pose.shoulder))
    };
    let (left, right) = (side(true), side(false));
    Body { hip, shoulder, legs: [left.0, right.0], arms: [left.1, right.1] }
}

fn standing(gesture: bool) -> Body {
    let hip = Vec2::new(0., 0.9);
    let shoulder = Vec2::new(0.01, 1.42);
    let feet = [-0.05, 0.06].map(|x| {
        let ankle = Vec2::new(x, 0.085);
        [knee(hip, ankle), ankle]
    });
    let raised = [Vec2::new(0.1, 1.12), Vec2::new(0.27, 1.24)];
    Body { hip, shoulder, legs: feet, arms: [arm(shoulder, 0.06), if gesture { raised } else { arm(shoulder, -0.04) }] }
}

// Seat height 0.45 m, hands on the thighs.
fn seated() -> Body {
    let hand = [Vec2::new(-0.05, 0.78), Vec2::new(0.2, 0.6)];
    Body {
        hip: Vec2::new(-0.12, 0.5),
        shoulder: Vec2::new(-0.15, 1.02),
        legs: [
            [Vec2::new(0.3, 0.53), Vec2::new(0.31, 0.085)],
            [Vec2::new(0.26, 0.51), Vec2::new(0.22, 0.085)],
        ],
        arms: [hand, hand],
    }
}

// Shift (and mirror) a body inside its cell.
fn placed(body: Body, x: f32, mirror: bool, scale: f32) -> Vec<Stroke> {
    let sign = if mirror { -1. } else { 1. };
    let at = |p: Vec2| Vec2::new(p.x * sign * scale + x, p.y * scale);
    strokes(&body)
        .into_iter()
        .map(|(a, b, ra, rb)| (at(a), at(b), ra * scale, rb * scale))
        .collect()
}

fn atlas() -> Image {
    let (width, height) = (CELLS * PX, 2 * PX);
    let mut cells: Vec<(usize, usize, Vec<Stroke>)> =
        (0..GAIT_FRAMES).map(|f| (f as usize, 1, placed(walker(f), 0., false, 1.))).collect();
    cells.push((STAND, 1, placed(standing(false), 0., false, 1.)));
    cells.push((SIT, 1, placed(seated(), 0., false, 1.)));
    let mut pair = placed(standing(true), -0.32, false, 1.);
    pair.extend(placed(standing(false), 0.32, true, 0.95));
    cells.push((PAIR, 2, pair));
    let mut alpha = vec![0u8; width * height];
    for (cell, span, strokes) in cells {
        for py in 0..height {
            for px in 0..span * PX {
                let p = Vec2::new(
                    (px as f32 + 0.5) / PX as f32 - span as f32 * 0.5,
                    (height - py) as f32 / PX as f32 - 0.5 / PX as f32,
                );
                let d = strokes.iter().fold(f32::MAX, |d, &(a, b, ra, rb)| {
                    let ab = b - a;
                    let t = ((p - a).dot(ab) / ab.length_squared().max(1e-6)).clamp(0., 1.);
                    d.min((p - a - ab * t).length() - (ra + (rb - ra) * t))
                });
                alpha[py * width + cell * PX + px] = ((0.5 - d * PX as f32).clamp(0., 1.) * 255.) as u8;
            }
        }
    }
    // White texels carry the shape in alpha; the material colour tints them.
    // Each smaller level gains alpha so thin limbs do not fade at a distance.
    let mut data = Vec::new();
    let (mut w, mut h) = (width, height);
    for level in 0..MIPS {
        if level > 0 {
            let at = |x: usize, y: usize| alpha[y * w + x] as u32;
            let next: Vec<u8> = (0..h / 2 * (w / 2))
                .map(|i| {
                    let (x, y) = (i % (w / 2) * 2, i / (w / 2) * 2);
                    ((at(x, y) + at(x + 1, y) + at(x, y + 1) + at(x + 1, y + 1)) * 3 / 8).min(255) as u8
                })
                .collect();
            (alpha, w, h) = (next, w / 2, h / 2);
        }
        data.extend(alpha.iter().flat_map(|&a| [255, 255, 255, a]));
    }
    let mut image = Image::new_uninit(
        Extent3d { width: width as u32, height: height as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(data);
    image.texture_descriptor.mip_level_count = MIPS;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mipmap_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::linear()
    });
    image
}
