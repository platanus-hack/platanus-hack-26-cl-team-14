//! Surface cars keep their route clock while the map omits underground or open-water poses.
use super::*;
use serde::Deserialize;
use std::collections::BTreeMap;

type Outline = Vec<[f64; 2]>;
type Edges = Vec<(Vec2, Vec2)>;

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Crossing {
    Tunnel,
    Bridge,
    Water,
}

type Intervals = Vec<(f32, f32, Crossing)>;

#[derive(Deserialize)]
pub(super) struct Pack {
    water: Vec<Vec<Outline>>,
    bridges: Vec<(u64, f32, Outline)>,
    pub(super) local: BTreeMap<String, Intervals>,
    pub(super) od: BTreeMap<String, Intervals>,
}

struct Polygon {
    low: Vec2,
    high: Vec2,
    // Only edges in the query's east-west column can cross a ray to the south.
    columns: HashMap<i32, Edges>,
}

fn xz(p: [f64; 2]) -> Vec2 {
    world(p[0], p[1]).xz()
}

fn tile(p: Vec2) -> IVec2 {
    (p / 200.).floor().as_ivec2()
}

fn insert<T: Copy>(tiles: &mut HashMap<IVec2, Vec<T>>, low: Vec2, high: Vec2, item: T) {
    let a = tile(low);
    let b = tile(high);
    for x in a.x..=b.x {
        for y in a.y..=b.y {
            tiles.entry(IVec2::new(x, y)).or_default().push(item);
        }
    }
}

impl Polygon {
    fn new(rings: Vec<Outline>) -> Self {
        let mut p = Self {
            low: Vec2::splat(f32::INFINITY),
            high: Vec2::splat(f32::NEG_INFINITY),
            columns: HashMap::new(),
        };
        for ring in rings {
            for edge in ring.windows(2) {
                let (a, b) = (xz(edge[0]), xz(edge[1]));
                p.low = p.low.min(a).min(b);
                p.high = p.high.max(a).max(b);
                for column in tile(a.min(b)).x..=tile(a.max(b)).x {
                    p.columns.entry(column).or_default().push((a, b));
                }
            }
        }
        p
    }

    fn contains(&self, p: Vec2, column: i32) -> bool {
        if !p.cmpge(self.low).all() || !p.cmple(self.high).all() {
            return false;
        }
        self.columns.get(&column).is_some_and(|edges| {
            // Even-odd parity includes both exterior rings and islands.
            edges
                .iter()
                .filter(|&&(a, b)| {
                    (a.x > p.x) != (b.x > p.x)
                        && p.y < a.y + (b.y - a.y) * (p.x - a.x) / (b.x - a.x)
                })
                .count()
                % 2
                == 1
        })
    }
}

#[derive(Clone, Copy)]
struct Deck {
    a: Vec2,
    b: Vec2,
    radius: f32,
}

// The pack allows 3.8 m of route simplification; the drawn lane adds 1.6 m.
const BRIDGE_TOLERANCE: f32 = 5.4;

impl Deck {
    fn lane(&self, p: Vec2, heading: Vec2, tolerance: f32) -> Option<Vec2> {
        let ab = self.b - self.a;
        let axis = ab.normalize_or_zero();
        // A tunnel running under a cross street does not use that street's deck.
        if axis.dot(heading).abs() < std::f32::consts::FRAC_1_SQRT_2 {
            return None;
        }
        let closest = self.a + axis * (p - self.a).dot(axis).clamp(0., ab.length());
        let offset = p - closest;
        if offset.length_squared() > (self.radius + tolerance).powi(2) {
            return None;
        }
        Some(closest + offset.clamp_length_max(self.radius - 0.05))
    }
}

pub(super) struct Water {
    polygons: Vec<Polygon>,
    areas: HashMap<IVec2, Vec<usize>>,
    decks: HashMap<IVec2, Vec<Deck>>,
}

impl Water {
    pub(super) fn new(pack: Pack) -> Self {
        let polygons: Vec<_> = pack.water.into_iter().map(Polygon::new).collect();
        let mut areas = HashMap::new();
        for (i, p) in polygons.iter().enumerate() {
            insert(&mut areas, p.low, p.high, i);
        }
        let mut decks = HashMap::new();
        for (_, width, line) in pack.bridges {
            for segment in line.windows(2) {
                let (a, b) = (xz(segment[0]), xz(segment[1]));
                let radius = width * 0.5;
                insert(
                    &mut decks,
                    a.min(b) - (radius + BRIDGE_TOLERANCE),
                    a.max(b) + (radius + BRIDGE_TOLERANCE),
                    Deck { a, b, radius },
                );
            }
        }
        Self {
            polygons,
            areas,
            decks,
        }
    }

    pub(super) fn surface_position(
        &self,
        path: &Path,
        metres: f32,
        position: Vec3,
        heading: Vec3,
    ) -> Option<Vec3> {
        let spans = &path.crossings;
        let i = spans.partition_point(|&(_, end, _)| end <= metres);
        let crossing = spans
            .get(i)
            .filter(|&&(start, _, _)| start <= metres)
            .map(|s| s.2);
        if crossing == Some(Crossing::Tunnel) {
            return None;
        }
        let p = position.xz();
        let cell = tile(p);
        if !self
            .areas
            .get(&cell)
            .is_some_and(|rows| rows.iter().any(|&i| self.polygons[i].contains(p, cell.x)))
        {
            return Some(position);
        }
        let heading = heading.xz().normalize_or_zero();
        // Only a route matched to the bridge can use the simplification allowance.
        // Snap its lateral offset onto the deck. Other cars need to be on the deck already.
        let tolerance = if crossing == Some(Crossing::Bridge) {
            BRIDGE_TOLERANCE
        } else {
            0.
        };
        let lane = self
            .decks
            .get(&cell)?
            .iter()
            .filter_map(|deck| deck.lane(p, heading, tolerance))
            .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))?;
        Some(Vec3::new(
            lane.x,
            terrain_from_xz(lane.x, lane.y) + 0.30,
            lane.y,
        ))
    }
}

pub(super) fn set_crossings(path: &mut Arc<Path>, raw: Option<&Intervals>) {
    let Some(intervals) = raw else {
        return;
    };
    let path = Arc::get_mut(path).unwrap();
    let metres = |index: f32| {
        let a = path.metres[index.floor() as usize];
        let b = path.metres[index.ceil() as usize];
        a + (b - a) * index.fract()
    };
    path.crossings = intervals
        .iter()
        .map(|&(a, b, kind)| (metres(a), metres(b), kind))
        .collect();
}
