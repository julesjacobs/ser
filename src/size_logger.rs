use crate::kleene::SMART_ORDER;
use crate::reachability::BIDIRECTIONAL_PRUNING_ENABLED;
use crate::semilinear::{GENERATE_LESS, REMOVE_REDUNDANT};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::{fs, fs::OpenOptions, io::Write, path::Path};

#[derive(Serialize)]
pub struct PetriNetSize {
    /// Identifier so you know which disjunct / iteration this came from
    pub disjunct_id: usize,
    /// “pre_pruning” or “post_pruning”
    pub stage: &'static str,
    /// Number of places in the net at this point
    pub num_places: usize,
    /// Number of transitions in the net at this point
    pub num_transitions: usize,
}

/// Append a record to a CSV file (no headers, so you can keep appending)
pub fn log_size_csv(path: &Path, entry: &PetriNetSize) -> Result<(), std::io::Error> {
    // Decide if we need to write headers: either file doesn't exist yet,
    // or it exists but is zero‐length.
    let need_header = match path.metadata() {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    };

    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false) // we'll manage headers ourselves
        .from_writer(file);

    if need_header {
        // write a top row naming each column
        wtr.write_record(&["index", "stage", "num_places", "num_transitions"])?;
    }

    // now append your entry
    wtr.serialize(entry)?;
    wtr.flush()?;
    Ok(())
}

/// Append a record to a JSON‐lines file (one JSON object per line)
pub fn log_size_json(path: &Path, entry: &PetriNetSize) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let json = serde_json::to_string(entry)?;
    writeln!(file, "{}", json)?;
    Ok(())
}

/// Log entry for SemilinearSet statistics
#[derive(Serialize)]
pub struct SemilinearStats {
    // Name of the benchmark
    pub program_name: String,
    /// Number of linear-set components
    pub num_components: usize,
    /// Number of period vectors per component
    pub periods_per_component: Vec<usize>,
}

/// Append a record to a CSV file for SemilinearSet statistics
pub fn log_semilinear_csv(path: &Path, entry: &SemilinearStats) -> Result<(), std::io::Error> {
    // Ensure output directory exists
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let file = OpenOptions::new().create(true).append(true).open(path)?;

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    let need_header = match path.metadata() {
        Ok(meta) => meta.len() == 0,
        Err(_) => true,
    };
    if need_header {
        wtr.write_record(&[
            "bidirectional_pruning ON",
            "remove_redundant ON",
            "generate_less ON",
            "smart_order ON",
            "benchmark",
            "num_components",
            "max_periods_per_component",
            "avg_periods_per_component",
        ])?;
    }

    let mut record = Vec::new();

    // read each flag and push "1"/"0"
    let bidir_pruning = BIDIRECTIONAL_PRUNING_ENABLED.load(Ordering::Relaxed);
    let remove_redundant = REMOVE_REDUNDANT.load(Ordering::Relaxed);
    let generate_less = GENERATE_LESS.load(Ordering::Relaxed);
    let smart_order = SMART_ORDER.load(Ordering::Relaxed);
    record.push(if bidir_pruning { "1" } else { "0" }.to_string());
    record.push(if remove_redundant { "1" } else { "0" }.to_string());
    record.push(if generate_less { "1" } else { "0" }.to_string());
    record.push(if smart_order { "1" } else { "0" }.to_string());

    record.push(entry.program_name.to_string());
    record.push(entry.num_components.to_string());

    let max_period = entry
        .periods_per_component
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
    record.push(max_period.to_string());

    let avg_period = if entry.periods_per_component.is_empty() {
        0.0
    } else {
        let sum: usize = entry.periods_per_component.iter().sum();
        sum as f64 / (entry.periods_per_component.len() as f64)
    };
    record.push(format!("{:.2}", avg_period)); // 2-decimal precision

    wtr.write_record(&record)?;
    wtr.flush()?;
    Ok(())
}
