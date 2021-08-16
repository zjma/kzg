use pairing::{
    Engine,
    group::{
        Curve,
        Group,
        ff::Field,
        prime::PrimeCurveAffine
    }
};
use thiserror::Error;
use core::fmt::Debug;

pub mod polynomial;

use polynomial::Polynomial;

/// parameters from tested setup
pub struct KZGParams<E: Engine, const MAX_DEGREE: usize> {
    /// generator of g
    g: E::G1Affine,
    /// generator of G2
    h: E::G2Affine,
    /// g^alpha^1, g^alpha^2, ...
    gs: [E::G1Affine; MAX_DEGREE],
    /// g^alpha^1, g^alpha^2, ...
    hs: [E::G2Affine; MAX_DEGREE]
}

// the commitment - "C" in the paper. It's a single group element
pub struct KZGCommitment<E: Engine>(E::G1Affine);

// A witness for a single element - "w_i" in the paper. It's a group element.
pub struct KZGWitness<E: Engine>(E::G1Affine);

#[derive(Error, Debug)]
pub enum KZGError {
    #[error("no polynomial!")]
    NoPolynomial,
    #[error("point not on polynomial!")]
    PointNotOnPolynomial
}


pub struct KZGProver<E: Engine, const MAX_DEGREE: usize> {
    parameters: KZGParams<E, MAX_DEGREE>,
    polynomial: Option<Polynomial<E, MAX_DEGREE>>,
    commitment: Option<KZGCommitment<E>>,
    batch_witness: Option<E::G1>,
    witnesses: [Option<E::G1Affine>; MAX_DEGREE]
}

pub struct KZGVerifier<E: Engine, const MAX_DEGREE: usize> {
    parameters: KZGParams<E, MAX_DEGREE>,
}

impl<E: Engine, const MAX_DEGREE: usize> KZGProver<E, MAX_DEGREE> {
    /// initializes `polynomial` to zero polynomial
    fn new(parameters: KZGParams<E, MAX_DEGREE>) -> Self {
        Self {
            parameters,
            polynomial: None,
            commitment: None,
            batch_witness: None,
            witnesses: [None; MAX_DEGREE]
        }
    }

    fn commit(&mut self, polynomial: Polynomial<E, MAX_DEGREE>) -> KZGCommitment<E>{
        let mut commitment = E::G1::identity();
        for (i, &coeff) in polynomial.coeffs.iter().enumerate() {
            if i == 0 {
                commitment += self.parameters.g * coeff;
            } else {
                commitment += self.parameters.gs[i-1] * coeff;
            }
        }

        self.polynomial = Some(polynomial);
        KZGCommitment(commitment.to_affine())
    }

    fn open(&self) -> Result<Polynomial<E, MAX_DEGREE>, KZGError> {
        self.polynomial.clone().ok_or(KZGError::NoPolynomial)
    }

    fn create_witness(&mut self, (x, y): (E::Fr, E::Fr)) -> Result<KZGWitness<E>, KZGError> {
        match self.polynomial {
            None => Err(KZGError::NoPolynomial),
            Some(ref polynomial) => {

                let mut dividend = polynomial.clone();
                dividend.coeffs[0] -= y;

                let mut divisor = Polynomial::new_from_coeffs([E::Fr::zero(); MAX_DEGREE], 1);
                divisor.coeffs[0] = -x;
                divisor.coeffs[1] = E::Fr::one();
                match dividend.long_division(&divisor) {
                    // by polynomial remainder theorem, if (x - point.x) does not divide self.polynomial, then
                    // self.polynomial(point.y) != point.1
                    (_, Some(_)) => Err(KZGError::PointNotOnPolynomial),
                    (psi, None) => {
                        let mut witness = E::G1::identity();
                        for (i, &coeff) in psi.coeffs.iter().enumerate() {
                            if i == 0 {
                                witness += self.parameters.g * coeff;
                            } else {
                                witness += self.parameters.gs[i-1] * coeff;
                            }
                        }

                        Ok(KZGWitness(witness.to_affine()))
                    }
                }
            }
        }
    }
}

impl<E: Engine, const MAX_DEGREE: usize> KZGVerifier<E, MAX_DEGREE> {
    fn verify_poly(&self, commitment: KZGCommitment<E>, polynomial: &Polynomial<E, MAX_DEGREE>) -> bool {
        let mut check = E::G1::identity();
        for (i, &coeff) in polynomial.coeffs.iter().enumerate() {
            if i == 0 {
                check += self.parameters.g * coeff;
            } else {
                check += self.parameters.gs[i-1] * coeff;
            }
        }

        check.to_affine() == commitment.0
    }

    fn verify_eval(&self, (x, y): (E::Fr, E::Fr), commitment: KZGCommitment<E>, witness: KZGWitness<E>) -> bool {
        let lhs = E::pairing(
            &witness.0,
            &(self.parameters.hs[0].to_curve() + self.parameters.h * -x).to_affine()
        );
        let rhs = E::pairing(
            &(commitment.0.to_curve() - self.parameters.g * -y).to_affine(),
            &self.parameters.h
        );

        lhs == rhs
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
