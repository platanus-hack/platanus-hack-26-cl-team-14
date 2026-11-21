use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, fs::File, path::Path};

const TOLERANCE: f64 = 1e-10;
const MAX_ITERATIONS: usize = 10_000;

#[derive(Deserialize)]
struct Cell {
    origin_cut: String,
    destination_cut: String,
    workers: f64,
}

struct Margins {
    rows: Vec<f64>,
    columns: Vec<f64>,
}

struct Fit {
    predicted: Vec<f64>,
    iterations: usize,
}

struct Unavailable {
    reason: String,
    iterations: usize,
}

#[derive(Serialize)]
struct Residuals {
    max_row_absolute: f64,
    max_row_relative: f64,
    max_column_absolute: f64,
    max_column_relative: f64,
}

#[derive(Serialize)]
struct Metrics {
    cpc: f64,
    summed_absolute_error: f64,
    predicted_total: f64,
    margin_residuals: Residuals,
}

#[derive(Serialize)]
struct Method {
    name: &'static str,
    status: &'static str,
    unavailable_reason: Option<String>,
    iteration_count: usize,
    predicted: Option<Vec<f64>>,
    metrics: Option<Metrics>,
}

fn load(path: &Path, cuts: &[String], pairs: &[(usize, usize)]) -> Vec<f64> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    let cells: HashMap<_, _> = reader
        .deserialize::<Cell>()
        .map(|row| {
            let cell = row.unwrap();
            ((cell.origin_cut, cell.destination_cut), cell.workers)
        })
        .collect();
    pairs
        .iter()
        .map(|&(i, j)| cells[&(cuts[i].clone(), cuts[j].clone())])
        .collect()
}

fn margins(values: &[f64], pairs: &[(usize, usize)], size: usize) -> Margins {
    let mut result = Margins {
        rows: vec![0.0; size],
        columns: vec![0.0; size],
    };
    for (&value, &(i, j)) in values.iter().zip(pairs) {
        result.rows[i] += value;
        result.columns[j] += value;
    }
    result
}

fn residuals(predicted: &Margins, observed: &Margins) -> Residuals {
    let maximum = |values: &[f64], targets: &[f64], relative: bool| {
        values
            .iter()
            .zip(targets)
            .map(|(value, target)| {
                (value - target).abs() / if relative { target.max(1.0) } else { 1.0 }
            })
            .fold(0.0, f64::max)
    };
    Residuals {
        max_row_absolute: maximum(&predicted.rows, &observed.rows, false),
        max_row_relative: maximum(&predicted.rows, &observed.rows, true),
        max_column_absolute: maximum(&predicted.columns, &observed.columns, false),
        max_column_relative: maximum(&predicted.columns, &observed.columns, true),
    }
}

fn persistence(
    source: &[f64],
    pairs: &[(usize, usize)],
    conditions: &Margins,
) -> Result<Fit, Unavailable> {
    let baseline = margins(source, pairs, conditions.rows.len());
    if baseline.rows.contains(&0.0) {
        return Err(Unavailable {
            reason: "Zero source origin denominator".into(),
            iterations: 0,
        });
    }
    Ok(Fit {
        predicted: source
            .iter()
            .zip(pairs)
            .map(|(&value, &(i, _))| conditions.rows[i] * value / baseline.rows[i])
            .collect(),
        iterations: 0,
    })
}

fn sweep(
    predicted: &mut [f64],
    pairs: &[(usize, usize)],
    conditions: &Margins,
) -> Result<(), String> {
    let columns = margins(predicted, pairs, conditions.rows.len()).columns;
    if columns.contains(&0.0) {
        return Err("Zero column-scaling denominator".into());
    }
    for (value, &(_, j)) in predicted.iter_mut().zip(pairs) {
        *value *= conditions.columns[j] / columns[j];
    }
    let rows = margins(predicted, pairs, conditions.rows.len()).rows;
    if rows.contains(&0.0) {
        return Err("Zero row-scaling denominator".into());
    }
    for (value, &(i, _)) in predicted.iter_mut().zip(pairs) {
        *value *= conditions.rows[i] / rows[i];
    }
    Ok(())
}

fn column_then_row(
    source: &[f64],
    pairs: &[(usize, usize)],
    conditions: &Margins,
) -> Result<Fit, Unavailable> {
    let mut predicted = source.to_vec();
    sweep(&mut predicted, pairs, conditions).map_err(|reason| Unavailable {
        reason,
        iterations: 1,
    })?;
    Ok(Fit {
        predicted,
        iterations: 1,
    })
}

fn ipf(source: &[f64], pairs: &[(usize, usize)], conditions: &Margins) -> Result<Fit, Unavailable> {
    let mut predicted = source.to_vec();
    for iteration in 1..=MAX_ITERATIONS {
        sweep(&mut predicted, pairs, conditions).map_err(|reason| Unavailable {
            reason,
            iterations: iteration,
        })?;
        let residual = residuals(
            &margins(&predicted, pairs, conditions.rows.len()),
            conditions,
        );
        if residual.max_row_relative.max(residual.max_column_relative) <= TOLERANCE {
            return Ok(Fit {
                predicted,
                iterations: iteration,
            });
        }
    }
    Err(Unavailable {
        reason: format!(
            "IPF did not converge to relative margin tolerance {TOLERANCE} in {MAX_ITERATIONS} iterations; support was not changed"
        ),
        iterations: MAX_ITERATIONS,
    })
}

fn evaluate(
    name: &'static str,
    fit: Result<Fit, Unavailable>,
    target: &[f64],
    pairs: &[(usize, usize)],
    conditions: &Margins,
) -> Method {
    match fit {
        Ok(fit) => {
            let predicted_total: f64 = fit.predicted.iter().sum();
            let target_total: f64 = target.iter().sum();
            let overlap: f64 = fit
                .predicted
                .iter()
                .zip(target)
                .map(|(p, t)| p.min(*t))
                .sum();
            let absolute_error = fit
                .predicted
                .iter()
                .zip(target)
                .map(|(p, t)| (p - t).abs())
                .sum();
            let margin_residuals = residuals(
                &margins(&fit.predicted, pairs, conditions.rows.len()),
                conditions,
            );
            Method {
                name,
                status: "available",
                unavailable_reason: None,
                iteration_count: fit.iterations,
                metrics: Some(Metrics {
                    cpc: 2.0 * overlap / (predicted_total + target_total),
                    summed_absolute_error: absolute_error,
                    predicted_total,
                    margin_residuals,
                }),
                predicted: Some(fit.predicted),
            }
        }
        Err(error) => Method {
            name,
            status: "unavailable",
            unavailable_reason: Some(error.reason),
            iteration_count: error.iterations,
            predicted: None,
            metrics: None,
        },
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let directory = args.next().expect("Pass the census_2002 data directory");
    let output = args.next().expect("Pass an output JSON path");
    let directory = Path::new(&directory);
    let declared: Value =
        serde_json::from_reader(File::open(directory.join("comparison_manifest.json")).unwrap())
            .unwrap();
    let cuts: Vec<String> = serde_json::from_value(declared["domain"]["cuts"].clone()).unwrap();
    let pairs: Vec<_> = (0..cuts.len())
        .flat_map(|i| {
            (0..cuts.len())
                .filter(move |&j| i != j)
                .map(move |j| (i, j))
        })
        .collect();
    let source = load(&directory.join("intercommunal_2002.csv"), &cuts, &pairs);
    let target = load(&directory.join("intercommunal_2024.csv"), &cuts, &pairs);
    let conditions = margins(&target, &pairs, cuts.len());

    // Pair values stay here; allocation functions receive only source cells and observed margins.
    let fits = [
        ("P_persistence", persistence(&source, &pairs, &conditions)),
        (
            "C_column_then_row",
            column_then_row(&source, &pairs, &conditions),
        ),
        ("IPF", ipf(&source, &pairs, &conditions)),
    ];
    let methods: Vec<_> = fits
        .into_iter()
        .map(|(name, fit)| evaluate(name, fit, &target, &pairs, &conditions))
        .collect();
    let target_total: f64 = target.iter().sum();
    let unsupported_mass: f64 = source
        .iter()
        .zip(&target)
        .filter(|(s, _)| **s == 0.0)
        .map(|(_, t)| t)
        .sum();
    let pair_order: Vec<_> = pairs.iter().map(|&(i, j)| [&cuts[i], &cuts[j]]).collect();
    let result = json!({
        "experiment": "2002_to_2024_conditional_intercommunal_allocation",
        "scope": "Retrospective allocation conditional on observed 2024 margins; no causal, future-margin or actual API validation claim",
        "fixed_parameters": {"ipf_tolerance": TOLERANCE, "ipf_max_iterations": MAX_ITERATIONS,
            "sweep_order": "columns_then_rows", "support_policy": "preserve_F0_zeros", "smoothing": false},
        "pair_order": pair_order,
        "source_total": source.iter().sum::<f64>(), "target_total": target_total,
        "observed_conditions": {"cut_order": cuts, "origin_margins": conditions.rows, "destination_margins": conditions.columns},
        "support": {"source_zero_cells": source.iter().filter(|&&s| s == 0.0).count(),
            "target_workers_on_source_zero_cells": unsupported_mass, "target_mass_fraction_on_source_zero_cells": unsupported_mass / target_total,
            "new_positive_target_edges": source.iter().zip(&target).filter(|(s, t)| **s == 0.0 && **t > 0.0).count()},
        "methods": methods,
        "declared_input_provenance": declared,
        "provenance_warning": "Manifest hashes are declared, not runtime verified. Record actual input, code and output hashes separately.",
        "limitations": [
            "Observed target margins are supplied conditions, not predicted outcomes.",
            "P ignores destination margins; C is the first IPF sweep, not a distinct allocation law. IPF fits extra column constraints.",
            "All 2652 off-diagonal cells remain in the domain; no new support, target renormalization or support repair.",
            "2002 home workers cannot be separated within the excluded diagonal; 2024 excludes work-at-home. Full-matrix cohort equivalence is not established.",
            "Unequal nonresponse, employed-student assumptions and multiple/no-fixed workplaces remain comparability limits.",
            "Repeated population snapshots do not identify relocation or a housing/job policy response."
        ]
    });
    serde_json::to_writer_pretty(File::create(output).unwrap(), &result).unwrap();
}
