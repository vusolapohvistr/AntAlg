use std::fs;
use std::env;
use std::collections::HashMap;
use rand::Rng;
use std::f64::MAX;

struct Config {
    alfa: f64,
    beta: f64,
    ant_capacity: f64,
    ro: f64,
    ant_num: i64,
    iters: i64,
}

struct Ant<'a> {
    path: Vec<usize>,
    total_way: f64,
    cant_find_way: bool,
    config: &'a Config,
}

trait AntInterface {
    fn go(&mut self,
          start_point: i32,
          targets: &Vec<i32>,
          weight_mat: &Vec<Vec<f64>>,
          pheromones_mat: &Vec<Vec<f64>>);
    fn change_pheromones_mat(&mut self, pheromones_matrix: &mut Vec<Vec<f64>>);
    fn is_all_targets_visited(&self, visited_targets_counter: &HashMap<i32, i32>) -> bool;
}

impl Ant<'_> {
    fn new<'a>(config: &'a Config) -> Ant {
        Ant {
            config,
            path: vec![],
            total_way: 0.0,
            cant_find_way: false,
        }
    }
}

impl AntInterface for Ant<'_> {
    fn go(&mut self,
          start_point: i32,
          targets: &Vec<i32>,
          weight_mat: &Vec<Vec<f64>>,
          pheromones_mat: &Vec<Vec<f64>>) {
        let mut current_pos: usize = start_point as usize;
        self.path.push(current_pos);

        let mut visited_targets_counter = HashMap::new();
        for target in targets {
            visited_targets_counter.insert(*target, 0);
        }

        let mut rand = rand::thread_rng();

        while self.is_all_targets_visited(&visited_targets_counter) == false {
            let mut balance_sum = 0.0;
            for i in 0..weight_mat.len() {
                if weight_mat[current_pos][i].eq(&0.0) || self.path.contains(&i) {
                    continue;
                }
                balance_sum += pheromones_mat[current_pos][i].powf(self.config.alfa)
                    * (1.0 / weight_mat[current_pos][i]).powf(self.config.beta);
            }

            if balance_sum == 0.0 {
                self.cant_find_way = true;
                break;
            }

            for i in 0..weight_mat.len() {
                if weight_mat[current_pos][i] == 0.0 || self.path.contains(&i) {
                    continue;
                }
                let prob_to_move: f64 = pheromones_mat[current_pos][i].powf(self.config.alfa) *
                    (1.0 / weight_mat[current_pos][i]).powf(self.config.beta) / balance_sum;

                let random_number: f64 = rand.gen();
                if random_number < prob_to_move {
                    self.path.push(i);
                    match visited_targets_counter.get(&(i as i32)) {
                        Some(mut v) => v = &(v + 1),
                        None => {}
                    }
                    self.total_way += weight_mat[current_pos][i];
                    current_pos = i;
                    break;
                }
            }
        }
    }


    fn change_pheromones_mat(&mut self, pheromones_matrix: &mut Vec<Vec<f64>>) {
        if self.cant_find_way == false {
            for i in 0..self.path.len() - 1 {
                pheromones_matrix[self.path[i]][self.path[i + 1]] +=
                    self.config.ant_capacity / self.total_way;
            }
        }
        self.total_way = 0.0;
        self.path.clear();
    }

    fn is_all_targets_visited(&self, visited_targets_counter: &HashMap<i32, i32>) -> bool {
        for v in visited_targets_counter.values() {
            if *v == 0 {
                return false
            }
        }
        true
    }
}

fn vaporize_pheromones(pheromones_mat: &mut Vec<Vec<f64>>, config: &Config) {
    for mut i in pheromones_mat.iter() {
        for mut g in i.iter() {
            g = &(g * (1.0 - config.ro));
        }
    }
}

fn get_possibly_shortest_way_sync(weight_mat: &mut Vec<Vec<f64>>, config: &Config, start_point: i32, targets: &Vec<i32>) -> Vec<usize> {
    let mut answer: Vec<usize> = Vec::new();
    let mut pheromones_mat = vec![vec![1.0; weight_mat.len()]; weight_mat.len()];
    let mut ants: Vec<Ant> = Vec::new();
    for i in 0..config.ant_num {
        ants.push(Ant::new(&config));
    }

    for i in 0..config.iters {
        let mut min_total_way = MAX;
        vaporize_pheromones(&mut pheromones_mat, &config);
        for ant in &mut ants {
            ant.go(start_point, &targets, &weight_mat, &pheromones_mat);
            if ant.total_way < min_total_way {
                min_total_way = ant.total_way;
                answer = ant.path.clone();
            }
        }
        for ant in &mut ants {
            ant.change_pheromones_mat(&mut pheromones_mat);
        }
    }
    answer
}

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

    let config = Config {
        alfa: 0.4,
        beta: 0.6,
        ant_capacity: 100.0,
        ro: 0.05,
        ant_num: 5,
        iters: 5
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

    let answer = get_possibly_shortest_way_sync(&mut weight_mat, &config, targets[0], &targets);
    println!("{:?}", answer);
}
