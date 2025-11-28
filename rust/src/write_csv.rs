use std::collections::HashMap;

use anyhow::Result;
use csv::WriterBuilder;
use ordered_float::OrderedFloat;

use crate::DistanceRecord;

pub fn write_csv(saved_distances: Vec<DistanceRecord>) -> Result<()> {
    let mut wtr = WriterBuilder::new()
        .has_headers(true)
        .from_path("output.csv")?;
    
    let mut values = saved_distances.clone().into_iter().collect::<Vec<DistanceRecord>>();

    values.sort_by_key(|r| OrderedFloat(r.length));

    for record in values {
        wtr.serialize(record)?;
    }

    wtr.flush()?;

    Ok(())
}