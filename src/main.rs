use ant_alg::ants_algs::{get_possibly_shortest_way_sync, Config};
use ant_alg::ants_algs::{get_possibly_shortest_way_channels, get_possibly_shortest_way_threads};




use std::time::Instant;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let file_path = &args[1];
    // let file = fs::read_to_string(file_path).unwrap();
    // let targets_str = &args[2..];
    // let mut targets = Vec::new();
    // for target in targets_str {
    //     let target: i32 = target.parse().unwrap();
    //     targets.push(target);
    // }

    // let config: &'static Config = &Config {
    //     alfa: 0.7,
    //     beta: 0.3,
    //     ant_capacity: 1000.0,
    //     ro: 0.3,
    //     ant_num: 50,
    //     iters: 10,
    // };

    // let mut weight_mat: Vec<Vec<f64>> = Vec::new();

    // for line in file.lines() {
    //     let mut temp_vec = vec![];
    //     for val in line.split(" ") {
    //         let b: f64 = val.parse().unwrap();
    //         temp_vec.push(b);
    //     }
    //     weight_mat.push(temp_vec);
    // }

    // if targets.len() < 3 {
    //     panic!("Not enough targets");
    // }

    // let (answer, min_way) = get_possibly_shortest_way_sync(weight_mat, config, targets[0], targets);
    // println!("path: {:?}, total_way: {}", answer, min_way);

    let graph_nodes_len = 100;
    let config: &'static Config = &Config {
        alfa: 0.7,
        beta: 0.3,
        ant_capacity: 1000.0,
        ro: 0.3,
        ant_num: 50,
        iters: 100,
    };

    let graph = ant_alg::ants_algs::gen_graph(graph_nodes_len, graph_nodes_len * 3, 10.0);
    let graph_clone_1 = graph.clone();
    let graph_clone_2 = graph.clone();

    let targets = vec![
        0,
        graph_nodes_len as i32 / 4,
        graph_nodes_len as i32 / 3,
        graph_nodes_len as i32 / 2,
    ];
    let now = Instant::now();
    let (answer, min_way) = get_possibly_shortest_way_sync(graph, config, targets[0], targets);
    println!(
        "sync: answer {:?} \n min_way {}, took {} ms",
        answer,
        min_way,
        now.elapsed().as_millis()
    );

    let targets = vec![
        0,
        graph_nodes_len as i32 / 4,
        graph_nodes_len as i32 / 3,
        graph_nodes_len as i32 / 2,
    ];
    let now = Instant::now();
    let (answer, min_way) =
        get_possibly_shortest_way_channels(graph_clone_1, config, targets[0], targets);
    println!(
        "chanells: answer {:?} \n min_way {}, took {} ms",
        answer,
        min_way,
        now.elapsed().as_millis()
    );

    let targets = vec![
        0,
        graph_nodes_len as i32 / 4,
        graph_nodes_len as i32 / 3,
        graph_nodes_len as i32 / 2,
    ];
    let now = Instant::now();
    let (answer, min_way) =
        get_possibly_shortest_way_threads(graph_clone_2, config, targets[0], targets);
    println!(
        "threads: answer {:?} \n min_way {}, took {} ms",
        answer,
        min_way,
        now.elapsed().as_millis()
    );
}
