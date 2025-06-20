use serde::Serialize;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
};

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

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)  // we'll manage headers ourselves
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
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let json = serde_json::to_string(entry)?;
    writeln!(file, "{}", json)?;
    Ok(())
}
