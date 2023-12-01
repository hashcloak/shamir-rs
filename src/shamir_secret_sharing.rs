use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_ff::Field;
use ark_poly::Polynomial;
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial, EvaluationDomain, Evaluations};
use rand::Rng;

const MODULUS: u64 = 127;

#[derive(MontConfig)]
#[modulus = "127"]
#[generator = "3"]
pub struct FqConfig;
pub type Fq = Fp64<MontBackend<FqConfig, 1>>;

pub fn generate_secret() -> Fq {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
    let r: u64 = rng.gen();
    let secret = r % MODULUS;

    Fq::from(secret)
}

// Obtain n shares for a secret in Zq
// The shares are obtained by creating a polynomial with degree t, where t < n/2
// returns n shares
pub fn get_shares_secret(secret: Fq, inputs: Vec<u64>, t: usize) -> Vec<(Fq, Fq)> {
    // 1. Generate polynomial of deg t
    let p = create_pol(secret, t);

    // 2. Evaluate polynomial at n points (1,..,n)
    let mut evals = Vec::new();
    for i in inputs {
        let x = Fq::from(i);
        let pi = p.evaluate(&x);
        evals.push((x, pi));
    }

    evals
}

// Returns a polynomial s + a_1x + a_2x^2 + .. + a_tx^t
fn create_pol(s: Fq, t: usize) -> DensePolynomial<Fq> {
    let mut rng = ark_std::test_rng();
    // Generate a random polynomial of degree t
    let mut poly = DensePolynomial::rand(t, &mut rng);

    // Replace the constant term by the secret
    poly.coeffs[0] = s;

    poly
}

// Interpolate polynomial f and return f(0)
pub fn interpolate(coeff: Vec<(Fq, Fq)>) -> Fq {
    let xs: Vec<Fq> = coeff.iter().map(|(x, _)| *x).collect();
    let ys: Vec<Fq> = coeff.iter().map(|(_, y)| *y).collect();

    let mut result = Fq::from(0u64);

    for i in 0..coeff.len() {
        let mut term = ys[i];
        for j in 0..coeff.len() {
            if i != j {
                assert!(xs[i] != xs[j]);
                let denominator: Fq = xs[i] - xs[j];
                let inv: Fq = denominator.inverse().unwrap();
                let numerator: Fq = -xs[j];
                term = term * (numerator * inv);
            }
        }
        result = result + term;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_recovery() {
        // Generate a random secret
        let secret = generate_secret();

        // Define the number of shares and threshold
        let n = 5; // Number of shares
        let t = 2; // Threshold (degree of polynomial)

        // Generate inputs (1..=n)
        let inputs: Vec<u64> = (1..=n).collect();

        // Obtain shares
        let shares = get_shares_secret(secret, inputs, t);

        // Recover the polynomial using interpolation
        let recovered_val = interpolate(shares);

        // Check if the constant term of the recovered polynomial is the secret
        assert_eq!(
            recovered_val, secret,
            "The recovered secret does not match the original secret"
        );
    }
}
