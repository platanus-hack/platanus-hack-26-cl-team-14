mod activity;
mod ground;
mod terrain;
use std::path::Path;
pub use terrain::terrain_from_xz;
use bevy::prelude::*;

/// Local Santiago coordinates in meters. East is +X and south is +Z.
pub fn world(lon: f64, lat: f64) -> Vec3 {
    let x=((lon + 70.65) * 92_915.0) as f32; let z=(-(lat + 33.45) * 110_905.0) as f32; Vec3::new(x,terrain_from_xz(x,z),z)
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

    fn position(&self,x:f32,y:f32)->Vec3 { let x=self.origin.x + x*0.92915; let z=self.origin.z - y*1.10905; Vec3::new(x,terrain_from_xz(x,z),z) }

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
    heights:activity::HeightScales,
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
    commands.insert_resource(Buildings{boxes,polygons,tiles,material, heights:activity::HeightScales::load(Path::new("web/data/city_building_block_ids.bin.gz"))});
}

pub(crate) fn draw_buildings(mut commands:Commands, mut buildings:ResMut<Buildings>, mut meshes:ResMut<Assets<Mesh>>, mut rig:ResMut<crate::Rig>,scenario:Res<crate::net::Scenario>) {
    if scenario.is_changed() && let Some(focus)=buildings.heights.update(&scenario.0) {
        for tile in &mut buildings.tiles {if let Some((_,e))=tile.take(){commands.entity(e).despawn();}}
        if let Some([lon,lat])=focus {rig.target=world(lon,lat);}
    }
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
                Detail::Far=>box_mesh(&buildings.boxes,tile,&|i|buildings.heights.scale(tile,i)),
                Detail::Near=>polygon_mesh(&buildings.boxes,&buildings.polygons,tile,&|i|buildings.heights.scale(tile,i))
            };
            let entity=commands.spawn((Mesh3d(meshes.add(mesh.finish())),MeshMaterial3d(buildings.material.clone()))).id();
            buildings.tiles[tile]=Some((detail,entity));
        }
    }
}

pub(crate) fn setup_ground(mut commands:Commands, mut meshes:ResMut<Assets<Mesh>>,mut materials:ResMut<Assets<StandardMaterial>>) {
    let mat=materials.add(StandardMaterial{cull_mode:None,perceptual_roughness:1.,..default()});
    for (name,mesh) in ground::load_ground(Path::new(".")) {
        commands.spawn((Name::new(name),Mesh3d(meshes.add(mesh)),MeshMaterial3d(mat.clone())));
    }
}
