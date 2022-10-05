use crate::Derivation;
use std::{
    fmt::{self, Display},
    hash::{Hash, Hasher},
    ops::Deref,
    str::{self, FromStr},
};
use thiserror::Error;

/// any derivation path scheme
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Default)]
pub struct AnyScheme;

/// a derivation path with a tagged scheme (for example [`Bip44`]).
///
/// This allows following the specific set of rules for a derivation
/// path, preventing some errors. For example, the Bip44 scheme enforce
/// the 3 first derivation to be hard derivation indices and the 2 last
/// ones to be soft derivation indices.
///
/// [`Bip44`]: ./struct.Bip44.html
#[derive(Debug)]
pub struct DerivationPath<S> {
    path: Vec<Derivation>,
    _marker: std::marker::PhantomData<S>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T>,
    T: Into<Derivation>,
{
    root: DerivationPath<S>,
    range: R,
}

impl<S> DerivationPath<S> {
    /// Iterate through every derivation indices of the given derivation path
    ///
    pub fn iter(&self) -> std::slice::Iter<'_, Derivation> {
        self.path.iter()
    }

    /// create a new derivation path with appending the new derivation
    /// index to the current derivation path
    ///
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn append_unchecked(&self, derivation: Derivation) -> Self {
        let mut cloned = self.clone();
        cloned.path.push(derivation);
        cloned
    }

    /// create a range of derivation path with the given derivation range
    ///
    /// this will create an iterator of DerivationPath with the first derivation
    /// indices being the one fo the given derivation path and the append indices
    /// the one from the range.
    ///
    /// i.e. if the derivation path is `m/'1/42` and the range is `..20` it will
    /// create an iterator of derivation path for all: `m/'1/42/0` `m/'1/42/1`
    /// `m/'1/42/2` ...  `m/'1/42/19`.
    pub fn sub_range<R, T>(&self, range: R) -> DerivationPathRange<S, R, T>
    where
        R: Iterator<Item = T>,
        T: Into<Derivation>,
    {
        let root = self.clone();

        DerivationPathRange { root, range }
    }

    pub(crate) fn new_empty() -> Self {
        Self {
            path: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub(crate) fn push(&mut self, derivation: Derivation) {
        self.path.push(derivation)
    }

    #[inline]
    pub(crate) fn get_unchecked(&self, index: usize) -> Derivation {
        if let Some(v) = self.get(index).copied() {
            v
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    pub fn coerce_unchecked<T>(self) -> DerivationPath<T> {
        DerivationPath {
            path: self.path,
            _marker: std::marker::PhantomData,
        }
    }
}

impl DerivationPath<AnyScheme> {
    pub fn new() -> Self {
        DerivationPath::new_empty()
    }
}

impl<S> Deref for DerivationPath<S> {
    type Target = [Derivation];
    fn deref(&self) -> &Self::Target {
        self.path.deref()
    }
}

/* Default ***************************************************************** */

impl Default for DerivationPath<AnyScheme> {
    fn default() -> Self {
        Self::new()
    }
}

/* Comparison ************************************************************** */

impl<T1, T2> PartialEq<DerivationPath<T1>> for DerivationPath<T2> {
    fn eq(&self, other: &DerivationPath<T1>) -> bool {
        self.path.eq(&other.path)
    }
}

impl<T> Eq for DerivationPath<T> {}

impl<T1, T2> PartialOrd<DerivationPath<T1>> for DerivationPath<T2> {
    fn partial_cmp(&self, other: &DerivationPath<T1>) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl<T> Ord for DerivationPath<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

/* Hasher ****************************************************************** */

impl<T> Hash for DerivationPath<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state);
        self._marker.hash(state);
    }
}

/* Iterator **************************************************************** */

impl<S, R, T> Iterator for DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T>,
    T: Into<Derivation>,
{
    type Item = DerivationPath<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?.into();
        let path = self.root.append_unchecked(next);
        Some(path)
    }
}

impl<S, R, T> ExactSizeIterator for DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T> + ExactSizeIterator,
    T: Into<Derivation>,
{
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<S, R, T> DoubleEndedIterator for DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T> + DoubleEndedIterator,
    T: Into<Derivation>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.range.next_back()?.into();
        let path = self.root.append_unchecked(next);
        Some(path)
    }
}

impl<S, R, T> std::iter::FusedIterator for DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T> + std::iter::FusedIterator,
    T: Into<Derivation>,
{
}

impl<S> IntoIterator for DerivationPath<S> {
    type Item = Derivation;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.path.into_iter()
    }
}

impl<'a, S> IntoIterator for &'a DerivationPath<S> {
    type Item = &'a Derivation;
    type IntoIter = std::slice::Iter<'a, Derivation>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/* FromIterator ************************************************************ */

impl std::iter::FromIterator<Derivation> for DerivationPath<AnyScheme> {
    fn from_iter<T: IntoIterator<Item = Derivation>>(iter: T) -> Self {
        let mut dp = Self::new_empty();
        dp.path = iter.into_iter().collect();
        dp
    }
}

/* Display ***************************************************************** */

impl<S> Display for DerivationPath<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")?;
        for derivation in self.iter() {
            write!(f, "/{}", derivation)?;
        }
        Ok(())
    }
}

/* FromStr ***************************************************************** */

#[derive(Debug, Error)]
pub enum ParseDerivationPathError {
    #[error("Derivation Path should start with 'm'")]
    NotValidRoot,

    #[error("Invalid derivation at index '{index}'")]
    NotValidDerivation {
        index: usize,
        #[source]
        source: crate::ParseDerivationError,
    },

    #[error("Invalid number of derivation ({actual}), expected {expected}")]
    InvalidNumberOfDerivations { actual: usize, expected: usize },
}

impl FromStr for DerivationPath<AnyScheme> {
    type Err = ParseDerivationPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut derivations = s.split('/');

        let m = derivations
            .next()
            .ok_or(ParseDerivationPathError::NotValidRoot)?;
        if m != "m" {
            return Err(ParseDerivationPathError::NotValidRoot);
        }

        let mut path = Self::new_empty();
        for (index, derivation) in derivations.enumerate() {
            let derivation = derivation
                .parse()
                .map_err(|source| ParseDerivationPathError::NotValidDerivation { index, source })?;
            path.push(derivation);
        }

        Ok(path)
    }
}

/* Clone ******************************************************************* */

impl<S> Clone for DerivationPath<S> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    const MAX_DERIVATION_PATH_ANY_SCHEME_LENGTH: usize = 24;

    impl Arbitrary for DerivationPath<AnyScheme> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let path_len = usize::arbitrary(g) % MAX_DERIVATION_PATH_ANY_SCHEME_LENGTH;

            std::iter::repeat_with(|| Derivation::arbitrary(g))
                .take(path_len)
                .collect()
        }
    }

    #[test]
    fn to_string() {
        let mut path = DerivationPath::<AnyScheme>::new_empty();
        assert_eq!(path.to_string(), "m");
        path.push(Derivation::new(0x0000_0000));
        assert_eq!(path.to_string(), "m/0");
        path.push(Derivation::new(0x0000_0007));
        assert_eq!(path.to_string(), "m/0/7");
        path.push(Derivation::new(0x8000_0001));
        assert_eq!(path.to_string(), "m/0/7/'1");
        path.push(Derivation::new(0x8000_000a));
        assert_eq!(path.to_string(), "m/0/7/'1/'10");
    }

    #[test]
    fn invalid_parse() {
        assert!("".parse::<DerivationPath<AnyScheme>>().is_err());
        assert!("a".parse::<DerivationPath<AnyScheme>>().is_err());
        assert!("M".parse::<DerivationPath<AnyScheme>>().is_err());
        assert!("m/a".parse::<DerivationPath<AnyScheme>>().is_err());
        assert!("m/\"1".parse::<DerivationPath<AnyScheme>>().is_err());
    }

    #[quickcheck]
    fn fmt_parse(derivation_path: DerivationPath<AnyScheme>) -> bool {
        let s = derivation_path.to_string();
        let v = s.parse::<DerivationPath<AnyScheme>>().unwrap();

        v == derivation_path
    }
}
