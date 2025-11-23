use std::collections::HashMap;

use anyhow::Result;
use csv::WriterBuilder;
use ordered_float::OrderedFloat;

use crate::DistanceRecord;

pub fn write_csv(saved_distances: &HashMap<usize, DistanceRecord>) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path("output.csv")?;

    let mut values: Vec<&DistanceRecord> = saved_distances
        .values()
        .collect();

    values.sort_by_key(|r| OrderedFloat(r.length));

    for record in  values {
        wtr.serialize(record)?;
    }

    wtr.flush()?;

    Ok(())
}