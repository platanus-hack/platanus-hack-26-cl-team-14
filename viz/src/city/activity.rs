// Both building levels of detail share the same block dictionary.
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::Read, path::Path};

pub struct HeightScales {
    offsets: Vec<usize>,
    keys: Vec<u16>,
    dictionary: Vec<(u64, u16)>,
    scales: Vec<f32>,
    previous: Vec<f32>,
    pub started: f32,
    revision: Option<u64>,
}

impl HeightScales {
    pub fn load(path: &Path) -> Self {
        let mut bytes = Vec::new();
        GzDecoder::new(File::open(path).unwrap())
            .read_to_end(&mut bytes)
            .unwrap();
        let word = |p| u16::from_le_bytes(bytes[p..p + 2].try_into().unwrap());
        let integer = |p| u32::from_le_bytes(bytes[p..p + 4].try_into().unwrap()) as usize;
        let tile_count = usize::from(word(4)) * usize::from(word(6));
        let offsets = (0..=tile_count).map(|i| integer(16 + i * 4)).collect();
        let dict_at = 16 + (tile_count + 1) * 4;
        let n = integer(8);
        let dictionary = (0..n)
            .map(|i| {
                let p = dict_at + i * 10;
                (
                    u64::from_le_bytes(bytes[p..p + 8].try_into().unwrap()),
                    word(p + 8),
                )
            })
            .collect();
        let keys_at = dict_at + n * 10;
        let keys = (0..integer(12)).map(|i| word(keys_at + i * 2)).collect();
        Self {
            offsets,
            keys,
            dictionary,
            scales: vec![1.0; n],
            previous: vec![1.0; n],
            started: -10.,
            revision: None,
        }
    }

    pub fn scale(&self, tile: usize, building: usize) -> (f32, f32) {
        let key = self.keys[self.offsets[tile] + building];
        if key == u16::MAX {
            (1., 1.)
        } else {
            let k = usize::from(key);
            (self.previous[k], self.scales[k])
        }
    }

    pub fn progress(&self, now: f32) -> f32 {
        let t = ((now - self.started) / 1.2).clamp(0., 1.);
        t * t * (3. - 2. * t)
    }

    // Every engine publication is a complete scenario relative to its own baseline.
    // 'Some' means the render caller should invalidate its cached tile meshes.
    // The optional coordinates are set only for a later revision with mapped changes.
    pub fn update(&mut self, result: &Value, now: f32) -> Option<Option<[f64; 2]>> {
        let revision = result["revision"].as_u64()?;
        if self.revision == Some(revision) {
            return None;
        }
        let t = self.progress(now);
        for (from, to) in self.previous.iter_mut().zip(&self.scales) {
            *from += (*to - *from) * t;
        }
        self.started = now;
        let has_previous = self.revision.replace(revision).is_some();
        let rows = |key: &str| result[key].as_array().into_iter().flatten();
        let mut rules = Vec::new();
        for change in rows("applied") {
            let kind = change["kind"].as_str().unwrap_or("");
            if !matches!(kind, "jobs" | "residents") {
                continue;
            }
            let target = change["target"]["type"].as_str().unwrap_or("");
            let Some(id) = numeric_id(&change["target"]["id"]) else {
                continue;
            };
            if !matches!(target, "block" | "comuna") {
                continue;
            }
            let note = &change["understood"];
            let before = note[format!("{kind}_before")].as_f64();
            let after = note[format!("{kind}_after")].as_f64();
            if let (Some(before), Some(after)) = (before, after)
                && before != after
            {
                // Height shows a relative activity change. Keep roofs visible at zero.
                let scale = if before > 0.0 {
                    (after / before).clamp(0.35, 3.0) as f32
                } else {
                    1.25
                };
                rules.push((target, id, scale));
            }
        }

        let changed: HashMap<u64, &Value> = rows("blocks_changed")
            .filter_map(|v| Some((numeric_id(&v["id"])?, v)))
            .collect();
        let mut focus = None;
        let mut matched = 0;
        let mut unresolved = 0;
        let mut mapped_comunas = std::collections::HashSet::new();
        for (i, &(block, cut)) in self.dictionary.iter().enumerate() {
            let mut matches = rules.iter().filter(|&&(target, id, _)| {
                if target == "block" {
                    id == block
                } else {
                    id == u64::from(cut)
                }
            });
            let single = matches.next();
            let multiple = matches.next().is_some();
            let scale = if result["blocks_changed_total"].as_u64() == Some(0) {
                1.0
            } else if let Some(&(_, _, scale)) = single
                && !multiple
            {
                scale
            } else if let Some(row) = changed.get(&block) {
                // No denominator is available in these net block rows. Show only direction.
                // Opposite changes in the two measures cannot share a single height.
                let jobs = row["jobs_delta"].as_f64().unwrap_or(0.0);
                let residents = row["residents_delta"].as_f64().unwrap_or(0.0);
                if jobs * residents < 0.0 {
                    unresolved += 1;
                    1.0
                } else if jobs > 0.0 || residents > 0.0 {
                    1.25
                } else if jobs < 0.0 || residents < 0.0 {
                    0.75
                } else {
                    1.0
                }
            } else {
                if multiple {
                    unresolved += 1;
                }
                1.0
            };
            self.scales[i] = scale;
            if scale != 1.0 {
                matched += 1;
                mapped_comunas.insert(u64::from(cut));
                if focus.is_none() {
                    focus = changed.get(&block).and_then(|row| location(row));
                }
            }
        }
        // A comuna-wide change can exceed the 300 block rows returned by the engine.
        // Prefer the comuna's coordinates over a single manzana in that case.
        if let Some(place) = rules.iter().find_map(|&(target, id, _)| {
            if target != "comuna" || !mapped_comunas.contains(&id) {
                return None;
            }
            rows("zones")
                .find(|z| numeric_id(&z["id"]) == Some(id))
                .and_then(location)
        }) {
            focus = Some(place);
        }

        eprintln!(
            "Scenario {revision}: activity height changed in {matched} assigned blocks; {unresolved} overlapping or conflicting blocks kept baseline. Height ratio 0.35–3; net changes without a base use 0.75/1.25."
        );
        if !has_previous {
            self.previous.copy_from_slice(&self.scales);
        }
        Some(if has_previous { focus } else { None })
    }
}

fn numeric_id(value: &Value) -> Option<u64> {
    value.as_str()?.parse().ok()
}
fn location(value: &Value) -> Option<[f64; 2]> {
    Some([value["longitude"].as_f64()?, value["latitude"].as_f64()?])
}
