use std::{fs::File, io};
use anyhow::{Context, Ok, Result};
use ordered_float::OrderedFloat;
use serde::Serialize;

use crate::{bruteforce::bruteforce, bruteforce_v2::bruteforce_v2};
pub mod bruteforce;
pub mod haversine;
pub mod app;
pub mod visualize;
pub mod write_csv;
pub mod bruteforce_v2;


#[derive(Debug)]
#[derive(Clone)]
pub struct Stop {
    long: f64,
    lat: f64,
    name: String,
    stop_number: i32,
    route_name: Option<String>
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Distance {
    pub length: OrderedFloat<f64>,
    pub a: usize,
    pub b: usize,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct DistanceRecord {
    pub length: f64,
    pub stop_a_name: String,
    pub stop_a_lat: f64,
    pub stop_a_long: f64,

    pub stop_b_name: String,
    pub stop_b_lat: f64,
    pub stop_b_long: f64,
}

type RecordGov = (String, String, i32);
type RecordOSM = (String, String, String, i32); //geometry.coordinates	route_name	properties.name	stop_number

fn split_coords_string(coords: &str) -> Result<(f64, f64)> {
    let split = coords
        .split_once(',')
        .context("Unable to unwrap")?;

    let long: f64 = split.0.parse()?;
    let lat: f64 = split.1.parse()?;

    Ok((long, lat))
}

fn load_osm_data() -> Result<Vec<Stop>> {
    let file = File::open("../data/osm_tram_stops.tsv")?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(file);

    let coords_list = reader.deserialize()
        .map(|result| -> Result<Stop> {
            let record: RecordOSM = result?;

            let (long, lat) = split_coords_string(&record.0)?;
            
            Ok(Stop {
                long,
                lat,
                route_name: Some(record.1),
                stop_number: record.3,
                name: record.2,
                
            })
        }).collect::<Result<Vec<Stop>>>()?;

    Ok(coords_list)
}


fn load_gov_data() -> Result<Vec<Stop>> {
    let file = File::open("../data/tram_stops.tsv")?;

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(file);

    let coords_list = reader.deserialize()
        .map(|result| -> Result<Stop> {
            let record: RecordGov = result?;

            let (long, lat) = split_coords_string(&record.0)?;
            
            Ok(Stop {
                long,
                lat,
                name: record.1,
                stop_number: record.2,
                route_name: None
            })
        }).collect::<Result<Vec<Stop>>>()?;

    Ok(coords_list)
}

fn main() -> io::Result<()> {
    let stops = load_osm_data().unwrap();
    let closest = bruteforce_v2(&stops);
    
    let mut terminal = ratatui::init();
    let mut app = app::App::default();

    // let stops = load_gov_data().unwrap();
    // let closest = bruteforce(&stops, |progress| app.progress = progress);

    app.result = Some(closest.map_err(|e|"".to_owned()));

    let app_result = app.run(&mut terminal);
    ratatui::restore();

    app_result
}