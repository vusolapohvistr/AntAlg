#![feature(array_windows)]

pub mod ants_algs {
    use rand::Rng;
    use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::mpsc;
    use std::thread;

    pub struct Config {
        pub alfa: f64,
        pub beta: f64,
        pub ant_capacity: f64,
        pub ro: f64,
        pub ant_num: i64,
        pub iters: i64,
    }

    #[derive(Clone)]
    struct Ant<'a> {
        path: Vec<usize>,
        total_way: f64,
        cant_find_way: bool,
        config: &'a Config,
    }

    impl Ant<'_> {
        fn new(config: &Config) -> Ant {
            Ant {
                config,
                path: vec![],
                total_way: 0.0,
                cant_find_way: false,
            }
        }

        fn go(
            &mut self,
            start_point: i32,
            targets: &[i32],
            weight_mat: &[Vec<f64>],
            pheromones_mat: &[Vec<f64>],
        ) {
            let mut current_pos: usize = start_point as usize;
            self.path.push(current_pos);

            let mut left_to_visit: HashSet<i32> = HashSet::from_iter(targets.iter().copied());
            let mut rand = rand::thread_rng();
            while !left_to_visit.is_empty()
                || (left_to_visit.is_empty() && self.path.last().copied() != Some(start_point as usize))
            {
                let balance_sum: f64 = weight_mat[current_pos]
                    .iter()
                    .zip(pheromones_mat[current_pos].iter())
                    .filter(|(w, _p)| **w != 0.0)
                    .map(|(w, p)| p.powf(self.config.alfa) * (1.0 / w).powf(self.config.beta))
                    .sum();

                if balance_sum == 0.0 {
                    self.cant_find_way = true;
                    break;
                }

                let prev = self.path.get(self.path.len() - 2).copied();
                let mut next = None;
                while next.is_none() {
                    next = weight_mat[current_pos]
                        .iter()
                        .zip(pheromones_mat[current_pos].iter().enumerate())
                        .filter(|(w, (i, _p))| **w != 0.0 && Some(*i) != prev)
                        .map(|(w, (i, p))| {
                            let prob_to_move: f64 = p.powf(self.config.alfa)
                                * (1.0 / w).powf(self.config.beta)
                                / balance_sum;

                            (i, prob_to_move)
                        })
                        .find(|(_i, prob_to_move)| *prob_to_move < rand.gen())
                        .map(|(i, _)| i);

                    if let Some(i) = next {
                        self.path.push(i);
                        left_to_visit.remove(&(i as i32));
                        self.total_way += weight_mat[current_pos][i];
                        current_pos = i;
                    }
                }
            }
        }

        fn change_pheromones_mat(&mut self, pheromones_matrix: &mut [Vec<f64>]) {
            if !self.cant_find_way {
                for [start, end] in self.path.array_windows::<2>() {
                    pheromones_matrix[*start][*end] += self.config.ant_capacity / self.total_way;
                }
            }

            self.total_way = 0.0;
            self.path.clear();
        }
    }

    fn vaporize_pheromones(pheromones_mat: &mut [Vec<f64>], config: &Config) {
        pheromones_mat
            .iter_mut()
            .for_each(|v| v.iter_mut().for_each(|g| *g *= 1.0 - config.ro));
    }

    pub fn get_possibly_shortest_way_sync(
        weight_mat: Vec<Vec<f64>>,
        config: &Config,
        start_point: i32,
        targets: Vec<i32>,
    ) -> (Vec<usize>, f64) {
        let mut answer: Vec<usize> = Vec::new();
        let mut min_way = f64::MAX;
        let mut pheromones_mat = vec![vec![1.0; weight_mat.len()]; weight_mat.len()];
        let mut ants: Vec<Ant> = vec![Ant::new(config); config.ant_num as usize];

        for _ in 0..config.iters {
            vaporize_pheromones(&mut pheromones_mat, config);
            ants.iter_mut()
                .for_each(|ant| ant.go(start_point, &targets, &weight_mat, &pheromones_mat));
            for ant in ants.iter() {
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

    pub fn get_possibly_shortest_way_threads(
        weight_mat: Vec<Vec<f64>>,
        config: &'static Config,
        start_point: i32,
        targets: Vec<i32>,
    ) -> (Vec<usize>, f64) {
        let mut answer: Vec<usize> = Vec::new();
        let mut min_way = f64::MAX;
        let mut pheromones_mat = vec![vec![1.0; weight_mat.len()]; weight_mat.len()];
        let mut ants: Vec<Ant> = vec![Ant::new(config); config.ant_num as usize];

        for _ in 0..config.iters {
            vaporize_pheromones(&mut pheromones_mat, config);
            ants.par_iter_mut()
                .for_each(|ant| ant.go(start_point, &targets, &weight_mat, &pheromones_mat));

            let min_ant = ants.par_iter().min_by(|a, b| a.total_way.total_cmp(&b.total_way));
            if let Some(min_ant) = min_ant {
                min_way = min_ant.total_way;
                answer = min_ant.path.clone();
            }

            for ant in &mut ants {
                ant.change_pheromones_mat(&mut pheromones_mat);
            }
        }

        (answer, min_way)
    }

    pub fn get_possibly_shortest_way_channels(
        weight_mat: Vec<Vec<f64>>,
        config: &'static Config,
        start_point: i32,
        targets: Vec<i32>,
    ) -> (Vec<usize>, f64) {
        let mut answer: Vec<usize> = Vec::new();
        let mut min_way = f64::MAX;
        let mut pheromones_mat = vec![vec![1.0; weight_mat.len()]; weight_mat.len()];
        let mut ants: Vec<Ant> = vec![Ant::new(config); config.ant_num as usize];

        let (tx, rx): (Sender<Ant>, Receiver<Ant>) = mpsc::channel();

        for _ in 0..config.iters {
            vaporize_pheromones(&mut pheromones_mat, config);
            let mut received_ants = Vec::new();

            thread::scope(|scope| {
                let mut tds = Vec::with_capacity(ants.len());

                for mut ant in ants.into_iter() {
                    let td = scope.spawn(|| {
                        ant.go(start_point, &targets, &weight_mat, &pheromones_mat);
                        tx.send(ant).unwrap();
                    });
                    tds.push(td);
                }

                for _ in 0..config.ant_num {
                    let ant = rx.recv().unwrap();
                    if ant.total_way < min_way {
                        min_way = ant.total_way;
                        answer = ant.path.clone();
                    }
                    received_ants.push(ant);
                }

                for td in tds {
                    td.join().unwrap_or_default();
                }
            });

            for ant in &mut received_ants {
                ant.change_pheromones_mat(&mut pheromones_mat);
            }

            ants = received_ants;
        }

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
