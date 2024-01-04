// TODO: generalize start: T, end: T where they all are Eq, and Copy
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

/// A 1D Range [start, end)
impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(end >= start);
        Self { start, end }
    }
}

/// An NDRange ( [start1, end1), [start2, end2) ...). Forms an n-dimensional box
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NDRange<const N: usize>(pub [Range; N]);

impl Range {
    pub fn split(&self, val: usize) -> (Self, Self) {
        let &Range { start, end } = self;
        let split_point = val.clamp(start, end);
        let (left, right) = (Range::new(start, split_point), Range::new(split_point, end));
        (left, right)
    }
}

impl<const N: usize> NDRange<N> {
    pub fn new(data: [Range; N]) -> Self {
        NDRange(data)
    }
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|Range { start, end }| start == end)
    }

    pub fn split(&self, dimension: usize, val: usize) -> (Self, Self) {
        let mut left = self.clone();
        let mut right = self.clone();
        let (left_range, right_range) = self.0[dimension].split(val);
        left.0[dimension] = left_range;
        right.0[dimension] = right_range;
        (left, right)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_split_range() {
        let range = Range::new(1, 10);
        assert_eq!(range.split(0), (Range::new(1, 1), Range::new(1, 10)));
        assert_eq!(range.split(1), (Range::new(1, 1), Range::new(1, 10)));
        assert_eq!(range.split(5), (Range::new(1, 5), Range::new(5, 10)));
        assert_eq!(range.split(10), (Range::new(1, 10), Range::new(10, 10)));
        assert_eq!(range.split(11), (Range::new(1, 10), Range::new(10, 10)));
    }

    #[test]
    fn test_split_ndrange() {
        let ndrange = NDRange::new([
            Range::new(1, 10),
            Range::new(1, 10),
            Range::new(1, 10),
            Range::new(1, 10),
        ]);

        assert_eq!(
            ndrange.split(0, 0),
            (
                NDRange::new([
                    Range::new(1, 1),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ]),
                NDRange::new([
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ])
            )
        );

        assert_eq!(
            ndrange.split(0, 5),
            (
                NDRange::new([
                    Range::new(1, 5),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ]),
                NDRange::new([
                    Range::new(5, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ])
            )
        );

        assert_eq!(
            ndrange.split(0, 10),
            (
                NDRange::new([
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ]),
                NDRange::new([
                    Range::new(10, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ])
            )
        );

        assert_eq!(
            ndrange.split(1, 5),
            (
                NDRange::new([
                    Range::new(1, 10),
                    Range::new(1, 5),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ]),
                NDRange::new([
                    Range::new(1, 10),
                    Range::new(5, 10),
                    Range::new(1, 10),
                    Range::new(1, 10),
                ])
            )
        );
    }
}
