mod benchmarks;

use crate::benchmarks::petgraph::benchmark_petgraph;
use clap::{App, Arg};
use std::{fs::read_to_string, path::Path};

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
    benchmark_petgraph(&maze, points, attempts, tries, searches);
}

#[derive(Debug, Clone)]
pub struct Maze {
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
