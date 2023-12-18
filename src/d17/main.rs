// #![deny(elided_lifetimes_in_paths)]

mod dijkstra;
mod minheap;
mod priorityqueue;
use dijkstra::dijkstra;
use std::error::Error;
use std::fs;

struct Grid<T> {
    width: usize,
    height: usize,
    items: Vec<T>,
}

impl<T> Grid<T> {
    fn get(&self, (r, c): (usize, usize)) -> &T {
        &self.items[self.width * r + c]
    }
}

fn parse(content: &str) -> Grid<usize> {
    let width = content.lines().next().unwrap().len();
    let height = content.lines().count();
    let items: Vec<_> = content
        .lines()
        .flat_map(|line| line.chars())
        .map(|ch| ch.to_string().parse::<usize>().unwrap()) // TODO: don't convert to string
        .collect();

    Grid {
        width,
        height,
        items,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

type Loc = (usize, usize);
type Node = (Loc, (Direction, usize));

fn get_children_fn_part1<'a, 'b>(
    node: &'a Node,
    grid: &'b Grid<usize>,
) -> impl Iterator<Item = (Node, usize)> + 'b {
    // let children_fn = |node: &Node| {
    use Direction::*;

    let &((r, c), (dir, run_length)) = node;
    let (r, c) = (r as isize, c as isize);

    // TODO: use ArrayVec
    let children: Vec<(Loc, Direction)> = [
        // all cardinal directions
        (r + 1, c, Down),
        (r - 1, c, Up),
        (r, c + 1, Right),
        (r, c - 1, Left),
    ]
    .into_iter()
    .filter(|&(new_r, new_c, _)| {
        // filter by still in grid
        0 <= new_r && new_r < (grid.height as isize) && 0 <= new_c && new_c < (grid.width as isize)
    })
    .filter(|&(_, _, newdir)| {
        // filter out can't go immediately in the previous direction (optimization to reduce
        // the branching factor)
        match dir {
            Direction::Up => newdir != Direction::Down,
            Direction::Down => newdir != Direction::Up,
            Direction::Left => newdir != Direction::Right,
            Direction::Right => newdir != Direction::Left,
        }
    })
    .map(|(r, c, d)| ((r.try_into().unwrap(), c.try_into().unwrap()), d))
    .collect();

    let children = children.into_iter().filter_map(move |(loc, this_dir)| {
        // increase run length
        let new_run_length = if this_dir == dir { run_length + 1 } else { 1 };
        if new_run_length >= 4 {
            return None;
        }
        Some((loc, (this_dir, new_run_length)))
    });

    let children = children.map(|node| {
        let weight = *grid.get(node.0);
        (node, weight)
    });

    children
}
fn main() -> Result<(), Box<dyn Error>> {
    // parse graph -> Grid
    // Make graphs
    //      Make graph1 from grid :: nodes are loc = (r, c), has a reference to the Grid
    //
    //      Make graph2 from grid :: nodes are ((r, c), n) where n  is the number of times you've been
    //      moving in that dir
    //
    //      Optional: instead of graphs use fn node -> [Node]. Maybe use smallvec
    // Graph

    let content = fs::read_to_string("src/d17/input")?;
    let grid = parse(&content);

    let src = (0, 0);
    let target = (grid.height - 1, grid.width - 1);
    let children_fn = |n: &Node| get_children_fn_part1(n, &grid);
    let (_, total_weight) = dijkstra(
        (src, (Direction::Right, 0)),
        |&(loc, _)| loc == target,
        children_fn,
    )
    .path_to_target()
    .unwrap();

    // let locations: Vec<_> = path.iter().map(|x| x.0).collect();
    // let values: Vec<_> = locations.iter().map(|&loc| grid.get(loc)).collect();

    // println!("{:?}", path);
    // println!("{:?}", values);
    println!("{:?}", total_weight);
    // let s = path.iter().map(|

    Ok(())
}
