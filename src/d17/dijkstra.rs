use crate::priorityqueue::PriorityQueue;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

// struct DijkstraState<N>
// where
//     N: PartialOrd + Ord + PartialEq + Eq + Default + Hash,
// {
//     start: N,
//     predecessor_map: HashMap<N, Option<N>>,
//     queue: minheap::Heap<N>,
// }

// TODO: if the only public function is path_to_target this doesn't need the marked list
pub struct DijkstraState<N> {
    predecessor_map: HashMap<N, (N, usize)>,
    target: Option<N>,
    src: N,
    marked: HashSet<N>,
}

impl<N: fmt::Debug + Clone + Eq + Hash> DijkstraState<N> {
    pub fn path_to_target(&self) -> Option<(Vec<N>, usize)> {
        let t = self.target.as_ref()?;
        debug_assert!(self.marked.contains(&t));
        let path = self.path_to_node(t);
        path
    }
    fn path_to_node<'a: 'b, 'b>(&'a self, mut node: &'b N) -> Option<(Vec<N>, usize)> {
        let (_, weight) = self.predecessor_map.get(node)?;
        let total_weight = *weight;

        let mut path = Vec::new();
        path.push(node.clone());
        while let Some((pred, _)) = self.predecessor_map.get(node) {
            path.push(pred.clone());
            if pred == &self.src {
                path.reverse();
                return Some((path, total_weight));
            }
            node = pred;
        }
        panic!("oh no");
    }
}

pub fn dijkstra<N, F, TF, R>(src: N, target_fn: TF, children: F) -> DijkstraState<N>
where
    // TODO: Consider making predecessor_map a BTree then you don't
    // need Hash. Or making it generic over BTreeMap/Hash
    // TODO: use Rc so you don't have to Clone
    N: PartialEq + Eq + Hash + Clone + fmt::Debug,
    F: Fn(&N) -> R,
    TF: Fn(&N) -> bool,
    R: Iterator<Item = (N, usize)>,
{
    // Ideas behind dijkstra:
    //      1. If (x1, x2, ... xn) is a shortest path from x1 to xn (where x1 = s), then all
    //         prefixes of that path must be the shortest path from x1 to that node. I.e (x1, x2)
    //         is a shortest path, (x1, x2, x3) is a shortest path, (x1 ... xk) for all k < n.
    //      2. Let M bet the set of k closest nodes to s. M = {m1, m2, ... mk} where m1 = s.
    //         By idea #1, the path from m1 to m_(k+1) must all be contained in M. In particular,
    //         the node right before m_(k_1) must be in M
    //
    // Algorithm:
    //      Initialize a queue Q = {s}
    //      while Q is not empty:
    //          u <- Q.pop()  // Q is the set S - M
    //          yield u       // the next mi that we haven't yield yet
    //          for u in children(u):
    //              add u to Q or update u in Q if it already exists
    //
    // So Q contains nodes for which we have not identified the shortest path. Q is a min heap, so
    // when we pop something from Q it's the closest element to s not in M. M is the set of all
    // nodes that we have yielded (not mentioned in the algorithm pseudocode, but it's implicitly
    // there). RELAX(u, v) adds elements to the queue if they're not there already, or updates
    // their shortest distance from s if they are in there. The term RELAX is borrowed from CLRS

    // TODO: lots of clone in here. Should probably RC in the queue and predcessaor map. Maybe with
    // an arena

    let mut queue: PriorityQueue<usize, N> = PriorityQueue::new();
    queue.push(src.clone(), 0);

    let mut state = DijkstraState {
        src,
        target: None,
        // map from v -> (u, <weight of path to v>)
        predecessor_map: HashMap::new(),
        // marked is the set of nodes we've already yielded. M in the psuedocode above
        marked: HashSet::new(),
    };
    let DijkstraState {
        ref mut target,
        ref mut predecessor_map,
        ref mut marked,
        ..
    } = state;

    while queue.len() > 0 {
        let (u, best_path_to_u) = queue.pop().unwrap();

        marked.insert(u.clone());
        if target_fn(&u) {
            *target = Some(u);
            return state; // TODO: construct DijkstraState here
        }

        for (v, weight) in children(&u) {
            // this is a new node. Add it to the queue
            if !queue.contains(&v) && !marked.contains(&v) {
                queue.push(v.clone(), best_path_to_u + weight);
                predecessor_map.insert(v.clone(), (u.clone(), best_path_to_u + weight));
            } else {
                // v is already in queue. Update it if this path is better
                queue.update(&v, |best_path_to_v: &mut usize| {
                    if best_path_to_u + weight < *best_path_to_v {
                        *best_path_to_v = best_path_to_u + weight;
                        *(predecessor_map.get_mut(&v).unwrap()) = (u.clone(), *best_path_to_v);
                    }
                });
            }
        }
    }

    return state; // TODO: construct DijkstraState here
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test1() {
        // let nodes = vec!['a', 'b', 'c', 'd', 'e'];
        let edges = vec![
            ('a', 'e', 10),
            ('a', 'b', 1),
            ('b', 'c', 1),
            ('c', 'd', 1),
            ('d', 'e', 1),
        ];
        let children_fn = |node: &char| {
            let children: Vec<_> = edges
                .iter()
                .filter_map(|&(src, dst, weight)| {
                    if &src == node {
                        return Some((dst, weight));
                    }
                    return None;
                })
                .collect();
            children.into_iter()
        };
        let (path, _) = dijkstra('a', |&x| x == 'e', children_fn)
            .path_to_target()
            .unwrap();
        assert_eq!(path, vec!['a', 'b', 'c', 'd', 'e']);
    }

    #[test]
    fn test2() {
        // let nodes = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'];
        let edges = vec![
            ('a', 'b', 1),
            ('a', 'c', 1),
            ('a', 'd', 5),
            ('b', 'e', 1),
            ('b', 'f', 2),
            ('c', 'f', 1),
            ('d', 'g', 1),
            ('f', 'g', 5),
            ('g', 'h', 1),
            ('h', 'i', 1),
            ('e', 'i', 10),
        ];
        let children_fn = |&node: &char| {
            edges.iter().filter_map(move |&(src, dst, weight)| {
                if src == node {
                    return Some((dst, weight));
                }
                return None;
            })
        };
        let (path, _) = dijkstra('a', |&x| x == 'i', children_fn)
            .path_to_target()
            .unwrap();
        assert_eq!(path, vec!['a', 'd', 'g', 'h', 'i']);
    }

    #[test]
    fn test3() {
        let edges = vec![('a', 'b', 1), ('a', 'c', 1), ('d', 'e', 1)];
        let children_fn = |&node: &char| {
            edges.iter().filter_map(move |&(src, dst, weight)| {
                if src == node {
                    return Some((dst, weight));
                }
                return None;
            })
        };

        assert!(matches!(
            dijkstra('a', |&x| x == 'd', children_fn).path_to_target(),
            None
        ));
    }
}
