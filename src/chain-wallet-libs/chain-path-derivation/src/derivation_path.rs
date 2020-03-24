use crate::Derivation;
use std::fmt::{self, Display};

pub struct AnyScheme;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl<S> Clone for DerivationPath<S> {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S> DerivationPath<S> {
    pub fn iter(&self) -> std::slice::Iter<'_, Derivation> {
        self.path.iter()
    }

    pub fn append(&self, derivation: Derivation) -> Self {
        let mut cloned = self.clone();
        cloned.path.push(derivation);
        cloned
    }

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

    pub(crate) fn get(&self, index: usize) -> Option<Derivation> {
        self.path.get(index).copied()
    }

    pub(crate) fn coerce_unchecked<T>(self) -> DerivationPath<T> {
        DerivationPath {
            path: self.path,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<S, R, T> Iterator for DerivationPathRange<S, R, T>
where
    R: Iterator<Item = T>,
    T: Into<Derivation>,
{
    type Item = DerivationPath<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.range.next()?.into();
        let path = self.root.append(next);
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
        let path = self.root.append(next);
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

impl<S> Display for DerivationPath<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")?;
        for derivation in self.iter() {
            write!(f, "/{}", derivation)?;
        }
        Ok(())
    }
}
