use crate::city::{Surface, ground::ribbon};
use bevy::{light::NotShadowCaster, prelude::*};

// Seconds on the presentation clock, independent of the simulated hour or replay speed.
const CLEARED: f32 = 0.85;
const RISE: f32 = 1.20;
const SETTLED: f32 = 2.65;

#[derive(Component)]
pub(crate) struct Motion {
    started: Option<f64>,
    phase: Phase,
}

enum Phase {
    Depart { rest: Transform, depth: f32 },
    Outline,
    Arrive { rest: Transform, depth: f32 },
}

impl Motion {
    fn new(phase: Phase) -> Self {
        Self {
            started: None,
            phase,
        }
    }
}

pub(super) fn arrive(commands: &mut Commands, entity: Entity, rest: Transform, depth: f32) {
    commands.entity(entity).insert((
        Visibility::Hidden,
        Motion::new(Phase::Arrive { rest, depth }),
    ));
}

pub(super) fn depart(commands: &mut Commands, entity: Entity, rest: Transform, depth: f32) {
    commands
        .entity(entity)
        .insert(Motion::new(Phase::Depart { rest, depth }));
}

pub(super) fn cancel(commands: &mut Commands, visual: Entity) {
    commands
        .entity(visual)
        .remove::<Motion>()
        .insert((Transform::IDENTITY, Visibility::Inherited));
}

pub(super) fn outline(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    footprint: &[Vec<Vec3>],
) -> Entity {
    let mut mesh = Surface::default();
    let ink = [0.027, 0.036, 0.105, 1.];
    for ring in footprint.iter().filter(|ring| ring.len() >= 3) {
        let mut closed = ring.clone();
        closed.push(ring[0]);
        let perimeter: f32 = closed.windows(2).map(|w| w[0].distance(w[1])).sum();
        ribbon(
            &mut mesh,
            &closed,
            (perimeter * 0.003).clamp(0.4, 1.9),
            0.92,
            ink,
        );
    }
    commands
        .spawn((
            Mesh3d(meshes.add(mesh.finish())),
            MeshMaterial3d(material),
            NotShadowCaster,
            Visibility::Hidden,
            Motion::new(Phase::Outline),
        ))
        .id()
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

pub(crate) fn animate(
    time: Res<Time<Real>>,
    title: Res<crate::boot::TitleLight>,
    mut commands: Commands,
    mut moving: Query<(Entity, &mut Motion, &mut Transform, &mut Visibility)>,
) {
    if title.0.is_some() {
        return;
    }
    let now = time.elapsed_secs_f64();
    for (entity, mut motion, mut pose, mut visibility) in &mut moving {
        let elapsed = (now - *motion.started.get_or_insert(now)) as f32;
        match motion.phase {
            Phase::Depart { rest, depth } => {
                pose.translation.y = rest.translation.y - depth * ease(elapsed / CLEARED);
                if elapsed >= CLEARED {
                    commands.entity(entity).despawn();
                }
            }
            Phase::Outline => {
                if elapsed >= SETTLED {
                    commands.entity(entity).despawn();
                } else if elapsed >= CLEARED {
                    visibility.set_if_neq(Visibility::Inherited);
                }
            }
            Phase::Arrive { rest, depth } => {
                if elapsed < RISE {
                    continue;
                }
                let progress = ease((elapsed - RISE) / (SETTLED - RISE));
                pose.translation.y = rest.translation.y - depth * (1. - progress);
                visibility.set_if_neq(Visibility::Inherited);
                if elapsed >= SETTLED {
                    commands.entity(entity).remove::<Motion>();
                }
            }
        }
    }
}
