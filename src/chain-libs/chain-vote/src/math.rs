#![allow(dead_code)]

/// Math module define polynomial types and operations that is used to setup the scheme.
use crate::gang::Scalar;
use rand_core::{CryptoRng, RngCore};

/// A polynomial of specific degree d
///
/// of the form: A * x^d + B * x^(d-1) + ... + Z * x^0
#[derive(Clone)]
pub struct Polynomial {
    pub elements: Vec<Scalar>,
}

impl std::fmt::Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (d, coef) in self.elements.iter().enumerate().rev() {
            match d {
                0 => write!(f, "{:?}", coef)?,
                1 => write!(f, "{:?} x +", coef)?,
                _ => write!(f, "{:?} x^{} +", coef, d)?,
            }
        }
        Ok(())
    }
}

impl Polynomial {
    /// Generate a new 0 polynomial of specific degree
    pub fn new(degree: usize) -> Self {
        Self {
            elements: vec![Scalar::zero(); degree + 1],
        }
    }

    pub fn set2(mut self, x0: Scalar, x1: Scalar) -> Self {
        assert!(self.degree() >= 1);
        self.elements[0] = x0;
        self.elements[1] = x1;
        self
    }

    /// Return the degree of the polynomial
    pub fn degree(&self) -> usize {
        assert!(!self.elements.is_empty());
        self.elements.len() - 1
    }

    /// Initialize from a vector, where each element represent the term coefficient
    /// starting from the lowest degree
    pub fn from_vec(elements: Vec<Scalar>) -> Self {
        assert_ne!(elements.len(), 0);
        Polynomial { elements }
    }

    /// generate a new polynomial of specific degree
    pub fn random<R: RngCore + CryptoRng>(rng: &mut R, degree: usize) -> Polynomial {
        let mut vec = Vec::with_capacity(degree + 1);

        for _ in 0..(degree + 1) {
            let r = Scalar::random(rng);
            vec.push(r);
        }
        Polynomial { elements: vec }
    }

    /// get the value of a polynomial a0 + a1 * x^1 + a2 * x^2 + .. + an * x^n for a value x=at
    pub fn evaluate(&self, at: &Scalar) -> Scalar {
        Scalar::sum(self.elements.iter().zip(at.exp_iter()).map(|(e, x)| e * x))
            .expect("empty polynomial")
    }

    /// Evaluate the polynomial at x=0
    pub fn at_zero(&self) -> Scalar {
        self.elements[0].clone()
    }

    pub fn get_coefficient_at(&self, degree: usize) -> &Scalar {
        &self.elements[degree]
    }

    pub fn get_coefficients(&self) -> std::slice::Iter<Scalar> {
        self.elements.iter()
    }
}

impl std::ops::Add<Polynomial> for Polynomial {
    type Output = Polynomial;

    fn add(self, rhs: Polynomial) -> Self::Output {
        if self.degree() >= rhs.degree() {
            let mut x = self.elements;
            for (e, r) in x.iter_mut().zip(rhs.elements.iter()) {
                *e = &*e + r;
            }
            Self { elements: x }
        } else {
            let mut x = rhs.elements;
            for (e, r) in x.iter_mut().zip(self.elements.iter()) {
                *e = &*e + r;
            }
            Self { elements: x }
        }
    }
}

impl std::ops::Mul<Polynomial> for Polynomial {
    type Output = Polynomial;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Polynomial) -> Self::Output {
        //println!("muling {} * {}", self, rhs);
        let mut acc = vec![Scalar::zero(); self.degree() + rhs.degree() + 1];
        for (left_degree, left_coeff) in self.elements.iter().enumerate() {
            for (right_degree, right_coeff) in rhs.elements.iter().enumerate() {
                let degree = left_degree + right_degree;
                acc[degree] = &acc[degree] + &(left_coeff * right_coeff);
            }
        }
        Polynomial { elements: acc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn poly_tests() {
        let poly_deg_4 = Polynomial::new(4).set2(Scalar::one(), Scalar::from_u64(3));

        assert_eq!(poly_deg_4.degree(), 4);
        assert_eq!(
            poly_deg_4.evaluate(&Scalar::from_u64(3)),
            Scalar::from_u64(10)
        );
        assert_eq!(poly_deg_4.at_zero(), Scalar::one());

        let poly_deg_2 = Polynomial::new(2).set2(Scalar::from_u64(13), Scalar::from_u64(2));
        let added_polys = poly_deg_4.clone() + poly_deg_2.clone();

        let expected_poly = Polynomial::from_vec(vec![Scalar::from_u64(14), Scalar::from_u64(5)]);

        for (a, b) in added_polys
            .get_coefficients()
            .zip(expected_poly.get_coefficients())
        {
            assert_eq!(a, b);
        }

        let mult_poly = poly_deg_4 * poly_deg_2;

        let expected_mult = Polynomial::from_vec(vec![
            Scalar::from_u64(13),
            Scalar::from_u64(41),
            Scalar::from_u64(6),
        ]);

        for (a, b) in mult_poly
            .get_coefficients()
            .zip(expected_mult.get_coefficients())
        {
            assert_eq!(a, b);
        }
    }
}
