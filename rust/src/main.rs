use std::{fs::File, io};
use anyhow::{Result, Context};
use serde::Serialize;

use crate::bruteforce::bruteforce;
pub mod bruteforce;
pub mod haversine;
pub mod app;
pub mod visualize;
pub mod write_csv;

#[derive(Debug, Default)]
#[derive(Clone)]
pub struct Stop {
    long: f64,
    lat: f64,
    name: String,
    stop_number: i32
}

#[derive(Debug, Default, Serialize)]
pub struct DistanceRecord {
    pub length: f64,
    pub stop_a_name: String,
    pub stop_a_lat: f64,
    pub stop_a_long: f64,

    pub stop_b_name: String,
    pub stop_b_lat: f64,
    pub stop_b_long: f64,
}

type Record = (String, String, i32);

fn load_data() -> Result<Vec<Stop>> {
    let file = File::open("../data/tram_stops.tsv")?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(file);

    let coords_list = reader.deserialize()
        .map(|result| -> Result<Stop> {
            let record: Record = result?;
            
            let split = record.0
                .split_once(',')
                .context("Unable to unwrap")?;

            let long: f64 = split.0.parse()?;
            let lat: f64 = split.1.parse()?;

            Ok(Stop {
                long,
                lat,
                name: record.1,
                stop_number: record.2
            })
        }).collect::<Result<Vec<Stop>>>()?;

    Ok(coords_list)
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = app::App::default();

    let stops = load_data().unwrap();
    let closest = bruteforce(&stops, |progress| app.progress = progress);

    app.result = Some(closest.map_err(|e|"".to_owned()));

    let app_result = app.run(&mut terminal);
    ratatui::restore();

    app_result
}