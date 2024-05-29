use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::Field;
use ark_ff::{One, PrimeField, Zero};
use ark_groth16::Groth16;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::bits::boolean::AllocatedBool;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::groups::curves::twisted_edwards::AffineVar;
use ark_r1cs_std::prelude::*;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
#[derive(Clone)]
pub struct DiscreteLogCircuit {
    pub x: Option<Fr>, // The witness (exponent)
    pub g: Fr,         // The base (generator)
    pub h: Fr,         // The result (public input)
}

impl ConstraintSynthesizer<Fr> for DiscreteLogCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let x = FpVar::new_witness(ark_relations::ns!(cs, "x"), || {
            self.x.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let g = FpVar::new_input(ark_relations::ns!(cs, "g"), || Ok(self.g))?;
        let h = FpVar::new_input(ark_relations::ns!(cs, "h"), || Ok(self.h))?;

        // Compute g^x
        let mut result = FpVar::one();
        let mut base = FpVar::new_constant(ark_relations::ns!(cs, "base"), self.g).unwrap();
        let mut exponent = FpVar::new_witness(ark_relations::ns!(cs, "exponent"), || {
            self.x.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Perform the exponentiation g^x = h and enforce the constraint
        for _ in 0..256 {
            let new_result = base.mul(&result)?;
            result = new_result;
            let (exponent_bit, _) = exponent.to_bits_le()?;
            let new_base = base.mul(base)?;
            base = new_base;
            exponent = exponent.sub(&exponent_bit.lc());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ark_std::test_rng;
    use rand::RngCore;
    #[test]
    fn test_discrete_log_circuit() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());
        let g = Fr::from(2u32); // Base (generator)
        let x = Fr::from(5u32); // Exponent
        let g_x = 5_i32.pow(2);
        let h = Fr::from(g_x); // Result (public input)
        let circuit = DiscreteLogCircuit { x: Some(x), g, h };

        // Generate proving and verifying keys
        let (pk, vk) =
            Groth16::<Bls12_381>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();

        // Create a proof
        let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap();

        // Verify the proof
        let is_valid = Groth16::<Bls12_381>::verify(&vk, &[g, h], &proof).unwrap();
        assert!(is_valid);

        // Print nicely formatted information
        fn tracesub<T: std::fmt::Debug>(name: &str, value: T) {
            println!("{}: {:?}", name, value);
        }

        // Example usage of tracesub
        tracesub("Base (g)", g);
        tracesub("Exponent (x)", x);
        tracesub("Result (h)", h);
    }
}
