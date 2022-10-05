use std::{
    convert::{TryFrom, TryInto},
    fmt::{self, Display},
    ops::Deref,
    str,
};
use thiserror::Error;

/// the soft derivation is upper bounded, this is the value
const SOFT_DERIVATION_UPPER_BOUND: u32 = 0x8000_0000;

/// a derivation value that can be used to derive keys
///
/// There is 2 kind of derivations, the soft and the hard derivations.
/// [`SoftDerivation`] are expected to allow derivation of the private
/// keys and of the public keys. [`HardDerivation`] are expected to allow
/// only the derivation of the private keys.
///
/// [`SoftDerivation`]: ./struct.SoftDerivation.html
/// [`HardDerivation`]: ./struct.HardDerivation.html
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Derivation(u32);

/// wrapper to guarantee the given derivation is a soft derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SoftDerivation(Derivation);

/// wrapper to guarantee the given derivation is a soft derivation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HardDerivation(Derivation);

/// iterator to create derivation values
///
/// # Examples
///
/// ```
/// # use chain_path_derivation::{Derivation, DerivationRange};
/// let range = DerivationRange::new(..20);
/// for (expected, derivation) in (0..20).zip(range) {
///     assert_eq!(derivation, Derivation::new(expected));
/// }
/// ```
#[derive(Debug)]
pub struct DerivationRange {
    range: std::ops::Range<Derivation>,
}

/// iterator to create derivation values
///
/// # Examples
///
/// ```
/// # use chain_path_derivation::{Derivation, SoftDerivation, SoftDerivationRange};
/// let range = SoftDerivationRange::new(..20);
/// for (expected, derivation) in (0..20).zip(range) {
///     assert_eq!(
///         derivation,
///         SoftDerivation::new_unchecked(Derivation::new(expected))
///     );
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SoftDerivationRange {
    range: std::ops::Range<SoftDerivation>,
}

/// iterator to create derivation values
///
/// # Examples
///
/// ```
/// # use chain_path_derivation::{Derivation, HardDerivation, HardDerivationRange};
/// let range = HardDerivationRange::new(..0x8000_0020);
/// for (expected, derivation) in (0x8000_0000..0x8000_0020).zip(range) {
///     assert_eq!(
///         derivation,
///         HardDerivation::new_unchecked(Derivation::new(expected))
///     );
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HardDerivationRange {
    range: std::ops::Range<HardDerivation>,
}

#[derive(Debug, Error)]
pub enum DerivationError {
    #[error("Not a valid derivation for a soft derivation ({0})")]
    InvalidSoftDerivation(Derivation),
    #[error("Not a valid derivation for a hard derivation ({0})")]
    InvalidHardDerivation(Derivation),
}

impl Derivation {
    /// create a new derivation with the given index
    #[inline]
    pub const fn new(v: u32) -> Self {
        Self(v)
    }

    /// test if the given derivation is a soft derivation
    ///
    /// # Example
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// let derivation = Derivation::new(42);
    /// assert!(derivation.is_soft_derivation());
    /// ```
    #[inline]
    pub fn is_soft_derivation(self) -> bool {
        self.0 < SOFT_DERIVATION_UPPER_BOUND
    }

    /// test if the given derivation is a hard derivation
    ///
    /// # Example
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// let derivation = Derivation::new(0x8000_0010);
    /// assert!(derivation.is_hard_derivation());
    /// ```
    #[inline]
    pub fn is_hard_derivation(self) -> bool {
        !self.is_soft_derivation()
    }

    /// returns the max derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// let max = Derivation::max_value();
    /// assert_eq!(max, Derivation::new(4294967295));
    /// ```
    #[inline]
    pub const fn max_value() -> Self {
        Self::new(u32::max_value())
    }

    /// returns the min derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// let min = Derivation::min_value();
    /// assert_eq!(min, Derivation::new(0));
    /// ```
    #[inline]
    pub const fn min_value() -> Self {
        Self::new(u32::min_value())
    }

    /// calculate `derivation + rhs`
    ///
    /// Returns the tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// assert_eq!(
    ///     Derivation::new(5).overflowing_add(2),
    ///     (Derivation::new(7), false)
    /// );
    /// assert_eq!(
    ///     Derivation::max_value().overflowing_add(1),
    ///     (Derivation::new(0), true)
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn overflowing_add(self, rhs: u32) -> (Self, bool) {
        let (v, b) = self.0.overflowing_add(rhs);
        (Self(v), b)
    }

    /// saturating integer addition. Computes `self + rhs`, saturating
    /// at the numeric bounds instead of overflowing.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// assert_eq!(Derivation::new(100).saturating_add(1), Derivation::new(101));
    /// assert_eq!(Derivation::max_value().saturating_add(2048), Derivation::max_value());
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn saturating_add(self, rhs: u32) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    /// checked integer addition. Computes `self + rhs`, returning `None` if overflow
    /// would occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// assert_eq!(Derivation::new(100).checked_add(1), Some(Derivation::new(101)));
    /// assert_eq!(Derivation::max_value().checked_add(2048), None);
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn checked_add(self, rhs: u32) -> Option<Self> {
        self.0.checked_add(rhs).map(Self)
    }

    /// Wrapping (modular) addition. Computes `self + rhs`, wrapping around the boundary
    /// of the type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::Derivation;
    /// assert_eq!(Derivation::new(100).wrapping_add(1), Derivation::new(101));
    /// assert_eq!(Derivation::max_value().wrapping_add(1), Derivation::new(0));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub const fn wrapping_add(self, rhs: u32) -> Self {
        Self(self.0.wrapping_add(rhs))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    fn saturating_sub(self, rhs: u32) -> Self {
        Self(self.0.saturating_sub(rhs))
    }
}

impl SoftDerivation {
    /// construct a soft derivation from the given derivation without
    /// checking the derivation is actually a soft derivation.
    ///
    /// this function does not perform any verification and if the value
    /// is not correct it will create a cascade of issues, be careful when
    /// utilizing this function.
    #[inline]
    pub const fn new_unchecked(derivation: Derivation) -> Self {
        Self(derivation)
    }

    /// build a soft derivation from the given `Derivation`. If the value
    /// is not a soft derivation it will return an error
    ///
    /// # Example
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation, DerivationError};
    /// # fn func() -> Result<(), DerivationError> {
    /// let derivation = Derivation::new(42);
    /// let derivation = SoftDerivation::new(derivation)?;
    ///
    /// println!("derivation: {}", derivation);
    /// # Ok(())
    /// # }
    /// #
    /// # func().unwrap();
    /// ```
    #[inline]
    pub fn new(derivation: Derivation) -> Result<Self, DerivationError> {
        if derivation.is_soft_derivation() {
            Ok(Self::new_unchecked(derivation))
        } else {
            Err(DerivationError::InvalidSoftDerivation(derivation))
        }
    }

    /// returns the max derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// let max = SoftDerivation::max_value();
    /// assert_eq!(max, SoftDerivation::new_unchecked(Derivation::new(0x7FFF_FFFF)));
    /// ```
    #[inline]
    pub const fn max_value() -> Self {
        Self::new_unchecked(Derivation::new(SOFT_DERIVATION_UPPER_BOUND - 1))
    }

    /// returns the min derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// let min = SoftDerivation::min_value();
    /// assert_eq!(min, SoftDerivation::new_unchecked(Derivation::new(0)));
    /// ```
    #[inline]
    pub const fn min_value() -> Self {
        Self::new_unchecked(Derivation::min_value())
    }

    /// calculate `self + rhs`
    ///
    /// Returns the tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// assert_eq!(
    ///     SoftDerivation::new_unchecked(Derivation::new(5)).overflowing_add(2),
    ///     (SoftDerivation::new_unchecked(Derivation::new(7)), false)
    /// );
    /// assert_eq!(
    ///     SoftDerivation::max_value().overflowing_add(1),
    ///     (SoftDerivation::new_unchecked(Derivation::new(0)), true)
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn overflowing_add(self, rhs: u32) -> (Self, bool) {
        let (v, b) = self.0.overflowing_add(rhs);

        if v.is_soft_derivation() {
            (Self::new_unchecked(v), b)
        } else {
            (
                Self::new_unchecked(Derivation::new(v.0 - SOFT_DERIVATION_UPPER_BOUND)),
                true,
            )
        }
    }

    /// saturating integer addition. Computes `self + rhs`, saturating
    /// at the numeric bounds instead of overflowing.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// assert_eq!(
    ///     SoftDerivation::new_unchecked(Derivation::new(100)).saturating_add(1),
    ///     SoftDerivation::new_unchecked(Derivation::new(101))
    /// );
    /// assert_eq!(
    ///     SoftDerivation::max_value().saturating_add(2048),
    ///     SoftDerivation::max_value(),
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn saturating_add(self, rhs: u32) -> Self {
        let d = self.0.saturating_add(rhs);

        // allow `unwrap_or`, it's 32bits of integer or a function pointer
        #[allow(clippy::or_fun_call)]
        Self::new(d).unwrap_or(Self::max_value())
    }

    /// checked integer addition. Computes `self + rhs`, returning `None` if overflow
    /// would occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// assert_eq!(
    ///     SoftDerivation::new_unchecked(Derivation::new(100)).checked_add(1),
    ///     Some(SoftDerivation::new_unchecked(Derivation::new(101)))
    /// );
    /// assert_eq!(
    ///     SoftDerivation::max_value().checked_add(2048),
    ///     None,
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn checked_add(self, rhs: u32) -> Option<Self> {
        let d = self.0.checked_add(rhs)?;

        Self::new(d).ok()
    }

    /// Wrapping (modular) addition. Computes `self + rhs`, wrapping around the boundary
    /// of the type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, SoftDerivation};
    /// assert_eq!(
    ///     SoftDerivation::new_unchecked(Derivation::new(100)).wrapping_add(1),
    ///     SoftDerivation::new_unchecked(Derivation::new(101))
    /// );
    /// assert_eq!(
    ///     SoftDerivation::max_value().wrapping_add(1),
    ///     SoftDerivation::new_unchecked(Derivation::new(0)),
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_add(self, rhs: u32) -> Self {
        let (d, _) = self.overflowing_add(rhs);
        d
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    fn saturating_sub(self, rhs: u32) -> Self {
        let d = self.0.saturating_sub(rhs);

        // allow `unwrap_or`, it's 32bits of integer or a function pointer
        #[allow(clippy::or_fun_call)]
        Self::new(d).unwrap_or(Self::min_value())
    }
}

impl HardDerivation {
    /// construct a hard derivation from the given derivation without
    /// checking the derivation is actually a hard derivation.
    ///
    /// this function does not perform any verification and if the value
    /// is not correct it will create a cascade of issues, be careful when
    /// utilizing this function.
    #[inline]
    pub const fn new_unchecked(derivation: Derivation) -> Self {
        Self(derivation)
    }

    /// build a hard derivation from the given `Derivation`. If the value
    /// is not a hard derivation it will return an error
    ///
    /// # Example
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation, DerivationError};
    /// # fn func() -> Result<(), DerivationError> {
    /// let derivation = Derivation::new(0x8000_0001);
    /// let derivation = HardDerivation::new(derivation)?;
    ///
    /// println!("derivation: {}", derivation);
    /// # Ok(())
    /// # }
    /// #
    /// # func().unwrap();
    /// ```
    #[inline]
    pub fn new(derivation: Derivation) -> Result<Self, DerivationError> {
        if derivation.is_hard_derivation() {
            Ok(Self::new_unchecked(derivation))
        } else {
            Err(DerivationError::InvalidHardDerivation(derivation))
        }
    }

    /// returns the max derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// let max = HardDerivation::max_value();
    /// assert_eq!(max, HardDerivation::new_unchecked(Derivation::new(0xFFFF_FFFF)));
    /// ```
    #[inline]
    pub const fn max_value() -> Self {
        Self::new_unchecked(Derivation::max_value())
    }

    /// returns the min derivation index value
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// let min = HardDerivation::min_value();
    /// assert_eq!(min, HardDerivation::new_unchecked(Derivation::new(0x8000_0000)));
    /// ```
    #[inline]
    pub const fn min_value() -> Self {
        Self::new_unchecked(Derivation::new(SOFT_DERIVATION_UPPER_BOUND))
    }

    /// calculate `self + rhs`
    ///
    /// Returns the tuple of the addition along with a boolean indicating whether
    /// an arithmetic overflow would occur. If an overflow would have occurred
    /// then the wrapped value is returned.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// assert_eq!(
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0005)).overflowing_add(2),
    ///     (HardDerivation::new_unchecked(Derivation::new(0x8000_0007)), false)
    /// );
    /// assert_eq!(
    ///     HardDerivation::max_value().overflowing_add(1),
    ///     (HardDerivation::new_unchecked(Derivation::new(0x8000_0000)), true)
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn overflowing_add(self, rhs: u32) -> (Self, bool) {
        let (v, b) = self.0.overflowing_add(rhs);

        if v.is_hard_derivation() {
            (Self::new_unchecked(v), b)
        } else {
            (
                Self::new_unchecked(Derivation::new(v.0 + SOFT_DERIVATION_UPPER_BOUND)),
                true,
            )
        }
    }

    /// saturating integer addition. Computes `self + rhs`, saturating
    /// at the numeric bounds instead of overflowing.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// assert_eq!(
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0100)).saturating_add(1),
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0101))
    /// );
    /// assert_eq!(
    ///     HardDerivation::max_value().saturating_add(2048),
    ///     HardDerivation::max_value(),
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn saturating_add(self, rhs: u32) -> Self {
        let d = self.0.saturating_add(rhs);

        // allow `unwrap_or`, it's 32bits of integer or a function pointer
        #[allow(clippy::or_fun_call)]
        Self::new(d).unwrap_or(Self::max_value())
    }

    /// checked integer addition. Computes `self + rhs`, returning `None` if overflow
    /// would occurred.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// assert_eq!(
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0100)).checked_add(1),
    ///     Some(HardDerivation::new_unchecked(Derivation::new(0x8000_0101)))
    /// );
    /// assert_eq!(
    ///     HardDerivation::max_value().checked_add(2048),
    ///     None,
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn checked_add(self, rhs: u32) -> Option<Self> {
        let d = self.0.checked_add(rhs)?;

        Self::new(d).ok()
    }

    /// Wrapping (modular) addition. Computes `self + rhs`, wrapping around the boundary
    /// of the type.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use chain_path_derivation::{Derivation, HardDerivation};
    /// assert_eq!(
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0100)).wrapping_add(1),
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0101))
    /// );
    /// assert_eq!(
    ///     HardDerivation::max_value().wrapping_add(1),
    ///     HardDerivation::new_unchecked(Derivation::new(0x8000_0000)),
    /// );
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_add(self, rhs: u32) -> Self {
        let (d, _) = self.overflowing_add(rhs);
        d
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    fn saturating_sub(self, rhs: u32) -> Self {
        let d = self.0.saturating_sub(rhs);

        // allow `unwrap_or`, it's 32bits of integer or a function pointer
        #[allow(clippy::or_fun_call)]
        Self::new(d).unwrap_or(Self::min_value())
    }
}

impl DerivationRange {
    /// create a derivation range from the given range
    pub fn new<R, T>(range: R) -> Self
    where
        R: std::ops::RangeBounds<T>,
        T: Into<Derivation> + Copy,
    {
        use std::ops::Bound;
        let start = match range.start_bound() {
            Bound::Unbounded => Derivation::min_value(),
            Bound::Included(b) => (*b).into(),
            Bound::Excluded(b) => (*b).into().saturating_add(1),
        };

        let end = match range.end_bound() {
            Bound::Unbounded => Derivation::max_value(),
            Bound::Included(b) => (*b).into().saturating_add(1),
            Bound::Excluded(b) => (*b).into(),
        };

        let range = std::ops::Range { start, end };

        Self { range }
    }
}

impl SoftDerivationRange {
    /// create a SoftDerivation range from the given range
    ///
    /// # panics
    ///
    /// this function will panic if the bounds are not valid SoftDerivation
    /// values.
    pub fn new<R, T>(range: R) -> Self
    where
        R: std::ops::RangeBounds<T>,
        T: TryInto<SoftDerivation> + Copy,
        <T as std::convert::TryInto<SoftDerivation>>::Error: std::error::Error,
    {
        use std::ops::Bound;
        let start = match range.start_bound() {
            Bound::Unbounded => Ok(SoftDerivation::min_value()),
            Bound::Included(b) => (*b).try_into(),
            Bound::Excluded(b) => (*b).try_into().map(|v| v.saturating_add(1)),
        };

        let end = match range.end_bound() {
            Bound::Unbounded => Ok(SoftDerivation::max_value()),
            Bound::Included(b) => (*b).try_into().map(|v| v.saturating_add(1)),
            Bound::Excluded(b) => (*b).try_into(),
        };

        let start =
            start.unwrap_or_else(|e| panic!("min bound is not a valid SoftDerivation, {:?}", e));
        let end =
            end.unwrap_or_else(|e| panic!("max bound is not a valid SoftDerivation, {:?}", e));

        let range = std::ops::Range { start, end };

        Self { range }
    }
}

impl HardDerivationRange {
    /// create a HardDerivation range from the given range
    ///
    /// # panics
    ///
    /// this function will panic if the bounds are not valid HardDerivation
    /// values.
    pub fn new<R, T>(range: R) -> Self
    where
        R: std::ops::RangeBounds<T>,
        T: TryInto<HardDerivation> + Copy,
        <T as std::convert::TryInto<HardDerivation>>::Error: std::error::Error,
    {
        use std::ops::Bound;
        let start = match range.start_bound() {
            Bound::Unbounded => Ok(HardDerivation::min_value()),
            Bound::Included(b) => (*b).try_into(),
            Bound::Excluded(b) => (*b).try_into().map(|v| v.saturating_add(1)),
        };

        let end = match range.end_bound() {
            Bound::Unbounded => Ok(HardDerivation::max_value()),
            Bound::Included(b) => (*b).try_into().map(|v| v.saturating_add(1)),
            Bound::Excluded(b) => (*b).try_into(),
        };

        let start =
            start.unwrap_or_else(|e| panic!("min bound is not a valid HardDerivation, {:?}", e));
        let end =
            end.unwrap_or_else(|e| panic!("max bound is not a valid HardDerivation, {:?}", e));

        let range = std::ops::Range { start, end };

        Self { range }
    }
}

/* Iterator **************************************************************** */

impl Iterator for DerivationRange {
    type Item = Derivation;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.range.start;
        if self.range.contains(&start) {
            self.range.start = start.saturating_add(1);
            Some(start)
        } else {
            None
        }
    }
}

impl Iterator for SoftDerivationRange {
    type Item = SoftDerivation;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.range.start;
        if self.range.contains(&start) {
            self.range.start = start.saturating_add(1);
            Some(start)
        } else {
            None
        }
    }
}

impl Iterator for HardDerivationRange {
    type Item = HardDerivation;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.range.start;
        if self.range.contains(&start) {
            self.range.start = start.saturating_add(1);
            Some(start)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for DerivationRange {
    fn len(&self) -> usize {
        let Derivation(start) = self.range.start;
        let Derivation(end) = self.range.end;

        (end - start) as usize
    }
}

impl ExactSizeIterator for SoftDerivationRange {
    fn len(&self) -> usize {
        let Derivation(start) = self.range.start.0;
        let Derivation(end) = self.range.end.0;

        (end - start) as usize
    }
}

impl ExactSizeIterator for HardDerivationRange {
    fn len(&self) -> usize {
        let Derivation(start) = self.range.start.0;
        let Derivation(end) = self.range.end.0;

        (end - start) as usize
    }
}

impl DoubleEndedIterator for DerivationRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back = self.range.end.saturating_sub(0);
        if self.range.contains(&next_back) {
            self.range.end = next_back;
            Some(next_back)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for SoftDerivationRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back = self.range.end.saturating_sub(0);
        if self.range.contains(&next_back) {
            self.range.end = next_back;
            Some(next_back)
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for HardDerivationRange {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back = self.range.end.saturating_sub(0);
        if self.range.contains(&next_back) {
            self.range.end = next_back;
            Some(next_back)
        } else {
            None
        }
    }
}

impl std::iter::FusedIterator for DerivationRange {}
impl std::iter::FusedIterator for SoftDerivationRange {}
impl std::iter::FusedIterator for HardDerivationRange {}

/* Default ***************************************************************** */

impl Default for SoftDerivation {
    fn default() -> Self {
        Self::min_value()
    }
}

impl Default for HardDerivation {
    fn default() -> Self {
        Self::min_value()
    }
}

/* Display ***************************************************************** */

impl Display for Derivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_soft_derivation() {
            self.0.fmt(f)
        } else {
            write!(f, "'{}", self.0 - SOFT_DERIVATION_UPPER_BOUND)
        }
    }
}

impl Display for SoftDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for HardDerivation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/* FromStr ***************************************************************** */

#[derive(Error, Debug)]
pub enum ParseDerivationError {
    #[error("Not a valid derivation value")]
    NaN(
        #[source]
        #[from]
        std::num::ParseIntError,
    ),

    #[error("Not a valid derivation")]
    InvalidDerivation(
        #[source]
        #[from]
        DerivationError,
    ),
}

impl str::FromStr for Derivation {
    type Err = ParseDerivationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('\'') {
            s.parse::<u32>()
                .map(|v| v + SOFT_DERIVATION_UPPER_BOUND)
                .map(Derivation)
                .map_err(ParseDerivationError::NaN)
        } else {
            s.parse().map(Derivation).map_err(ParseDerivationError::NaN)
        }
    }
}

impl str::FromStr for SoftDerivation {
    type Err = ParseDerivationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let derivation = s.parse()?;
        Ok(Self::new(derivation)?)
    }
}

impl str::FromStr for HardDerivation {
    type Err = ParseDerivationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let derivation = s.parse()?;
        Ok(Self::new(derivation)?)
    }
}

/* Conversion ************************************************************** */

impl From<u32> for Derivation {
    fn from(v: u32) -> Derivation {
        Derivation(v)
    }
}

impl From<Derivation> for u32 {
    fn from(d: Derivation) -> Self {
        d.0
    }
}

impl From<SoftDerivation> for Derivation {
    fn from(d: SoftDerivation) -> Self {
        d.0
    }
}

impl From<HardDerivation> for Derivation {
    fn from(d: HardDerivation) -> Self {
        d.0
    }
}

impl TryFrom<Derivation> for SoftDerivation {
    type Error = DerivationError;

    fn try_from(value: Derivation) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<Derivation> for HardDerivation {
    type Error = DerivationError;

    fn try_from(value: Derivation) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<u32> for SoftDerivation {
    type Error = DerivationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(Derivation::new(value))
    }
}

impl TryFrom<u32> for HardDerivation {
    type Error = DerivationError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(Derivation::new(value))
    }
}

/* Deref ******************************************************************* */

impl Deref for Derivation {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SoftDerivation {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Deref for HardDerivation {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for Derivation {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            Derivation(u32::arbitrary(g))
        }
    }

    impl Arbitrary for SoftDerivation {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let derivation = Derivation(u32::arbitrary(g) % SOFT_DERIVATION_UPPER_BOUND);
            Self::new(derivation).expect("Generated an invalid value for soft derivation")
        }
    }

    impl Arbitrary for HardDerivation {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let derivation = Derivation(
                u32::arbitrary(g) % SOFT_DERIVATION_UPPER_BOUND + SOFT_DERIVATION_UPPER_BOUND,
            );
            Self::new(derivation).expect("Generated an invalid value for hard derivation")
        }
    }

    #[test]
    fn derivation_iterator_1() {
        let range = DerivationRange::new(..8);
        let expected = vec![
            Derivation::new(0),
            Derivation::new(1),
            Derivation::new(2),
            Derivation::new(3),
            Derivation::new(4),
            Derivation::new(5),
            Derivation::new(6),
            Derivation::new(7),
        ];

        for (address, expected) in range.zip(expected) {
            assert_eq!(address, expected);
        }
    }

    #[test]
    fn derivation_iterator_2() {
        let range = DerivationRange::new(4..8);
        let expected = vec![
            Derivation::new(4),
            Derivation::new(5),
            Derivation::new(6),
            Derivation::new(7),
        ];

        for (address, expected) in range.zip(expected) {
            assert_eq!(address, expected);
        }
    }

    #[test]
    fn derivation_iterator_3() {
        let range = DerivationRange::new(4..=8);
        let expected = vec![
            Derivation::new(4),
            Derivation::new(5),
            Derivation::new(6),
            Derivation::new(7),
            Derivation::new(8),
        ];

        for (address, expected) in range.zip(expected) {
            assert_eq!(address, expected);
        }
    }

    #[test]
    fn derivation_iterator_4() {
        let range = DerivationRange::new::<_, u32>(..);

        assert_eq!(range.len(), u32::max_value() as usize);
    }

    #[test]
    fn to_string() {
        assert_eq!(Derivation(0).to_string(), "0");
        assert_eq!(Derivation(9289).to_string(), "9289");
        assert_eq!(Derivation(SOFT_DERIVATION_UPPER_BOUND).to_string(), "'0");
        assert_eq!(
            Derivation(SOFT_DERIVATION_UPPER_BOUND + 9289).to_string(),
            "'9289"
        );
    }

    #[quickcheck]
    fn fmt_parse_derivation(derivation: Derivation) -> bool {
        let s = derivation.to_string();
        let v = s.parse::<Derivation>().unwrap();

        v == derivation
    }

    #[quickcheck]
    fn fmt_parse_soft_derivation(derivation: SoftDerivation) -> bool {
        let s = derivation.to_string();
        let v = s.parse::<SoftDerivation>().unwrap();

        v == derivation
    }

    #[quickcheck]
    fn fmt_parse_hard_derivation(derivation: HardDerivation) -> bool {
        let s = derivation.to_string();
        let v = s.parse::<HardDerivation>().unwrap();

        v == derivation
    }
}
