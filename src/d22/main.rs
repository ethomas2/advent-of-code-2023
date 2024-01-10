use itertools::Itertools;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fs;
// use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Hash, Clone)] // TODO: clone is only needed for debugging
struct Brick {
    name: usize,
    xrange: std::ops::RangeInclusive<usize>,
    yrange: std::ops::RangeInclusive<usize>,
    zrange: std::ops::RangeInclusive<usize>,
}

fn parse(input: &str) -> Result<Vec<Brick>, String> {
    // let mut names = 0..;
    let mut names = 0..;
    let bricks: Vec<_> = input
        .lines()
        .map(|line| {
            let parts: Vec<_> = line.split("~").collect(); // TODO: don't allocate vector
            debug_assert!(parts.len() == 2); // TODO: return Error
            let mut ranges = parts[0]
                .split(",")
                .zip(parts[1].split(","))
                .map(|(ch1, ch2)| {
                    let x1 = ch1.parse::<usize>().unwrap();
                    let x2 = ch2.parse::<usize>().unwrap();
                    x1.min(x2)..=x1.max(x2)
                });
            let brick = Brick {
                name: names.next().unwrap(),
                xrange: ranges.next().unwrap(),
                yrange: ranges.next().unwrap(),
                zrange: ranges.next().unwrap(),
            };
            debug_assert!(ranges.next() == None);
            brick
        })
        .collect();
    Ok(bricks)
}

type Loc = (usize, usize, usize);
impl Brick {
    fn locations(&self) -> impl Iterator<Item = Loc> {
        let (xrange, yrange, zrange) = (
            self.xrange.clone(),
            self.yrange.clone(),
            self.zrange.clone(),
        );
        xrange
            .cartesian_product(yrange)
            .cartesian_product(zrange)
            .map(|((x, y), z)| (x, y, z))
    }

    fn fall(&self) -> Brick {
        let (xrange, yrange, mut zrange) = (
            self.xrange.clone(),
            self.yrange.clone(),
            self.zrange.clone(),
        );
        zrange = (zrange.start() - 1)..=(zrange.end() - 1);
        Brick {
            name: self.name,
            xrange,
            yrange,
            zrange,
        }
    }

    fn fall_checked(&self, occupied_map: &OccupiedMap) -> Option<Brick> {
        let minz = *self.zrange.start();
        if minz <= 1 {
            return None;
        }
        let bottom_layer = self.locations().filter(|(_, _, z)| z == &minz);
        let mut translated_bottom_layer = bottom_layer.map(|(x, y, z)| (x, y, z - 1));
        if translated_bottom_layer.any(|tup| occupied_map.contains_key(&tup)) {
            return None;
        }
        return Some(self.fall());
    }
}

fn bricks_above<'a>(
    brick: &'a Brick,
    occupied_map: &'a OccupiedMap,
) -> impl Iterator<Item = &'a Brick> + 'a + Clone {
    brick
        .locations()
        .filter_map(|(x, y, z)| occupied_map.get(&(x, y, z + 1)))
        .sorted_by_key(|x| Arc::as_ptr(x))
        .unique_by(|x| Arc::as_ptr(x))
        .filter(|brick_above| Arc::as_ptr(brick_above) != brick)
        .map(|x| &**x)
}

fn bricks_below<'a>(
    brick: &'a Brick, // TODO: any reason this can't take an Rc?
    occupied_map: &'a OccupiedMap,
) -> impl Iterator<Item = &'a Brick> + 'a {
    brick
        .locations()
        .filter_map(|(x, y, z)| occupied_map.get(&(x, y, z - 1)))
        .sorted_by_key(|x| Arc::as_ptr(x))
        .unique_by(|x| Arc::as_ptr(x))
        .filter(|brick_below| Arc::as_ptr(brick_below) != brick)
        .map(|x| &**x)
}

fn integrity_check(bricks: &Vec<Arc<Brick>>, occupied_map: &OccupiedMap) {
    #[cfg(debug_assertions)]
    {
        // each brick has all locations mapped to itself
        for brick in bricks {
            for loc in brick.locations() {
                debug_assert!(occupied_map.get(&loc) == Some(brick));
            }
        }

        // occupied map has no extras
        debug_assert!(occupied_map.len() == bricks.iter().flat_map(|b| b.locations()).count());
    }
}

/// Returns if destroying this brick would cause things to fall
/// A brick if "fragile" if at least one of the bricks above it has the property that this brick is
/// the only brick below it
fn is_fragile(brick: &Brick, occupied_map: &OccupiedMap) -> bool {
    let mut bricks_above_me = bricks_above(brick, occupied_map);

    // a brick is "fragile" (ie not destroyable) if at least one of the bricks above it has
    // the property that this brick is the only brick below it
    let fragile = bricks_above_me.any(|brick_above| {
        // debug_assert!(bricks_below(&brick_above, &occupied_map)
        //     .any(|below_above| Rc::as_ptr(&below_above) == Rc::as_ptr(&brick)));
        bricks_below(&brick_above, occupied_map).count() == 1
    });
    fragile
}

type OccupiedMap = HashMap<Loc, Arc<Brick>>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CollapseState {
    Collapsed,
    Stable,
}

fn collapse_state(
    brick: &Brick,
    occupied_map: &OccupiedMap,
    cache: &mut HashMap<&Brick, CollapseState>,
) -> CollapseState {
    if let Some(ans) = cache.get(brick) {
        return *ans;
    }

    let all_below_collapsed = bricks_below(brick, occupied_map).count() > 0
        && bricks_below(brick, occupied_map)
            .all(|b| collapse_state(b, occupied_map, cache) == CollapseState::Collapsed);
    if all_below_collapsed {
        return CollapseState::Collapsed;
    }

    return CollapseState::Stable;
}

fn main() -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string("src/d22/input")?;
    let mut bricks: Vec<_> = parse(&content)?
        .into_iter()
        .map(|brick| Arc::new(brick))
        .collect();
    bricks.sort_by_key(|brick| *brick.zrange.start());

    let mut occupied_map: OccupiedMap = bricks
        .iter()
        .flat_map(|brick| brick.locations().map(move |loc| (loc, brick.clone())))
        .collect();

    // make all bricks fall
    {
        // for each brick
        for i in 0..bricks.len() {
            // fall until you can't
            loop {
                integrity_check(&bricks, &occupied_map);
                let brick = bricks[i].clone();
                let newbrick_opt = brick.fall_checked(&occupied_map);
                match newbrick_opt {
                    None => break,
                    Some(newbrick) => {
                        let newbrick = Arc::new(newbrick);
                        brick.locations().for_each(|loc| {
                            occupied_map.remove(&loc);
                        });
                        newbrick.locations().for_each(|loc| {
                            occupied_map.insert(loc, newbrick.clone());
                        });
                        bricks[i] = newbrick;
                    }
                }
            }
        }
    }

    // check which ones are destroyable
    {
        let destroyable_bricks = bricks
            .iter()
            .filter(|&brick| !is_fragile(&**brick, &occupied_map));
        println!("p1 {}", destroyable_bricks.count());
    }

    // part 2: for each fragile brick determine how many bricks are above it
    // Correct: 70727
    {
        let brick_refs: Vec<&Brick> = bricks.iter().map(|b| &**b).collect();
        let p2: usize = brick_refs
            .par_iter()
            .map(|&brick_to_collapse| {
                // TODO: turn cache in to Vec
                let mut cache = HashMap::with_capacity(brick_refs.len() * 2);
                cache.insert(brick_to_collapse, CollapseState::Collapsed);

                let collapsed_brick_count = brick_refs
                    .iter()
                    .filter(|b| {
                        let b = **b;
                        collapse_state(b, &occupied_map, &mut cache) == CollapseState::Collapsed
                    })
                    .count();

                collapsed_brick_count - 1
            })
            .sum();
        println!("p2 {}", p2);
    }

    Ok(())
}
