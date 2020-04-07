use std::fs;
use std::env;
use rand::prelude::*;
use ant_alg::ants_algs::{Config, get_possibly_shortest_way_sync, gen_graph};
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let file = fs::read_to_string(file_path).unwrap();
    let targets_str = &args[2..];
    let mut targets = Vec::new();
    for target in targets_str {
        let target: i32 = target.parse().unwrap();
        targets.push(target);
    }

    let config: &'static Config = &Config {
        alfa: 0.7,
        beta: 0.3,
        ant_capacity: 1000.0,
        ro: 0.3,
        ant_num: 50,
        iters: 10
    };

    let mut weight_mat: Vec<Vec<f64>> = Vec::new();

    for line in file.lines() {
        let mut temp_vec = vec![];
        for val in line.split(" ") {
            let b: f64 = val.parse().unwrap();
            temp_vec.push(b);
        }
        weight_mat.push(temp_vec);
    }
    
    if targets.len() < 3 {
        panic!("Not enough targets");
    }

    let (answer, min_way) = get_possibly_shortest_way_sync(weight_mat, config, targets[0], targets);
    println!("path: {:?}, total_way: {}", answer, min_way);
}