use glam::{DMat4, DQuat, DVec3, DVec4};
use reqwest::{Client, Url};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const ROOT: &str = "https://tile.googleapis.com/v1/3dtiles/root.json";
const HALF: f64 = 250.;
const ERROR: f64 = 2.;

fn ecef(lon: f64, lat: f64, height: f64) -> DVec3 {
    let (sl, cl) = lat.to_radians().sin_cos();
    let (so, co) = lon.to_radians().sin_cos();
    let n = 6378137. / (1. - 0.0066943799901413165 * sl * sl).sqrt();
    DVec3::new(
        (n + height) * cl * co,
        (n + height) * cl * so,
        (n * (1. - 0.0066943799901413165) + height) * sl,
    )
}

// Existing mapped place references in tools/pack/named_places.py.
// These are OSM-object centres, not inferred simulation block assignments.
fn anchor(repo: &Path, site: &str) -> Result<(f64, f64, f64)> {
    let (lon, lat) = match site {
        "mut" => (-70.6011321, -33.4172801), // OSM relation/18760408
        "monumental" => (-70.6059663, -33.5065368), // OSM relation/14111047
        _ => return Err("site must be mut or monumental".into()),
    };
    let dem = fs::read(repo.join("web/data/terrain.i16"))?;
    let col = (lon + 70.85) * 1200.;
    let row = (-33.3 - lat) * 1200.;
    let (i, j) = (col as usize, row as usize);
    let (u, v) = (col - i as f64, row - j as f64);
    let h = |x: usize, y: usize| {
        let p = 2 * (y * 481 + x);
        i16::from_le_bytes([dem[p], dem[p + 1]]) as f64
    };
    let (a, b, c, d) = (h(i, j), h(i + 1, j), h(i, j + 1), h(i + 1, j + 1));
    let height = if u + v <= 1. {
        a + u * (b - a) + v * (c - a)
    } else {
        d + (1. - u) * (c - d) + (1. - v) * (b - d)
    };
    Ok((lon, lat, height))
}

fn earth_to_local(lon: f64, lat: f64, height: f64) -> DMat4 {
    let (sl, cl) = lat.to_radians().sin_cos();
    let (so, co) = lon.to_radians().sin_cos();
    let east = DVec3::new(-so, co, 0.);
    let up = DVec3::new(cl * co, cl * so, sl);
    let south = DVec3::new(sl * co, sl * so, -cl);
    let rot =
        DMat4::from_cols(east.extend(0.), up.extend(0.), south.extend(0.), DVec4::W).transpose();
    rot * DMat4::from_translation(-ecef(lon, lat, height))
}

fn matrix(v: &Value) -> DMat4 {
    v.as_array()
        .map(|a| DMat4::from_cols_array(&std::array::from_fn(|i| a[i].as_f64().unwrap())))
        .unwrap_or(DMat4::IDENTITY)
}

fn node_matrix(v: &Value) -> DMat4 {
    if v["matrix"].is_array() {
        return matrix(&v["matrix"]);
    }
    let vec = |name: &str, default: DVec3| {
        v[name]
            .as_array()
            .map(|a| {
                DVec3::new(
                    a[0].as_f64().unwrap(),
                    a[1].as_f64().unwrap(),
                    a[2].as_f64().unwrap(),
                )
            })
            .unwrap_or(default)
    };
    let q = v["rotation"]
        .as_array()
        .map(|a| DQuat::from_array(std::array::from_fn(|i| a[i].as_f64().unwrap())))
        .unwrap_or(DQuat::IDENTITY);
    DMat4::from_scale_rotation_translation(
        vec("scale", DVec3::ONE),
        q,
        vec("translation", DVec3::ZERO),
    )
}

fn center_distance(b: &Value, transform: DMat4, local: DMat4) -> f64 {
    let center = if let Some(r) = b["region"].as_array() {
        local.transform_point3(ecef(
            ((r[0].as_f64().unwrap() + r[2].as_f64().unwrap()) / 2.).to_degrees(),
            ((r[1].as_f64().unwrap() + r[3].as_f64().unwrap()) / 2.).to_degrees(),
            0.,
        ))
    } else {
        let a = b["box"]
            .as_array()
            .or_else(|| b["sphere"].as_array())
            .unwrap();
        (local * transform).transform_point3(DVec3::new(
            a[0].as_f64().unwrap(),
            a[1].as_f64().unwrap(),
            a[2].as_f64().unwrap(),
        ))
    };
    center.x * center.x + center.z * center.z
}

// Conservative projected bounds retain full edge tiles. No mesh clipping.
fn intersects(b: &Value, transform: DMat4, local: DMat4) -> bool {
    let mut points = Vec::new();
    if let Some(a) = b["box"].as_array() {
        let p = |i: usize| {
            DVec3::new(
                a[i].as_f64().unwrap(),
                a[i + 1].as_f64().unwrap(),
                a[i + 2].as_f64().unwrap(),
            )
        };
        let (c, x, y, z) = (p(0), p(3), p(6), p(9));
        for sx in [-1., 1.] {
            for sy in [-1., 1.] {
                for sz in [-1., 1.] {
                    points.push((local * transform).transform_point3(c + sx * x + sy * y + sz * z));
                }
            }
        }
    } else if let Some(a) = b["region"].as_array() {
        for lon in [a[0].as_f64().unwrap(), a[2].as_f64().unwrap()] {
            for lat in [a[1].as_f64().unwrap(), a[3].as_f64().unwrap()] {
                for h in [a[4].as_f64().unwrap(), a[5].as_f64().unwrap()] {
                    points.push(local.transform_point3(ecef(
                        lon.to_degrees(),
                        lat.to_degrees(),
                        h,
                    )));
                }
            }
        }
    } else if let Some(a) = b["sphere"].as_array() {
        let m = local * transform;
        let c = m.transform_point3(DVec3::new(
            a[0].as_f64().unwrap(),
            a[1].as_f64().unwrap(),
            a[2].as_f64().unwrap(),
        ));
        let r = a[3].as_f64().unwrap()
            * transform
                .x_axis
                .truncate()
                .length()
                .max(transform.y_axis.truncate().length())
                .max(transform.z_axis.truncate().length());
        return c.x + r >= -HALF && c.x - r <= HALF && c.z + r >= -HALF && c.z - r <= HALF;
    } else {
        panic!("Unsupported bounding volume");
    }
    let lo = points.iter().copied().reduce(DVec3::min).unwrap();
    let hi = points.iter().copied().reduce(DVec3::max).unwrap();
    hi.x >= -HALF && lo.x <= HALF && hi.z >= -HALF && lo.z <= HALF
}

fn content_url(base: &Url, uri: &str) -> Result<Url> {
    let mut url = base.join(uri)?;
    if url.host_str() != Some("tile.googleapis.com") {
        return Err("Tile content has an unexpected origin".into());
    }
    if !url.query_pairs().any(|(k, _)| k == "session") {
        if let Some((_, v)) = base.query_pairs().find(|(k, _)| k == "session") {
            url.query_pairs_mut().append_pair("session", &v);
        }
    }
    Ok(url)
}

async fn get(client: &Client, url: &Url, key: &str) -> Result<Vec<u8>> {
    let mut url = url.clone();
    url.query_pairs_mut().append_pair("key", key);
    // Remove request URLs from errors: they contain the API key and session.
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|_| "Tile request failed")?;
    if !response.status().is_success() {
        return Err(format!("Tile HTTP {}", response.status().as_u16()).into());
    }
    Ok(response
        .bytes()
        .await
        .map_err(|_| "Tile body failed")?
        .to_vec())
}

fn glb_json(bytes: &[u8]) -> Result<Value> {
    let len = u32::from_le_bytes(bytes[12..16].try_into()?) as usize;
    Ok(serde_json::from_slice(&bytes[20..20 + len])?)
}

fn write_glb(json: &Value, bytes: &[u8]) -> Vec<u8> {
    let old = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let mut j = serde_json::to_vec(json).unwrap();
    while j.len() % 4 != 0 {
        j.push(b' ');
    }
    let tail = &bytes[20 + old..];
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&((20 + j.len() + tail.len()) as u32).to_le_bytes());
    out.extend_from_slice(&(j.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend(j);
    out.extend(tail);
    out
}

async fn prepare(
    client: Client,
    key: String,
    url: Url,
    output: PathBuf,
    id: usize,
    transform: DMat4,
) -> Result<(Value, Vec<String>, usize, usize)> {
    let bytes = get(&client, &url, &key).await?;
    let name = format!("{id:06}");
    let source = output.join("source").join(format!("{name}.glb"));
    fs::write(&source, &bytes)?;
    let mut doc = glb_json(&bytes)?;
    let copyrights = doc["asset"]["copyright"]
        .as_str()
        .unwrap_or("")
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect::<Vec<_>>();
    let extensions = doc["extensionsRequired"].clone();
    // Resolve companion images and buffers beside the saved source before import.
    let mut downloaded = bytes.len();
    for kind in ["buffers", "images"] {
        if let Some(resources) = doc[kind].as_array_mut() {
            for (i, r) in resources.iter_mut().enumerate() {
                if let Some(uri) = r["uri"].as_str().filter(|s| !s.starts_with("data:")) {
                    let resolved = content_url(&url, uri)?;
                    let resource = get(&client, &resolved, &key).await?;
                    downloaded += resource.len();
                    let ext = Path::new(resolved.path())
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("bin");
                    let local = format!("{name}-{kind}-{i}.{ext}");
                    fs::write(output.join("source").join(&local), &resource)?;
                    fs::write(output.join("tiles").join(&local), resource)?;
                    r["uri"] = json!(local);
                }
            }
        }
    }
    let import_path = output.join("source").join(format!("{name}-local.glb"));
    fs::write(&import_path, write_glb(&doc, &bytes))?;
    let mut imported = draco_gltf::import(&import_path)?;
    let mut primitives = Vec::new();
    for (m, mesh) in doc["meshes"].as_array().unwrap().iter().enumerate() {
        for (p, primitive) in mesh["primitives"].as_array().unwrap().iter().enumerate() {
            if primitive["extensions"]["KHR_draco_mesh_compression"].is_object() {
                primitives.push(draco_gltf::PrimitiveIndex::new(draco_gltf::MeshIndex(m), p));
            }
        }
    }
    for p in &primitives {
        let geometry = imported.read_primitive(*p)?;
        imported.write_primitive(
            *p,
            &geometry,
            draco_gltf::GeometryWriteOptions {
                encoding: draco_gltf::GeometryEncoding::Raw,
            },
        )?;
    }
    let ready = imported.to_bytes(draco_gltf::OutputFormat::GlbV2)?;
    let mut ready_doc = glb_json(&ready)?;
    // Earth-scale node placement must cancel in f64 BEFORE Bevy reads it as f32.
    let placement = transform * DMat4::from_rotation_x(std::f64::consts::FRAC_PI_2);
    let roots: HashSet<usize> = ready_doc["scenes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|s| {
            s["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|n| n.as_u64().unwrap() as usize)
        })
        .collect();
    for root in roots {
        let m = placement * node_matrix(&ready_doc["nodes"][root]);
        let node = ready_doc["nodes"][root].as_object_mut().unwrap();
        node.remove("translation");
        node.remove("rotation");
        node.remove("scale");
        node.insert("matrix".to_string(), json!(m.to_cols_array()));
    }
    let ready = write_glb(&ready_doc, &ready);
    fs::write(output.join("tiles").join(format!("{name}.glb")), &ready)?;
    println!(
        "Prepared tile {id}: input={} output={} Draco primitives={} required={extensions}",
        bytes.len(),
        ready.len(),
        primitives.len()
    );
    let vertices: usize = ready_doc["meshes"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["primitives"].as_array().unwrap())
        .map(|p| {
            ready_doc["accessors"][p["attributes"]["POSITION"].as_u64().unwrap() as usize]["count"]
                .as_u64()
                .unwrap() as usize
        })
        .sum();
    Ok((
        json!({"path":format!("tiles/{name}.glb"),"transform":DMat4::IDENTITY.to_cols_array(),"vertices":vertices,"placement_baked_into_scene_roots":true,"required_extensions":ready_doc["extensionsRequired"],"source_required_extensions":extensions}),
        copyrights,
        downloaded,
        ready.len(),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let started = Instant::now();
    let args: Vec<_> = std::env::args().collect();
    let output = PathBuf::from(args.get(1).ok_or(
        "usage: capture_demo_tiles NEW_OUTPUT_DIRECTORY --site mut|monumental [--one-tile]",
    )?);
    let site = args
        .windows(2)
        .find(|a| a[0] == "--site")
        .map(|a| a[1].as_str())
        .ok_or("--site mut|monumental is required")?;
    if !matches!(site, "mut" | "monumental") {
        return Err("site must be mut or monumental".into());
    }
    let one = args.iter().any(|s| s == "--one-tile");
    fs::create_dir(&output)?;
    fs::create_dir(output.join("source"))?;
    fs::create_dir(output.join("tiles"))?;
    fs::write(output.join("google.png"), include_bytes!("../google.png"))?;
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (lon, lat, height) = anchor(&repo, site)?;
    let local = earth_to_local(lon, lat, height);
    let key = std::env::var("GOOGLE_MAPS_API_KEY")?;
    let client = Client::new();
    let root_url = Url::parse(ROOT)?;
    let root_bytes = get(&client, &root_url, &key).await?;
    fs::write(output.join("source/root.json"), &root_bytes)?;
    let root: Value = serde_json::from_slice(&root_bytes)?;
    let mut queue = vec![(
        root["root"].clone(),
        root_url,
        DMat4::IDENTITY,
        "REPLACE".to_string(),
    )];
    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let mut json_bytes = root_bytes.len();
    let mut tileset_count = 1;
    while let Some((tile, base, parent, inherited_refine)) = queue.pop() {
        let transform = parent * matrix(&tile["transform"]);
        if !intersects(&tile["boundingVolume"], transform, local) {
            continue;
        }
        let refine = tile["refine"]
            .as_str()
            .unwrap_or(&inherited_refine)
            .to_string();
        let children = tile["children"].as_array().cloned().unwrap_or_default();
        let descend = tile["geometricError"].as_f64().unwrap_or(0.) > ERROR && !children.is_empty();
        let mut contents = tile["contents"].as_array().cloned().unwrap_or_default();
        if tile["content"].is_object() {
            contents.push(tile["content"].clone());
        }
        for content in contents {
            let uri = content["uri"]
                .as_str()
                .or_else(|| content["url"].as_str())
                .unwrap();
            let url = content_url(&base, uri)?;
            if url.path().ends_with(".json") {
                let raw = get(&client, &url, &key).await?;
                json_bytes += raw.len();
                fs::write(
                    output
                        .join("source")
                        .join(format!("tileset-{tileset_count:06}.json")),
                    &raw,
                )?;
                tileset_count += 1;
                let external: Value = serde_json::from_slice(&raw)?;
                queue.push((external["root"].clone(), url, transform, refine.clone()));
            } else if !descend || refine == "ADD" {
                if seen.insert(url.to_string()) {
                    selected.push((
                        url,
                        local * transform,
                        center_distance(&tile["boundingVolume"], transform, local),
                    ));
                }
            }
        }
        if descend {
            for child in children.into_iter().rev() {
                queue.push((child, base.clone(), transform, refine.clone()));
            }
        }
    }
    println!(
        "Selected {} tiles from {tileset_count} tilesets; anchor {lon},{lat},{height}",
        selected.len()
    );
    if selected.is_empty() {
        return Err("No tiles selected for this site".into());
    }
    selected.sort_by(|a, b| a.2.total_cmp(&b.2));
    let mut rows = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();
    let mut downloaded = json_bytes;
    let mut prepared = 0;
    // One-tile mode tries central candidates until it finds substantial geometry.
    // This avoids accepting a tiny detached surface fragment as the site gate.
    let mut attempts = 0;
    for chunk in selected.chunks(if one { 1 } else { 4 }) {
        let mut jobs = Vec::new();
        for (url, transform, _) in chunk {
            let id = attempts + jobs.len();
            jobs.push(tokio::spawn(prepare(
                client.clone(),
                key.clone(),
                url.clone(),
                output.clone(),
                id,
                *transform,
            )));
        }
        for job in jobs {
            let (row, credits, a, b) = job.await??;
            attempts += 1;
            downloaded += a;
            prepared += b;
            if one && row["vertices"].as_u64().unwrap() < 1000 {
                continue;
            }
            rows.push(row);
            for c in credits {
                *counts.entry(c).or_default() += 1;
            }
        }
        if one && !rows.is_empty() {
            break;
        }
    }
    if rows.is_empty() {
        return Err("No representative tile with at least1000 vertices found".into());
    }
    let mut credits: Vec<_> = counts.into_iter().collect();
    credits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let attribution: Vec<_> = credits.iter().map(|(s, _)| s.clone()).collect();
    fs::write(
        output.join("ATTRIBUTION.txt"),
        format!("Google Maps\n{}\n", attribution.join("; ")),
    )?;
    let source_object = if site == "mut" {
        "relation/18760408"
    } else {
        "relation/14111047"
    };
    let manifest = json!({"site":site,"anchor_source":{"file":"tools/pack/named_places.py","osm_object":source_object,"method":"existing OSM object centre"},"anchor":{"longitude":lon,"latitude":lat,"height_m":height,"height_datum":"existing Copernicus DSM EGM2008; used as local reference, not converted to ellipsoid"},"coordinate_system":"X east, Y up, Z south","transform_space":"identity outer placement; scene roots already in local metres after f64 Earth and axis conversion","bounds":{"half_extent_m":HALF},"max_geometric_error_m":ERROR,"captured_at_unix_s":SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),"attribution":attribution,"google_logo":"google.png","tiles":rows,"download_bytes":downloaded,"prepared_glb_bytes":prepared,"capture_seconds":started.elapsed().as_secs_f64(),"one_tile_gate":one});
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    println!(
        "Saved {} tiles; downloaded {downloaded} bytes; prepared {prepared} bytes; capture {:.2}s",
        rows.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
