use crate::Maze;
use petgraph::{
    algo::astar,
    graph::{NodeIndex, UnGraph},
};
use std::time::Instant;

#[cfg(feature = "raw_result")]
macro_rules! astar {
    ($from:expr, $to:expr, $maze:expr, $graph:expr) => {
        find_path_astar_inner($from, $to, $maze, $graph)
    };
}
#[cfg(not(feature = "raw_result"))]
macro_rules! astar {
    ($from:expr, $to:expr, $maze:expr, $graph:expr) => {
        find_path_astar($from, $to, $maze, $graph)
    };
}

pub fn benchmark_petgraph(
    maze: &Maze,
    points: Option<Vec<((usize, usize), (usize, usize))>>,
    attempts: usize,
    tries: usize,
    searches: usize,
) {
    let graph = build_petgraph(maze);
    if let Some(points) = points {
        for point in &points {
            let from = point.0;
            let to = point.1;
            let mut durations = Vec::with_capacity(tries);
            for i in 0..tries {
                let mut path = None;
                let timer = Instant::now();
                for _ in 0..attempts {
                    path = astar!(from, to, maze, &graph);
                }
                let elapsed = timer.elapsed();
                if let Some(path) = path {
                    println!(
                        "#{} | Duration: {:?} | Points: {}:{},{}:{} | Path: {} | Attempts: {}",
                        i,
                        elapsed,
                        from.0,
                        from.1,
                        to.0,
                        to.1,
                        path.len(),
                        attempts,
                    );
                    durations.push(elapsed);
                }
            }
            if !durations.is_empty() {
                let min = durations.iter().min().unwrap();
                let max = durations.iter().max().unwrap();
                let diff = *max - *min;
                let avg = (*min + *max) / 2;
                println!(
                    "Avg: {:?} | Min: {:?} | Max: {:?} | Diff: {:?}",
                    avg, min, max, diff
                );
            }
        }
    } else {
        let mut found = 0;
        for i in 0..(searches * 10) {
            let from = (
                rand::random::<usize>() % maze.cols,
                rand::random::<usize>() % maze.rows,
            );
            let to = (
                rand::random::<usize>() % maze.cols,
                rand::random::<usize>() % maze.rows,
            );
            let timer = Instant::now();
            if let Some(mut path) = astar!(from, to, maze, &graph) {
                for _ in 1..attempts {
                    path = astar!(from, to, maze, &graph).unwrap();
                }
                let elapsed = timer.elapsed();
                println!(
                    "#{} ({}) | Duration: {:?} | Points: {}:{},{}:{} | Path: {} | Attempts: {}",
                    found,
                    i,
                    elapsed,
                    from.0,
                    from.1,
                    to.0,
                    to.1,
                    path.len(),
                    attempts,
                );
                found += 1;
                if found >= searches {
                    break;
                }
            }
        }
    }
}

fn build_petgraph(maze: &Maze) -> UnGraph<usize, (usize, usize), usize> {
    let mut graph =
        UnGraph::with_capacity(maze.cols * maze.rows, (maze.cols - 1) * (maze.rows - 1) * 4);
    for i in 0..maze.tiles.len() {
        let node = graph.add_node(i);
        if node != i.into() {
            panic!(
                "Expects node to be equal of its index: {} -> {}",
                node.index(),
                i
            );
        }
    }
    for row in 1..maze.rows {
        for col in 1..maze.cols {
            if let Some((ci, cv)) = maze.get_index_with_tile(col, row) {
                if cv {
                    continue;
                }
                if let Some((i, v)) = maze.get_index_with_tile(col, row - 1) {
                    if !v {
                        graph.add_edge(ci.into(), i.into(), (ci, i));
                    }
                }
                if let Some((i, v)) = maze.get_index_with_tile(col, row + 1) {
                    if !v {
                        graph.add_edge(ci.into(), i.into(), (ci, i));
                    }
                }
                if let Some((i, v)) = maze.get_index_with_tile(col - 1, row) {
                    if !v {
                        graph.add_edge(ci.into(), i.into(), (ci, i));
                    }
                }
                if let Some((i, v)) = maze.get_index_with_tile(col + 1, row) {
                    if !v {
                        graph.add_edge(ci.into(), i.into(), (ci, i));
                    }
                }
            }
        }
    }
    graph
}

fn find_path_astar_inner(
    from: (usize, usize),
    to: (usize, usize),
    maze: &Maze,
    graph: &UnGraph<usize, (usize, usize), usize>,
) -> Option<Vec<NodeIndex<usize>>> {
    if from.0 >= maze.cols || from.1 >= maze.rows || to.0 >= maze.cols || to.1 >= maze.rows {
        return None;
    }
    let from = (from.1 * maze.cols + from.0).into();
    let to = (to.1 * maze.cols + to.0).into();
    Some(astar(&graph, from, |finish| finish == to, |_| 1, |_| 0)?.1)
}

#[cfg(not(feature = "raw_result"))]
fn find_path_astar(
    from: (usize, usize),
    to: (usize, usize),
    maze: &Maze,
    graph: &UnGraph<usize, (usize, usize), usize>,
) -> Option<Vec<usize>> {
    Some(
        find_path_astar_inner(from, to, maze, graph)?
            .into_iter()
            .map(|node| node.index())
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maze_file() {
        let source = read_to_string("./resources/maze.txt").unwrap();
        let maze = load_maze("./resources/maze.txt");
        let processed = maze.to_string();
        assert_eq!(source, processed);
    }
}
