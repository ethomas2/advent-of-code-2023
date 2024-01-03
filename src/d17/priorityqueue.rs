use crate::minheap::Heap;
use core::cmp::Ordering;
use core::fmt;

/// A Wrapper around an arbitrary value just to make those arbitrary values trivially orderable.
/// That way they can be placed in a heap, which requires everything is orderable
pub struct OrdWrapper<T>(T);
impl<T> PartialOrd for OrdWrapper<T> {
    fn partial_cmp(&self, _: &Self) -> Option<Ordering> {
        Some(Ordering::Equal)
    }
}
impl<T> Ord for OrdWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl<T> PartialEq for OrdWrapper<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl<T> Eq for OrdWrapper<T> {}

impl<T: fmt::Debug> fmt::Debug for OrdWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)?;
        Ok(())
    }
}

pub struct PriorityQueue<V: PartialEq + Eq + PartialOrd + Ord, N> {
    heap: Heap<(V, OrdWrapper<N>)>,
}

impl<K, V> fmt::Debug for PriorityQueue<K, V>
where
    K: fmt::Debug + PartialEq + Eq + PartialOrd + Ord,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.heap)?;
        Ok(())
    }
}

impl<V: PartialEq + Eq + PartialOrd + Ord, N: Eq> PriorityQueue<V, N> {
    pub fn new() -> Self {
        PriorityQueue { heap: Heap::new() }
    }

    pub fn push(&mut self, node: N, val: V) {
        self.heap.push((val, OrdWrapper(node)));
    }

    pub fn pop(&mut self) -> Option<(N, V)> {
        self.heap.pop_first().map(|(v, n)| (n.0, v))
    }

    pub fn update<F>(&mut self, node: &N, update_fn: F) -> bool
    where
        F: FnOnce(&mut V),
    {
        // TODO: don't use position here. It's O(n). Keep an auxilary hash map
        if let Some(idx) = self.heap.position(|(_, OrdWrapper(n))| n == node) {
            let (mut v, ord_wrapper) = self.heap.pop_idx(idx).unwrap();
            update_fn(&mut v);
            self.heap.push((v, ord_wrapper));
            return true;
        }
        return false;
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn contains(&self, node: &N) -> bool {
        // TODO: don't use position here. It's O(n). Keep an auxilary hash map
        matches!(
            self.heap.position(|(_, OrdWrapper(ref n))| n == node),
            Some(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_pop() {
        let mut pq = PriorityQueue::new();
        pq.push("node1", 10);
        pq.push("node2", 5);
        pq.push("node3", 20);

        assert_eq!(pq.pop(), Some(("node2", 5)));
        assert_eq!(pq.pop(), Some(("node1", 10)));
        assert_eq!(pq.pop(), Some(("node3", 20)));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_update() {
        let mut pq = PriorityQueue::new();
        pq.push("node1", 10);
        pq.push("node2", 15);
        pq.push("node3", 20);

        // Update node2's value to 5
        pq.update(&"node2", |val| *val = 5);

        assert_eq!(pq.pop(), Some(("node2", 5)));
        assert_eq!(pq.pop(), Some(("node1", 10)));
        assert_eq!(pq.pop(), Some(("node3", 20)));
        assert_eq!(pq.pop(), None);
    }

    // #[test]
    // fn test_foo() {

    //     queue = [(6, ((0, 0), (Up, 1))), (8, ((0, 1), (Right, 1))), (10, ((1, 1), (Right, 1))), (10, ((1, 0), (Up, 1))), (14, ((2, 1), (Right, 1)))]
    // }
}
