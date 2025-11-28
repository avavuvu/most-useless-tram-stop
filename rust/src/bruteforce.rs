use std::{collections::{BinaryHeap}};
use anyhow::Result;
use std::cmp::Reverse;


use crate::{Distance, Stop, haversine::haversine_distance};

pub fn bruteforce<F>(stops: &Vec<Stop>, mut progress_callback: F) -> Result<Vec<(f64, Stop, Stop)>> 
where 
    F: FnMut(u16) 
{
    let mut heap: BinaryHeap<Reverse<Distance>> = BinaryHeap::new();
    let mut count: u64 = 0;

    for (i, i_stop) in stops.iter().enumerate() {
        let i_stop_streets = i_stop.name.split_once('/');

        for (j, j_stop) in stops.iter().enumerate() {
            count += 1;
            let progress = (i + 1) as u16 / stops.len() as u16 * 100;
            progress_callback(progress);

            if i == j { 
                continue
            }

            let stop_difference = i_stop.stop_number - j_stop.stop_number;

            if stop_difference == 0 || stop_difference.abs() > 1 {
                continue;
            }

            let j_stop_streets = j_stop.name.split_once('/');

            if let (Some(i_split), Some(j_split)) = (i_stop_streets, j_stop_streets) {
                if (j_split.0 == i_split.0 && j_split.1 == i_split.1)
                || (j_split.0 == i_split.1 && j_split.1 == i_split.0) {
    
                    // println!("{} and {} are ultimately the same", j_split.0, i_split.0);
                    continue;
                }
            }

            let distance = haversine_distance(i_stop.lat, i_stop.long, j_stop.lat, j_stop.long);

            heap.push(Reverse(Distance { 
                length: ordered_float::OrderedFloat(distance), a: i, b: j 
            }));
        }
    }

    Ok(heap
        .into_sorted_vec()
        .into_iter()
        .rev()
        .take(200)
        .map(|Reverse(distance)| {
            (
                distance.length.into_inner(), 
                stops[distance.a].clone(),
                stops[distance.b].clone()
            )
        })
        .collect()
    )
}