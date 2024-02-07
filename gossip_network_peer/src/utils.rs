/// Contains the utils

// importing necessary packages
use std::{collections::HashSet, error::Error};
use rand::Rng;

// importing necessary files
use crate::file_reader;

// Reads the Ips and ports from a file and parses it.
pub fn get_ips(file_path: &str) ->  Result<Vec<String>, Box<dyn Error>> {
    let (ips, ports) = file_reader::read_file(file_path)?;
    let mut ip: Vec<String> = Vec::new();
    for i in 0..ips.len() {
        ip.push(format!("{}:{}", ips[i], ports[i]));
    }
    Ok(ip)
}

// returns the time in miliseconds from a string
pub fn parse_and_convert_to_ms(time_string: &str) -> i32 {
    // Split the string and collect into a Vec
    let parts: Vec<&str> = time_string.split(":").collect();
    // Ensure correct format
    if parts.len() != 3 {
        panic!("Invalid time format: expected MM::SS::MS");
    }

    // Parse each part and handle potential parsing errors
    let minutes = parts[0].parse::<i32>().unwrap();
    let seconds = parts[1].parse::<i32>().unwrap();
    let milliseconds = parts[2].parse::<i32>().unwrap();

    // Calculate total milliseconds
    let total_milliseconds = minutes * 60 * 1000 + seconds * 1000 + milliseconds;

    // Handle overflow
    if total_milliseconds > i32::MAX {
        panic!("Overflow: total milliseconds exceed u64 limit");
    }
    total_milliseconds
}

// logic for selecting k distinct nodes at random from a Vector.
pub fn select_k_nodes(seeds: Vec<String> , k: usize) -> Vec<String> {
    let n = seeds.len();
    // let k = n/2+1;
    let mut rng = rand::thread_rng();
    let mut distinct_indexes = HashSet::with_capacity(k);

    while distinct_indexes.len() < k {
        let random_number = rng.gen_range(0..n);
        distinct_indexes.insert(random_number);
    }
    let distinct_indexes: Vec<usize> = distinct_indexes.into_iter().collect();
    let result_vector: Vec<_> = distinct_indexes.clone()
        .iter()
        .map(|&index| seeds[index].clone())
        .collect();
    return result_vector
}
