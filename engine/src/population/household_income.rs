use super::{Person, UNKNOWN};
use serde::Serialize;

/// CASEN 2024 RM worker weights (`expr`) of the four bands, by home comuna, in the order of `cuts`
/// (`data/mvp/casen_worker_income.csv`, tools/pack/casen_worker_income.py).
pub fn comuna_weights(dir: &std::path::Path, cuts: &[String]) -> Vec<[u32; 4]> {
    let mut rows = std::collections::HashMap::new();
    for r in csv::Reader::from_path(dir.join("casen_worker_income.csv")).unwrap().records().map(|r| r.unwrap()) {
        rows.insert(r[0].to_string(), [1, 2, 3, 4].map(|i| r[i].parse::<u32>().unwrap()));
    }
    cuts.iter().map(|c| rows[c]).collect()
}

// Whole-household income, in November 2024 Chilean pesos.
const LOWER_BOUNDS: [u32; 4] = [0, 745_001, 1_312_768, 2_323_475];
const UPPER_BOUNDS: [Option<u32>; 4] = [Some(745_000), Some(1_312_767), Some(2_323_474), None];

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum HouseholdIncome {
    Imputed {
        band: u8,
        monthly_clp_min: u32,
        monthly_clp_max: Option<u32>,
        price_month: &'static str,
        region_cut: u8,
        source: &'static str,
        shared_membership: &'static str,
    },
    Unmodeled {
        reason: &'static str,
    },
}

impl Person {
    /// Use the same baseline person index for both sides of a paired replay.
    /// `weights`: CASEN band weights of the home comuna (`comuna_weights`).
    pub fn household_income(&self, baseline_index: usize, weights: &[u32; 4]) -> HouseholdIncome {
        if self.household_size == UNKNOWN {
            return HouseholdIncome::Unmodeled { reason: "Private-household membership is unknown." };
        }
        if self.student || !(1..=4).contains(&self.age_band) {
            return HouseholdIncome::Unmodeled { reason: "The regional input covers workers aged 15 and older." };
        }

        // SplitMix64 with an income-specific seed. The home block is not a household key.
        let mut draw = (baseline_index as u64).wrapping_add(0x8C59_3450_D8DC_A2EB);
        draw = (draw ^ (draw >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        draw = (draw ^ (draw >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        draw ^= draw >> 31;
        let total = weights.iter().sum::<u32>();
        let rank = (draw % u64::from(total)) as u32;
        let band = weights.iter()
            .scan(0, |sum, weight| { *sum += weight; Some(*sum) })
            .take_while(|&upper| rank >= upper)
            .count();

        HouseholdIncome::Imputed {
            band: (band + 1) as u8,
            monthly_clp_min: LOWER_BOUNDS[band],
            monthly_clp_max: UPPER_BOUNDS[band],
            price_month: "2024-11",
            region_cut: 13,
            source: "CASEN 2024 ymonecorh, RM employed-person weights (expr) of the home comuna; commune is not a CASEN survey domain",
            shared_membership: "unmodeled",
        }
    }
}
