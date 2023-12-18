use std::fmt;
/// A heap with a pop_idx(idx) operation
pub struct Heap<T: PartialOrd + Ord + PartialEq + Eq> {
    items: Vec<T>,
}

impl<T> fmt::Debug for Heap<T>
where
    T: fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.items)?;
        Ok(())
    }
}

impl<T: PartialOrd + Ord + PartialEq + Eq> Heap<T> {
    pub fn new() -> Self {
        Heap { items: Vec::new() }
    }
    pub fn push(&mut self, val: T) {
        self.items.push(val);
        let idx = self.items.len() - 1;
        self.sift_up(idx);
    }
    pub fn pop_first(&mut self) -> Option<T> {
        self.pop_idx(0)
    }

    fn sift_down(&mut self, mut idx: usize) -> bool {
        let mut mutated = false;
        loop {
            let idx_to_swap_with = [2 * (idx + 1) - 1, 2 * (idx + 1)]
                .into_iter()
                .filter(|&candidate_idx| {
                    candidate_idx < self.items.len() && self.items[candidate_idx] < self.items[idx]
                })
                .min_by(|&x, &y| self.items[x].cmp(&self.items[y]));
            match idx_to_swap_with {
                None => break,
                Some(idx2) => {
                    self.items.swap(idx, idx2);
                    idx = idx2;
                    mutated = true;
                }
            }
        }
        return mutated;
    }

    fn sift_up(&mut self, mut idx: usize) -> bool {
        let mut mutated = false;
        while idx > 0 && self.items[(idx + 1) / 2 - 1] > self.items[idx] {
            // self.items.split_at_mut
            self.items.swap((idx + 1) / 2 - 1, idx);
            mutated = true;
            idx = (idx + 1) / 2 - 1;
        }
        return mutated;
    }

    pub fn pop_idx(&mut self, idx: usize) -> Option<T> {
        if idx >= self.items.len() {
            return None;
        }
        if self.items.len() == 0 {
            return None;
        }

        let n = self.items.len();

        // x is the last element
        let mut x = (&mut self.items).drain((n - 1)..).next().unwrap();
        if idx == n - 1 {
            return Some(x);
        }

        // make x the element to return, but items[idx] might be too big and violate the heap
        // property
        debug_assert!(idx < self.items.len());
        std::mem::swap(&mut x, &mut self.items[idx]);

        // restore the heap property
        self.sift_down(idx);
        self.sift_up(idx);

        Some(x)
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn position<F>(&self, condition: F) -> Option<usize>
    where
        F: Fn(&T) -> bool,
    {
        // TODO: make this not O(n). Keep an auxilary hash map
        let idx = self.items.iter().position(condition)?;
        debug_assert!(idx < self.items.len());
        Some(idx)
    }

    // pub fn contains(&self, val: &T) -> bool {
    //     matches!(self.position(|x| x == val), Some(_))
    // }

    // pub fn drain_all(mut self) -> impl Iterator<Item = T> {
    //     std::iter::from_fn(move || self.pop_first())
    // }
}

#[cfg(test)]
mod test {
    use super::*;

    fn obeys_heap_property<T: Ord>(items: &[T]) -> bool {
        for i in 1..items.len() {
            let parent_idx = (i + 1) / 2 - 1;
            if items[i] < items[parent_idx] {
                // Ensure that T is the same size as i32
                assert_eq!(mem::size_of::<T>(), mem::size_of::<i32>());

                unsafe {
                    let items: &[i32] = mem::transmute(items);
                    println!("{:?} fails heap property at {}", items, i);
                }
                return false;
            }
        }
        return true;
    }
    #[test]
    fn test_push() {
        let sorted_arr = [1, 2, 3, 4, 5, 6, 7, 8];
        let permutations = sorted_arr.into_iter().permutations(sorted_arr.len());
        for perm in permutations {
            let mut heap = Heap::new();
            for x in perm {
                heap.push(x);
                assert!(obeys_heap_property(&heap.items));
            }

            let drained = {
                let mut d = Vec::new();
                while heap.len() > 0 {
                    d.push(heap.pop_first().unwrap());
                    assert!(obeys_heap_property(&heap.items));
                }
                d
            };

            assert_eq!(drained, sorted_arr);
            assert_eq!(heap.pop_first(), None);
        }
    }

    #[test]
    fn test_pop_idx() {
        let sorted_arr = [1, 2, 3, 4, 5, 6, 7, 8];
        let permutations = sorted_arr.into_iter().permutations(sorted_arr.len());
        for perm in permutations {
            for idx in 0..sorted_arr.len() {
                let mut heap = {
                    let mut heap = Heap::new();
                    for &x in &perm {
                        heap.push(x);
                        assert!(obeys_heap_property(&heap.items));
                    }
                    heap
                };

                let elm = heap.items[idx];
                println!("before pop items={:?} idx={:?}", &heap.items, idx);
                assert_eq!(elm, heap.pop_idx(idx).unwrap());
                assert!(obeys_heap_property(&heap.items));

                let drained = {
                    let mut d = Vec::new();
                    while heap.len() > 0 {
                        d.push(heap.pop_first().unwrap());
                        assert!(obeys_heap_property(&heap.items));
                    }
                    d
                };
                let is_sorted = drained.iter().tuple_windows().all(|(a, b)| a < b);
                assert!(is_sorted, "failed {perm:?} {idx}", perm = perm, idx = idx);
            }
        }
    }
}
