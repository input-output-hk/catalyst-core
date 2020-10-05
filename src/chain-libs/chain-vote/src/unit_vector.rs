pub use crate::gang::Scalar;

#[derive(Clone, Copy)]
/// Represent a Unit vector which size is @size and the @ith element (0-indexed) is enabled
pub struct UnitVector {
    ith: usize,
    size: usize,
}

impl std::fmt::Debug for UnitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_{}({})", self.ith, self.size)
    }
}

impl std::fmt::Display for UnitVector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "e_{}({})", self.ith, self.size)
    }
}

// `is_empty` cannot ever be useful in the case of UnitVector,
// as the size will always be > 0 as enforced in new()
#[allow(clippy::len_without_is_empty)]
impl UnitVector {
    /// Create a new
    pub fn new(size: usize, ith: usize) -> Self {
        assert!(size > 0);
        assert!(ith < size);
        UnitVector { ith, size }
    }

    pub fn iter(&self) -> UnitVectorIter {
        UnitVectorIter(0, *self)
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn ith(&self) -> usize {
        self.ith
    }

    pub fn is_jth(&self, j: usize) -> bool {
        if j >= self.size {
            panic!(
                "out of bounds: unit vector {} accessing index {}",
                self.size, j
            );
        } else {
            j == self.ith
        }
    }

    pub fn jth(&self, j: usize) -> Scalar {
        if j >= self.size {
            panic!(
                "out of bounds: unit vector {} accessing index {}",
                self.size, j
            );
        } else if j == self.ith {
            Scalar::one()
        } else {
            Scalar::zero()
        }
    }
}

pub fn binrep(n: usize, digits: u32) -> Vec<bool> {
    assert!(n < 2usize.pow(digits));
    (0..digits)
        .rev()
        .map(|i: u32| (n & (1 << i)) != 0)
        .collect::<Vec<bool>>()
}

#[derive(Clone, Copy)]
pub struct UnitVectorIter(usize, UnitVector);

impl Iterator for UnitVectorIter {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.0;
        if i == self.1.size {
            None
        } else {
            self.0 += 1;
            Some(i == self.1.ith)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_vector() {
        let uv = UnitVector::new(5, 0);
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            [true, false, false, false, false]
        );
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            &(0usize..5).map(|i| uv.is_jth(i)).collect::<Vec<_>>()[..]
        );

        let uv = UnitVector::new(5, 4);
        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            [false, false, false, false, true]
        );

        assert_eq!(
            &uv.iter().collect::<Vec<_>>()[..],
            &(0usize..5).map(|i| uv.is_jth(i)).collect::<Vec<_>>()[..]
        );
    }

    #[test]
    fn unit_binrep() {
        assert_eq!(binrep(3, 5), &[false, false, false, true, true])
    }
}
