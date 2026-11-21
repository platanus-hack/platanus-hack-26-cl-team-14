use bevy::prelude::*;

/// Local Santiago coordinates in meters. East is +X and south is +Z.
pub fn world(lon: f64, lat: f64) -> Vec3 {
    Vec3::new(((lon + 70.65) * 92_915.0) as f32, 0.0, (-(lat + 33.45) * 110_905.0) as f32)
}

use bevy::{asset::RenderAssetUsages, mesh::Indices, render::render_resource::PrimitiveTopology};
use flate2::read::GzDecoder;
use std::{fs::File, io::Read};

pub type Rgba = [f32; 4];

#[derive(Default)]
pub struct Surface {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<Rgba>,
    pub indices: Vec<u32>,
}

impl Surface {
    fn vertex(&mut self, point: Vec3, normal: Vec3, color: Rgba) {
        self.positions.push(point.to_array());
        self.normals.push(normal.to_array());
        self.colors.push(color);
    }

    // Each roof keeps the polygon's courtyards. Earcut handles concave buildings.
    pub fn polygon(&mut self, rings: &[Vec<Vec3>], base: f32, height: f32, wall: Rgba, roof: Rgba) {
        if rings.is_empty() || rings[0].len() < 3 { return; }
        let top = Vec3::Y * (base + height);
        let first = self.positions.len() as u32;
        let mut coords = Vec::new();
        let mut holes = Vec::new();
        for (j, ring) in rings.iter().enumerate() {
            if j > 0 { holes.push(coords.len()/2); }
            for p in ring {
                self.vertex(*p + top, Vec3::Y, roof);
                coords.extend([p.x as f64, -p.z as f64]);
            }
        }
        if rings.len() == 1 && rings[0].len() == 4 && convex_quad(&rings[0]) {
            self.indices.extend([first,first+1,first+2,first,first+2,first+3]);
        } else if let Ok(triangles) = earcutr::earcut(&coords,&holes,2) {
            self.indices.extend(triangles.into_iter().map(|i| first+i as u32));
        }
        if height == 0.0 { return; }
        for ring in rings {
            let signed: f32 = ring.iter().zip(ring.iter().cycle().skip(1)).map(|(a,b)| a.x*b.z - b.x*a.z).sum();
            for (a,b) in ring.iter().zip(ring.iter().cycle().skip(1)) {
                let n = (*b-*a).cross(Vec3::Y).normalize_or_zero() * -signed.signum();
                let first=self.positions.len() as u32;
                for p in [*a + Vec3::Y*base, *b+Vec3::Y*base, *b+top,*a+top] { self.vertex(p,n,wall); }
                self.indices.extend([first,first+1,first+2,first,first+2,first+3]);
            }
        }
    }

    pub fn finish(self) -> Mesh {
        Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION,self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL,self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR,self.colors)
            .with_inserted_indices(Indices::U32(self.indices))
    }
}

fn convex_quad(p: &[Vec3])->bool {
    let cross = |i:usize| (p[(i+1)%4]-p[i]).cross(p[(i+2)%4]-p[(i+1)%4]).y;
    (1..4).all(|i|cross(0)*cross(i)>0.)
}

fn word(b: &[u8], at: usize) -> u16 { u16::from_le_bytes(b[at..at+2].try_into().unwrap()) }
fn integer(b: &[u8], at: usize) -> usize { u32::from_le_bytes(b[at..at+4].try_into().unwrap()) as usize }

struct Pack {
    bytes: Vec<u8>,
    offsets: Vec<usize>,
    cols: usize,
    origin: Vec3,
    step: usize,
}

impl Pack {
    fn load(name: &str) -> Self {
        let mut bytes=Vec::new();
        GzDecoder::new(File::open(format!("web/data/city_building_{name}.bin.gz")).unwrap()).read_to_end(&mut bytes).unwrap();
        let cols=word(&bytes,4) as usize;
        let n=cols*word(&bytes,6) as usize;
        let coord=|offset| i32::from_le_bytes(bytes[offset..offset+4].try_into().unwrap()) as f64 / 100000.;
        let origin=world(coord(8),coord(12));
        let step=integer(&bytes,24);
        let offsets=(0..=n).map(|i|integer(&bytes,28+i*4)).collect();
        Self{bytes,offsets,cols,origin,step}
    }

    fn position(&self,x:f32,y:f32)->Vec3 { self.origin + Vec3::new(x*0.92915, 0., -y*1.10905) }

    fn center(&self, tile: usize) -> Vec3 {
        self.position(((tile%self.cols) as f32+0.5)*self.step as f32, ((tile/self.cols) as f32+0.5)*self.step as f32)
    }

    fn slice(&self,tile:usize)->&[u8] { &self.bytes[self.offsets[tile]..self.offsets[tile+1]] }
}

fn paint(x:u16,y:u16,height:f32) -> (Rgba,Rgba) {
    let seed=(x as u32).wrapping_mul(73856093) ^ (y as u32).wrapping_mul(19349663);
    let roof=if height>30. {
        [[0.29,0.40,0.46,1.],[0.57,0.62,0.60,1.],[0.36,0.45,0.50,1.]][seed as usize%3]
    } else {
        [[0.59,0.28,0.18,1.],[0.63,0.38,0.26,1.],[0.41,0.40,0.36,1.],[0.52,0.52,0.47,1.],[0.60,0.50,0.37,1.],[0.39,0.31,0.26,1.],[0.47,0.53,0.52,1.],[0.66,0.57,0.49,1.]][seed as usize%8]
    };
    let value=0.65+(seed%23) as f32/100.;
    ([value*0.86,value*0.79,value*0.70,1.],roof)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Detail { Far, Near }

#[derive(Resource)]
pub(crate) struct Buildings {
    boxes: Pack,
    polygons: Pack,
    tiles: Vec<Option<(Detail, Entity)>>,
    material: Handle<StandardMaterial>,
}

fn box_mesh(pack:&Pack,tile:usize,scale:&impl Fn(usize)->f32)->Surface {
    let bytes=pack.slice(tile);
    let count=integer(bytes,0);
    let mut mesh=Surface::default();
    mesh.positions.reserve(count*8);
    mesh.indices.reserve(count*30);
    for i in 0..count {
        let at=4+i*12;
        let x=word(bytes,at);
        let y=word(bytes,at+2);
        let center=pack.position(x as f32,y as f32);
        let width=word(bytes,at+4) as f32*0.05;
        let depth=word(bytes,at+6) as f32*0.05;
        let (sin,cos)=(word(bytes,at+8) as f32*0.01).to_radians().sin_cos();
        let u=Vec3::new(cos,0.,-sin)*width;
        let v=Vec3::new(sin,0.,cos)*depth;
        let h=word(bytes,at+10) as f32*0.1;
        let (wall,roof)=paint(x,y,h);
        let base=mesh.positions.len() as u32;
        let corners=[center-u-v,center+u-v,center+u+v,center-u+v];
        for p in corners {mesh.vertex(p,Vec3::Y,wall);}
        for p in corners {mesh.vertex(p+Vec3::Y*h*scale(i),Vec3::Y,roof);}
        mesh.indices.extend([4,6,5,4,7,6, 0,4,5,0,5,1, 1,5,6,1,6,2, 2,6,7,2,7,3, 3,7,4,3,4,0].map(|n|base+n));
    }
    mesh
}

fn polygon_mesh(boxes:&Pack,polys:&Pack,tile:usize,scale:&impl Fn(usize)->f32)->Surface {
    let bytes=polys.slice(tile);
    let boxbytes=boxes.slice(tile);
    let count=integer(bytes,0);
    let ring_table=12+(count+1)*4;
    let vertices=ring_table+(integer(bytes,4)+1)*4;
    let heights=vertices+integer(bytes,8)*4;
    let mut mesh=Surface::default();
    for i in 0..count {
        let first=integer(bytes,12+i*4);
        let end=integer(bytes,16+i*4);
        let rings:Vec<Vec<Vec3>>=(first..end).map(|r|{
            (integer(bytes,ring_table+r*4)..integer(bytes,ring_table+(r+1)*4)).map(|v|{
                let at=vertices+4*v;
                polys.position(word(bytes,at) as f32,word(bytes,at+2) as f32)
            }).collect()
        }).collect();
        let h=word(bytes,heights+2*i) as f32*0.1;
        let (wall,roof)=paint(word(boxbytes,4+i*12),word(boxbytes,6+i*12),h);
        mesh.polygon(&rings,0.,h*scale(i),wall,roof);
    }
    mesh
}

pub fn load_buildings(mut commands:Commands, mut materials:ResMut<Assets<StandardMaterial>>) {
    let boxes=Pack::load("boxes");
    let polygons=Pack::load("polygons");
    let tiles=vec![None;boxes.offsets.len()-1];
    let count:usize=(0..tiles.len()).map(|i|boxes.slice(i)).filter(|s|!s.is_empty()).map(|s|integer(s,0)).sum();
    println!("Loaded {count} buildings from {} tiles",tiles.len());
    let material=materials.add(StandardMaterial{cull_mode:None,double_sided:false,perceptual_roughness:0.94,..default()});
    commands.insert_resource(Buildings{boxes,polygons,tiles,material});
}

pub(crate) fn draw_buildings(mut commands:Commands, mut buildings:ResMut<Buildings>, mut meshes:ResMut<Assets<Mesh>>, rig:Res<crate::Rig>) {
    let mut jobs=Vec::new();
    for tile in 0..buildings.tiles.len() {
        if buildings.boxes.slice(tile).is_empty() {continue;}
        let d=buildings.boxes.center(tile).distance(rig.target);
        // A tile is roughly 2 km wide. Include full tiles on either side of the visible frustum.
        let desired=if d>rig.distance*1.5+1700. {None}
            else if rig.distance<4200. && d<rig.distance*0.72+1500. {Some(Detail::Near)}
            else {Some(Detail::Far)};
        if buildings.tiles[tile].map(|t|t.0)==desired { continue; }
        jobs.push((d,tile,desired));
    }
    jobs.sort_by(|a,b|a.0.total_cmp(&b.0));
    for (_,tile,detail) in jobs.into_iter().take(8) {
        if let Some((_,entity))=buildings.tiles[tile].take() {commands.entity(entity).despawn();}
        if let Some(detail)=detail {
            let mesh=match detail {
                Detail::Far=>box_mesh(&buildings.boxes,tile,&|_|1.),
                Detail::Near=>polygon_mesh(&buildings.boxes,&buildings.polygons,tile,&|_|1.)
            };
            let entity=commands.spawn((Mesh3d(meshes.add(mesh.finish())),MeshMaterial3d(buildings.material.clone()))).id();
            buildings.tiles[tile]=Some((detail,entity));
        }
    }
}

use std::{collections::BTreeMap, io::BufReader, path::Path};

use serde_json::Value;


type Color4 = [f32; 4];
type GroundTiles = BTreeMap<(i32, i32), Surface>;

// Half-widths are cartographic line widths in metres.
const ROAD: Color4 = [0.48, 0.49, 0.48, 1.0];
const SMALL_ROAD: Color4 = [0.60, 0.60, 0.56, 1.0];
const GRASS: Color4 = [0.23, 0.38, 0.18, 1.0];
const WATER: Color4 = [0.16, 0.41, 0.44, 1.0];
const STONE: Color4 = [0.62, 0.59, 0.48, 1.0];
const ROOF: Color4 = [0.33, 0.40, 0.37, 1.0];
const GLASS: Color4 = [0.13, 0.32, 0.37, 1.0];

fn coordinates(line: &Value) -> Vec<Vec3> {
    line.as_array().unwrap().iter().map(|pair| {
        world(pair[0].as_f64().unwrap(), pair[1].as_f64().unwrap())
    }).collect()
}

fn tile_for(tiles: &mut GroundTiles, point: Vec3) -> &mut Surface {
    let tile = ((point.x / 1700.0).floor() as i32, (point.z / 1700.0).floor() as i32);
    tiles.entry(tile).or_default()
}

// One shared pair of vertices at each join keeps streets continuous on bends.
fn ribbon(mesh: &mut Surface, points: &[Vec3], half_width: f32, elevation: f32, color: Color4) {
    let mut points = points.to_vec();
    points.dedup();
    if points.len() < 2 { return; }
    let normal = |a: Vec3, b: Vec3| Vec3::new(a.z - b.z, 0.0, b.x - a.x).normalize_or_zero();
    let first_vertex = mesh.positions.len() as u32;
    let last = points.len() - 1;
    for (i, &p) in points.iter().enumerate() {
        let previous = normal(points[i.saturating_sub(1)], p);
        let next = normal(p, points[(i + 1).min(last)]);
        let join = (previous + next).normalize_or_zero();
        let reference = if i == last { previous } else { next };
        let offset = join * (half_width / join.dot(reference).max(0.5));
        let center = p + Vec3::Y * elevation;
        mesh.positions.extend([(center + offset).to_array(), (center - offset).to_array()]);
        mesh.normals.extend([[0.0, 1.0, 0.0]; 2]);
        mesh.colors.extend([color; 2]);
        if i != last {
            let a = first_vertex + i as u32 * 2;
            mesh.indices.extend([a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
    }
}

fn roads(tiles: &mut GroundTiles, path: &Path) {
    let fc: Value = serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
    for feature in fc["features"].as_array().unwrap() {
        let points = coordinates(&feature["geometry"]["coordinates"]);
        let (half_width, color) = match feature["properties"]["kind"].as_str().unwrap() {
            "motorway" => (6.5, ROAD),
            "trunk" | "primary" => (5.0, ROAD),
            "secondary" => (4.0, ROAD),
            _ => (2.8, SMALL_ROAD),
        };
        ribbon(tile_for(tiles, points[0]), &points, half_width, 0.13, color);
    }
}

fn landmarks(tiles: &mut GroundTiles, path: &Path) {
    let fc: Value = serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap();
    for feature in fc["features"].as_array().unwrap() {
        let props = &feature["properties"];
        let shape = &feature["geometry"]["coordinates"];
        let kind = props["kind"].as_str().unwrap();
        if kind == "peak" {
            // These are summit points. Terrain supplies their surfaces.
            continue;
        }
        if kind == "river" {
            let points = coordinates(shape);
            ribbon(tile_for(tiles, points[points.len() / 2]), &points, 15.0, 0.08, WATER);
            continue;
        }
        let rings: Vec<Vec<Vec3>> = shape.as_array().unwrap().iter().map(|ring| {
            let mut points = coordinates(ring);
            points.pop(); // GeoJSON repeats its closing coordinate.
            points
        }).collect();
        let tile = tile_for(tiles, rings[0][0]);
        match kind {
            "park" => tile.polygon(&rings, 0.03, 0.0, GRASS, GRASS),
            "tower" => tile.polygon(&rings, 0.0, props["height_m"].as_f64().unwrap() as f32 + 0.05, GLASS, GLASS),
            "palace" => tile.polygon(&rings, 0.0, props["levels"].as_f64().unwrap() as f32 * 3.5 + 0.05, STONE, ROOF),
            "stadium" => {
                // Mark the surveyed stadium perimeter. The source has no structure height.
                let mut perimeter = rings[0].clone();
                perimeter.push(perimeter[0]);
                ribbon(tile, &perimeter, 4.5, 0.18, STONE);
            }
            _ => (),
        }
    }
}

pub(crate) fn load_ground(root: &Path) -> Vec<(String, Mesh)> {
    let mut tiles = GroundTiles::new();
    roads(&mut tiles, &root.join("data/mvp/roads.geojson"));
    landmarks(&mut tiles, &root.join("web/data/santiago_landmarks.geojson"));
    tiles.into_iter().map(|((x,z), surface)| (format!("City ground {x} {z}"), surface.finish())).collect()
}

pub(crate) fn setup_ground(mut commands:Commands, mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<StandardMaterial>>) {
    let mat=materials.add(StandardMaterial{cull_mode:None,perceptual_roughness:1.,..default()});
    for (name,mesh) in load_ground(Path::new(".")) {
        commands.spawn((Name::new(name),Mesh3d(meshes.add(mesh)),MeshMaterial3d(mat.clone())));
    }
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(190000.,190000.))),
        MeshMaterial3d(materials.add(Color::srgb_u8(179,171,150))),
        Transform::from_xyz(0.,-0.2,0.)
    ));
}
