// Both building levels of detail share the same block dictionary.
use flate2::read::GzDecoder;
use serde_json::Value;
use std::{collections::HashMap, io::Read, path::Path};

pub struct HeightScales {
    offsets: Vec<usize>,
    keys: Vec<u16>,
    dictionary: Vec<(u64, u16)>,
    scales: Vec<f32>,
    changed: Vec<bool>,
    previous: Vec<f32>,
    pub started: f32,
    revision: Option<u64>,
}

impl HeightScales {
    pub fn load(path: &Path) -> Self {
        let mut bytes = Vec::new();
        GzDecoder::new(crate::data::open(path).unwrap())
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
            changed: vec![false; n],
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

    pub fn tile_changed(&self, tile: usize) -> bool {
        self.keys[self.offsets[tile]..self.offsets[tile + 1]]
            .iter()
            .any(|&k| k != u16::MAX && self.changed[k as usize])
    }

    pub fn progress(&self, now: f32) -> f32 {
        let t = ((now - self.started) / 1.2).clamp(0., 1.);
        t * t * (3. - 2. * t)
    }

    // Every engine publication is a complete scenario relative to its own baseline.
    pub fn update(&mut self, result: &Value, now: f32) -> bool {
        let Some(revision) = result["revision"].as_u64() else {
            return false;
        };
        if self.revision == Some(revision) {
            return false;
        }
        let t = self.progress(now);
        if t == 1.0 {
            self.previous.copy_from_slice(&self.scales);
        } else {
            for (from, to) in self.previous.iter_mut().zip(&self.scales) {
                *from += (*to - *from) * t;
            }
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
        let mut matched = 0;
        let mut unresolved = 0;
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
            self.changed[i] = self.scales[i] != scale || self.previous[i] != scale;
            self.scales[i] = scale;
            if scale != 1.0 {
                matched += 1;
            }
        }
        eprintln!(
            "Scenario {revision}: activity height changed in {matched} assigned blocks; {unresolved} overlapping or conflicting blocks kept baseline. Height ratio 0.35–3; net changes without a base use 0.75/1.25."
        );
        if !has_previous {
            self.previous.copy_from_slice(&self.scales);
        }
        true
    }
}

fn numeric_id(value: &Value) -> Option<u64> {
    value.as_str()?.parse().ok()
}
