use clap::{App, Arg};
use petgraph::{
    algo::astar,
    graph::{NodeIndex, UnGraph},
};
use std::{fs::read_to_string, path::Path, time::Instant};

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

fn main() {
    let matches = App::new("Rust Maze Solver Benchmarks")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .value_name("FILE")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("attempts")
                .short("a")
                .long("attempts")
                .value_name("NUMBER")
                .takes_value(true)
                .default_value("1")
                .required(false),
        )
        .arg(
            Arg::with_name("tries")
                .short("t")
                .long("tries")
                .value_name("NUMBER")
                .takes_value(true)
                .default_value("1")
                .required(false),
        )
        .arg(
            Arg::with_name("searches")
                .short("s")
                .long("searches")
                .value_name("NUMBER")
                .takes_value(true)
                .default_value("1")
                .required(false),
        )
        .arg(
            Arg::with_name("point")
                .short("p")
                .long("point")
                .value_name("FromX:FromX,ToX:ToY")
                .takes_value(true)
                .multiple(true)
                .required(false),
        )
        .get_matches();
    let input = matches.value_of("input").unwrap();
    let attempts = matches
        .value_of("attempts")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();
    let tries = matches
        .value_of("tries")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();
    let searches = matches
        .value_of("searches")
        .unwrap_or("1")
        .parse::<usize>()
        .unwrap();
    let points = matches.values_of("point").map(|points| {
        points
            .map(|item| {
                let parts = item
                    .split(',')
                    .filter(|part| !part.trim().is_empty())
                    .collect::<Vec<_>>();
                if parts.len() != 2 {
                    panic!("Points should have 2 elements!");
                }
                let parts = parts
                    .into_iter()
                    .map(|part| {
                        let pairs = part
                            .split(':')
                            .filter(|part| !part.trim().is_empty())
                            .collect::<Vec<_>>();
                        if pairs.len() != 2 {
                            panic!("Point should have 2 elements!");
                        }
                        let x = pairs[0].parse::<usize>().unwrap();
                        let y = pairs[1].parse::<usize>().unwrap();
                        (x, y)
                    })
                    .collect::<Vec<_>>();
                (parts[0], parts[1])
            })
            .collect::<Vec<_>>()
    });
    let maze = load_maze(input);
    let graph = build_petgraph(&maze);
    if let Some(points) = points {
        for point in &points {
            let from = point.0;
            let to = point.1;
            let mut durations = Vec::with_capacity(tries);
            for i in 0..tries {
                let mut path = None;
                let timer = Instant::now();
                for _ in 0..attempts {
                    path = astar!(from, to, &maze, &graph);
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
                println!("Min: {:?} | Max: {:?} | Diff: {:?}", min, max, *max - *min);
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
            if let Some(mut path) = astar!(from, to, &maze, &graph) {
                for _ in 1..attempts {
                    path = astar!(from, to, &maze, &graph).unwrap();
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

#[derive(Debug, Clone)]
struct Maze {
    pub cols: usize,
    pub rows: usize,
    pub tiles: Vec<bool>,
}

impl Maze {
    #[inline(always)]
    pub fn get_index(&self, col: usize, row: usize) -> Option<usize> {
        if col >= self.cols || row >= self.rows {
            None
        } else {
            Some(row * self.cols + col)
        }
    }

    pub fn get_index_with_tile(&self, col: usize, row: usize) -> Option<(usize, bool)> {
        if let Some(index) = self.get_index(col, row) {
            Some((index, self.tiles[index]))
        } else {
            None
        }
    }
}

impl ToString for Maze {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(self.cols * self.rows + self.rows * 2);
        for row in 0..self.rows {
            for col in 0..self.cols {
                let value = self.tiles[row * self.cols + col];
                result.push(if value { '#' } else { ' ' });
            }
            result.push('\r');
            result.push('\n');
        }
        result
    }
}

fn load_maze<P>(path: P) -> Maze
where
    P: AsRef<Path>,
{
    let contents = read_to_string(path.as_ref()).unwrap();
    let lines = contents
        .split(|c| c == '\r' || c == '\n')
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        panic!("Maze file is empty!");
    }
    let cols = lines[0].len();
    let rows = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if line.len() != cols {
            panic!(
                "Number of columns ({}) at line {} is different than expected: {}",
                line.len(),
                i,
                cols
            );
        }
    }
    let tiles = lines
        .into_iter()
        .flat_map(|line| line.chars().map(|c| c == '#'))
        .collect::<Vec<_>>();
    if tiles.len() != cols * rows {
        panic!("Expected {} tiles, got {}", cols * rows, tiles.len());
    }
    Maze { cols, rows, tiles }
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
