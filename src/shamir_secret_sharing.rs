use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use ark_ff::fields::{Fp64, MontBackend, MontConfig};
use ark_poly::Polynomial;
use ark_std::test_rng;
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
pub fn get_shares_secret(secret: Fq, n: u64, t:usize) -> Vec<Fq> {
  // 1. Generate polynomial of deg t
  let p = create_pol(secret, t);

  // 2. Evaluate polynomial at n points (1,..,n)
  let mut evals = Vec::new();
  for i in 0u64..n.into() {
    let pi = p.evaluate(&Fq::from(i));
    evals.push(pi);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shares_secret() {
        let secret = Fq::from(5u64); // Example secret
        let n = 10; // Number of shares
        let t = 3; // Deg pol

        let shares = get_shares_secret(secret, n, t);

        assert_eq!(shares.len(), n as usize);

        println!("Shares are {:#?}", shares);

    }
}
