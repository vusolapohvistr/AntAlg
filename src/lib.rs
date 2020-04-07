pub mod ants_algs {
    use std::collections::HashMap;
    use rand::Rng;
    use std::f64::MAX;
    use std::thread;
    use std::sync::{Mutex, Arc, mpsc};
    use std::thread::JoinHandle;
    use std::cmp::min;
    use std::sync::mpsc::{Sender, Receiver};
    use std::time::{Duration};


    pub struct Config {
        pub alfa: f64,
        pub beta: f64,
        pub ant_capacity: f64,
        pub ro: f64,
        pub ant_num: i64,
        pub iters: i64,
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
        fn is_all_targets_visited(&self, visited_targets_counter: &HashMap<usize, i32>) -> bool;
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
                visited_targets_counter.insert(*target as usize, 0);
            }

            let mut rand = rand::thread_rng();

            while !self.is_all_targets_visited(&visited_targets_counter)
                || (self.is_all_targets_visited(&visited_targets_counter) && *self.path.last().unwrap() != start_point as usize) {
                let mut balance_sum = 0.0;
                for i in 0..weight_mat.len() {
                    if weight_mat[current_pos][i].eq(&0.0) {
                        continue;
                    }
                    balance_sum += pheromones_mat[current_pos][i].powf(self.config.alfa)
                        * (1.0 / weight_mat[current_pos][i]).powf(self.config.beta);
                }

                if balance_sum == 0.0 {
                    self.cant_find_way = true;
                    break;
                }

                for mut i in 0..weight_mat.len() {
                    if weight_mat[current_pos][i].eq(&0.0)
                        || (self.path.len() > 1 && i == self.path[self.path.len() - 2]) {
                        continue;
                    }
                    let prob_to_move: f64 = pheromones_mat[current_pos][i].powf(self.config.alfa) *
                        (1.0 / weight_mat[current_pos][i]).powf(self.config.beta) / balance_sum;

                    let random_number: f64 = rand.gen();
                    if random_number < prob_to_move {
                        self.path.push(i);
                        if let Some(v) = visited_targets_counter.get(&mut i) {
                            visited_targets_counter.insert(i, *v + 1);
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

        fn is_all_targets_visited(&self, visited_targets_counter: &HashMap<usize, i32>) -> bool {
            for v in visited_targets_counter.values() {
                if *v == 0 {
                    return false
                }
            }
            true
        }
    }

    fn vaporize_pheromones(pheromones_mat: &mut Vec<Vec<f64>>, config: &Config) {
        for i in pheromones_mat.iter() {
            for mut g in i.iter() {
                g = &(g * (1.0 - config.ro));
            }
        }
    }

    pub fn get_possibly_shortest_way_sync(weight_mat: Vec<Vec<f64>>, config: &Config, start_point: i32, targets: Vec<i32>) -> (Vec<usize>, f64) {
        let mut answer: Vec<usize> = Vec::new();
        let mut min_way = MAX;
        let mut pheromones_mat = vec![vec![1.0; weight_mat.len()]; weight_mat.len()];
        let mut ants: Vec<Ant> = Vec::new();
        for _ in 0..config.ant_num {
            ants.push(Ant::new(&config));
        }

        for _ in 0..config.iters {
            vaporize_pheromones(&mut pheromones_mat, &config);
            for ant in &mut ants {
                ant.go(start_point, &targets, &weight_mat, &pheromones_mat);
                if ant.total_way < min_way {
                    min_way = ant.total_way;
                    answer = ant.path.clone();
                }
            }
            for ant in &mut ants {
                ant.change_pheromones_mat(&mut pheromones_mat);
            }
        }
        (answer, min_way)
    }

    pub fn get_possibly_shortest_way_threads(weight_mat: Vec<Vec<f64>>, config: &'static Config, start_point: i32, targets: Vec<i32>) -> (Vec<usize>, f64) {
        let weight_mat = Arc::new(weight_mat);
        let targets = Arc::new(targets);
        let answer = Arc::new(Mutex::new(Vec::new()));
        let min_way = Arc::new(Mutex::new(MAX));
        let pheromones_mat = Arc::new(Mutex::new(vec![vec![1.0; weight_mat.len()]; weight_mat.len()]));
        let mut ants: Vec<Mutex<Ant>> =  Vec::new();
        for _ in 0..config.ant_num {
            ants.push(Mutex::new(Ant::new(&config)));
        }

        let ants = Arc::new(ants);

        for _ in 0..config.iters {
            let mut tds: Vec<JoinHandle<()>> = Vec::new();
            let mut pheromones_mat = &mut *pheromones_mat.lock().unwrap();
            vaporize_pheromones(&mut pheromones_mat, &config);
            for i in 0..ants.len() {
                let ants_ref_copy = ants.clone();
                let pheromones_mat = pheromones_mat.clone();
                let weight_mat = weight_mat.clone();
                let targets = targets.clone();
                let min_way = min_way.clone();
                let answer = answer.clone();
                let td = thread::spawn(move || {
                    let ant = &ants_ref_copy[i];
                    let ant = &mut *ant.lock().unwrap();
                    ant.go(start_point, &targets, &weight_mat, &pheromones_mat);
                    if ant.total_way < *min_way.lock().unwrap() {
                        *min_way.lock().unwrap() = ant.total_way;
                        *answer.lock().unwrap() = ant.path.clone();
                    }
                });
                tds.push(td);
            }
            for td in tds {
                td.join().unwrap_or_default();
            }
            for i in 0..ants.len() {
                let ant = &ants[i];
                (*ant.lock().unwrap()).change_pheromones_mat(&mut pheromones_mat);
            }
        }

        let answer= (*answer.lock().unwrap()).clone();
        let min_way = *min_way.lock().unwrap();
        (answer, min_way)
    }

    pub fn get_possibly_shortest_way_channels(weight_mat: Vec<Vec<f64>>, config: &'static Config, start_point: i32, targets: Vec<i32>) -> (Vec<usize>, f64) {
        let weight_mat = Arc::new(weight_mat);
        let targets = Arc::new(targets);
        let answer = Arc::new(Mutex::new(Vec::new()));
        let min_way = Arc::new(Mutex::new(MAX));
        let pheromones_mat =  Arc::new(Mutex::new(vec![vec![1.0; weight_mat.len()]; weight_mat.len()]));
        let mut ants =  Vec::new();
        for _ in 0..config.ant_num {
            ants.push(Ant::new(&config));
        }

        let (tx, rx): (Sender<Ant>, Receiver<Ant>) = mpsc::channel();

        for _ in 0..config.iters {
            let mut pheromones_mat = &mut *pheromones_mat.lock().unwrap();
            vaporize_pheromones(&mut pheromones_mat, &config);
            let mut received_ants = Vec::new();
            let mut tds: Vec<JoinHandle<()>> = Vec::new();
            for _ in 0..config.ant_num {
                let pheromones_mat = pheromones_mat.clone();
                let weight_mat = weight_mat.clone();
                let targets = targets.clone();
                let min_way = min_way.clone();
                let answer = answer.clone();
                let mut ant = ants.pop().unwrap();
                let thread_tx = tx.clone();
                let td = thread::spawn(move || {
                    ant.go(start_point, &targets, &weight_mat, &pheromones_mat);
                    if ant.total_way < *min_way.lock().unwrap() {
                        *min_way.lock().unwrap() = ant.total_way;
                        *answer.lock().unwrap() = ant.path.clone();
                    }
                    thread_tx.send(ant).unwrap();
                });
                tds.push(td);
            }

            for _ in 0..config.ant_num {
                received_ants.push(rx.recv().unwrap());
            }

            for td in tds {
                td.join().unwrap_or_default();
            }

            for ant in &mut received_ants {
                ant.change_pheromones_mat(&mut pheromones_mat);
            }

            ants = received_ants;
        }

        let answer= (*answer.lock().unwrap()).clone();
        let min_way = *min_way.lock().unwrap();

        (answer, min_way)
    }

    pub fn gen_graph(n: usize, additional_edges: usize, max_weight: f64) -> Vec<Vec<f64>> {
        let mut result = vec![vec![0.0; n]; n];
        
        let mut used_nodes: HashMap<usize, bool> = HashMap::new();
        let mut rng = rand::thread_rng();
        let mut current_node: usize = 0;

        while used_nodes.len() != n {
            let mut next_node: usize = current_node;

            while next_node == current_node {
                next_node = rng.gen_range(0, n);
            }
            let weight = (rng.gen_range(1.0, max_weight)).round();
            result[current_node][next_node] = weight;
            result[next_node][current_node] = weight;
            used_nodes.entry(current_node).or_insert(true);
    
            current_node = next_node;
        }

        if current_node != 0 {
            result[current_node][0] = 10.0;
            result[0][current_node] = 10.0;
        }
    
        for _ in 0..additional_edges {
            let mut next_node: usize = current_node;
            while next_node == current_node {
                next_node = rng.gen_range(0, n);
            }
            let weight = (rng.gen_range(1.0, max_weight)).round();
            result[current_node][next_node] = weight;
            result[next_node][current_node] = weight;
            current_node = next_node;
        }

        if current_node != 0 {
            result[current_node][0] = 10.0;
            result[0][current_node] = 10.0;
        }
    
        result
    }
}