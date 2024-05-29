use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::Field;
use ark_groth16::Groth16;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::R1CSVar;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::CircuitSpecificSetupSNARK;
use ark_snark::SNARK;
use ark_std::rand::RngCore;
use ark_std::rand::{Rng, SeedableRng};
use ark_std::UniformRand;

// Define a helper function to check primality
fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    for i in 2..=((n as f64).sqrt() as u32) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// Define the semi-prime circuit
#[derive(Clone)]
pub struct SemiPrimeCircuit {
    pub p: Option<Fr>, // The first prime factor
    pub q: Option<Fr>, // The second prime factor
    pub n: Option<Fr>, // The public input (product to verify)
}

impl ConstraintSynthesizer<Fr> for SemiPrimeCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let p = FpVar::new_witness(ark_relations::ns!(cs, "p"), || {
            self.p.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let q = FpVar::new_witness(ark_relations::ns!(cs, "q"), || {
            self.q.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let n = FpVar::new_input(ark_relations::ns!(cs, "n"), || {
            self.n.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Constraint to enforce n = p * q
        let pq = &p * &q;
        pq.enforce_equal(&n)?;

        // Constraints to enforce p and q are prime (naive method, for demonstration purposes)
        for i in 2..=(10 as u32) {
            // Adjust this range as necessary
            if is_prime(i) {
                let i_fr = Fr::from(i);
                let i_var = FpVar::Constant(i_fr);

                // Ensure p is not divisible by any prime in the range
                let p_mod_i = modolo(&p, &i_var);
                p_mod_i.enforce_not_equal(&FpVar::zero())?;

                // Ensure q is not divisible by any prime in the range
                let q_mod_i = &q % &i_var;
                q_mod_i.enforce_not_equal(&FpVar::zero())?;
            }
        }

        Ok(())
    }
}

fn modolo(p: &FpVar<Fr>, i: &FpVar<Fr>) -> FpVar<Fr> {
    let p_val = p.value().unwrap();
    let i_val = i.value().unwrap();
    let result;

    while p_val > i_val {
        p_vao;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::test_rng;

    #[test]
    fn test_semi_prime_circuit() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());

        let circuit = SemiPrimeCircuit {
            p: None,
            q: None,
            n: None,
        };

        // Generate a pk vk
        let (pk, vk) =
            Groth16::<Bls12_381>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();

        // Example prime numbers and their product
        let p = Fr::from(3u32); // First prime factor
        let q = Fr::from(7u32); // Second prime factor
        let n = p * q; // Product

        // Create an instance of the semi-prime circuit
        let circuit = SemiPrimeCircuit {
            p: Some(p),
            q: Some(q),
            n: Some(n),
        };

        // Create a proof of the semi-prime circuit
        let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap();

        // Verify the proof
        let is_valid = Groth16::<Bls12_381>::verify(&vk, &vec![n], &proof).unwrap();
        assert!(is_valid);

        // Add a tracesub function to print nicely formatted information
        fn tracesub<T: std::fmt::Debug>(name: &str, value: T) {
            println!("{}: {:?}", name, value);
        }

        // Example usage of tracesub
        tracesub("First prime factor (p)", p);
        tracesub("Second prime factor (q)", q);
        tracesub("Product (n)", n);
    }
}
