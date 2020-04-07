#[cfg(test)]
mod tests {
    use ant_alg::ants_algs::*;
    use std::time::Instant;

    #[test]
    fn ants_alg_sync_bench() {
        let graph_nodes_len = 100;
        let config: &'static Config = &Config {
            alfa: 0.7,
            beta: 0.3,
            ant_capacity: 1000.0,
            ro: 0.3,
            ant_num: 50,
            iters: 100
        };
        
        let graph = ant_alg::ants_algs::gen_graph(graph_nodes_len, graph_nodes_len * 3, 10.0);
        let graph_clone_1 = graph.clone();
        let graph_clone_2 = graph.clone();

        let targets = vec![0, graph_nodes_len as i32 / 4, graph_nodes_len as i32 / 3, graph_nodes_len as i32 / 2];
        let now = Instant::now(); 
        let (answer, min_way) = get_possibly_shortest_way_sync(graph, config, targets[0], targets);
        println!("sync: answer {:?} \n min_way {}, took {} ms", answer, min_way, now.elapsed().as_millis());

        let targets = vec![0, graph_nodes_len as i32 / 4, graph_nodes_len as i32 / 3, graph_nodes_len as i32 / 2];
        let now = Instant::now(); 
        let (answer, min_way) = get_possibly_shortest_way_channels(graph_clone_1, config, targets[0], targets);
        println!("chanells: answer {:?} \n min_way {}, took {} ms", answer, min_way, now.elapsed().as_millis());

        let targets = vec![0, graph_nodes_len as i32 / 4, graph_nodes_len as i32 / 3, graph_nodes_len as i32 / 2];
        let now = Instant::now(); 
        let (answer, min_way) = get_possibly_shortest_way_threads(graph_clone_2, config, targets[0], targets);
        println!("threads: answer {:?} \n min_way {}, took {} ms", answer, min_way, now.elapsed().as_millis());
    }
}