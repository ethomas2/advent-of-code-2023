// #![deny(elided_lifetimes_in_paths)]

mod dijkstra;
mod minheap;
mod priorityqueue;
use dijkstra::dijkstra;
use itertools::Itertools;
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
type Part1Node = (Loc, (Direction, usize));

fn get_children_fn_part1<'a, 'b>(
    node: &'a Part1Node,
    grid: &'b Grid<usize>,
) -> impl Iterator<Item = (Part1Node, usize)> + 'b {
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

const ALL_DIRS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

type Part2Node = (Loc, Option<Direction>, usize);
fn get_children_fn_part2<'a, 'b>(
    node: &'a Part2Node,
    grid: &'b Grid<usize>,
) -> impl Iterator<Item = (Part2Node, usize)> + 'b {
    let &((r, c), dir, runlength) = node;
    let (r, c) = (r as isize, c as isize);

    let mut candidate_nodes = Vec::new();
    for newdir in ALL_DIRS {
        let newloc = match newdir {
            Direction::Up => (r - 1, c),
            Direction::Down => (r + 1, c),
            Direction::Left => (r, c - 1),
            Direction::Right => (r, c + 1),
        };
        let new_runlength = if Some(newdir) == dir {
            runlength + 1
        } else {
            1
        };
        candidate_nodes.push((newloc, Some(newdir), new_runlength));
    }

    let (width, height) = (grid.width as isize, grid.height as isize);
    // if (r, c) == (4, 0) {
    //     println!("{:?}", candidate_nodes);
    // }
    let nodes = candidate_nodes
        .into_iter()
        .filter(move |&((r, c), _, _)| 0 <= r && r < height && 0 <= c && c < width)
        // TODO: can I remove the move here?
        .filter(move |&(_, newdir, _)| {
            if runlength < 4 {
                dir.is_none() || newdir == dir
            } else {
                true
            }
        })
        .filter(|&(_, _, runlength)| runlength <= 10)
        .map(|((r, c), newdir, new_runlength)| {
            let (r, c) = (r as usize, c as usize);
            let weight = *grid.get((r, c));
            let newnode = ((r, c), newdir, new_runlength);
            (newnode, weight)
        });

    let (nodes, dbg_nodes) = nodes.tee();
    let dbg_nodes: Vec<_> = dbg_nodes.collect();
    // if (r, c) == (4, 0) {
    //     println!("src {:?}  result {:?}", node, dbg_nodes);
    // }

    nodes
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

    // part 1
    {
        let src = (0, 0);
        let target = (grid.height - 1, grid.width - 1);
        let children_fn = |n: &Part1Node| get_children_fn_part1(n, &grid);
        let (_, total_weight) = dijkstra(
            (src, (Direction::Right, 0)),
            |&(loc, _)| loc == target,
            children_fn,
        )
        .path_to_target()
        .unwrap();

        println!("{:?}", total_weight);
    }

    // part 2
    {
        let src = ((0, 0), None, 0);
        let target = (grid.height - 1, grid.width - 1);
        let children_fn = |n: &Part2Node| get_children_fn_part2(n, &grid);
        let (path, total_weight) = dijkstra(
            src,
            |&(loc, _, runlength)| loc == target && runlength >= 4,
            children_fn,
        )
        .path_to_target()
        .unwrap();

        println!("{:?}", path);
        println!("{:?}", total_weight);
    }

    Ok(())
}
