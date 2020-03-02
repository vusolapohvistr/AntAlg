use std::fs;
use std::env;

struct Config {
    alfa: f32,
    beta: f32,
    ant_capacity: i32,
    ro: f32,
    ant_num: i32,
    iters: i32,
}

struct Ant {
    path: Vec<i32>,
    total_way: i32,
    cant_find_way: bool,
}

trait AntInterface {
    fn go(&self,
          start_point: i32,
          targets: Vec<i32>,
          weight_mat: Vec<Vec<i32>>,
          pheromones_mat: Vec<Vec<i32>>);
    fn change_pheromones_mat(&self, &mut pheromones_matrix: Vec<Vec<i32>>);
}

impl Ant {
    fn new() -> Ant {
        Ant {
            path: vec![],
            total_way: 0,
            cant_find_way: false,
        }
    }
}

impl AntInterface for Ant {
    fn go(&mut self,
          start_point: i32,
          targets: Vec<i32>,
          weight_mat: Vec<Vec<i32>>,
          pheromones_mat: Vec<Vec<i32>>) {
        let current_pos = start_point;
        self.path.push(current_pos);
    }


    fn change_pheromones_mat(&self, &mut pheromones_matrix: Vec<Vec<i32>>) {
        unimplemented!()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let file = fs::read_to_string(file_path).unwrap();
    let targets = &args[2..];

    let config = Config {
        alfa: 0.4,
        beta: 0.6,
        ant_capacity: 100,
        ro: 0.05,
        ant_num: 5,
        iters: 5
    };

    println!("{:?}", targets);
}
