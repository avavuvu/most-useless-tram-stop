use std::{cmp::Reverse, collections::{BinaryHeap, HashMap}};

use crate::{Stop, haversine::haversine_distance};
use anyhow::{Result};
use ordered_float::OrderedFloat;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Distance {
    pub length: OrderedFloat<f64>,
    pub a: usize,
    pub b: usize,
    key: String
}

pub fn bruteforce_v2(stops: &Vec<Stop>) -> Result<Vec<(f64, Stop, Stop)>> {

    // first split each into its own route

    let mut routes_map = HashMap::<String, Vec<Stop>>::new(); 
    for stop in stops {
        routes_map
            .entry(match &stop.route_name {
                None => panic!("Should not pass government data here"),
                Some(s) => s,
            }.clone())
            .or_default()
            .push(stop.clone());
    }

    let mut all_stops = BinaryHeap::<Reverse<Distance>>::new();

    for (key, stops) in routes_map.iter() {
        for (i, i_stop) in stops.iter().enumerate() {
            for (j, j_stop) in stops.iter().enumerate() {
                if i == j { 
                    continue
                }

                let stop_difference = i_stop.stop_number - j_stop.stop_number;

                // this is just to check hawthorn road & dandenong road
                if i_stop.stop_modifier.eq(&j_stop.stop_modifier) && stop_difference == 0  {
                    continue;
                }
    
                let distance = haversine_distance(i_stop.lat, i_stop.long, j_stop.lat, j_stop.long);
    
                all_stops.push(Reverse((Distance { 
                    length: ordered_float::OrderedFloat(distance), a: i, b: j, key: key.clone()
                })));
            }
        }
    }

    let mut results: Vec<Distance>= all_stops
        .into_sorted_vec()
        .into_iter()
        .rev()
        .map(|Reverse(distance)|distance)
        .collect();
        

    results.dedup_by(|a, b| {
        (a.length - b.length).abs() < 0.001
    });

    Ok(results
        .into_iter()
        .take(200)
        .map(|distance| {
            (
                distance.length.into_inner(), 
                routes_map[&distance.key][distance.a].clone(),
                routes_map[&distance.key][distance.b].clone(),
            )
        })
        .collect()
    )
}