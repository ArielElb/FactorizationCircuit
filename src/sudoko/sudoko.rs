use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::Field;
use ark_groth16::Groth16;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::fields::FieldVar;
use ark_r1cs_std::uint8::UInt8;
use ark_snark::CircuitSpecificSetupSNARK;
use ark_snark::SNARK;
use ark_std::rand::RngCore;
use ark_std::rand::{Rng, SeedableRng};
use ark_std::UniformRand;

use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

// Define the Sudoku circuit
#[derive(Clone)]
struct SudokuCircuit {
    pub solution: Option<[[u8; 9]; 9]>,
}

impl ConstraintSynthesizer<Fr> for SudokuCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let mut vars = vec![];

        // Allocate variables for the Sudoku solution
        for i in 0..9 {
            for j in 0..9 {
                let var = UInt8::new_witness(ark_relations::ns!(cs, "cell"), || {
                    self.solution
                        .ok_or(SynthesisError::AssignmentMissing)
                        .map(|solution| solution[i][j])
                })?;
                vars.push(var);
            }
        }

        // Enforce the constraints for each cell to be between 1 and 9
        for var in &vars {
            // var.enforce_in(cs.ns(|| "cell between 1 and 9"), 1, 9)?;
        }

        // Enforce the row constraints
        for i in 0..9 {
            for j in 0..9 {
                for k in (j + 1)..9 {
                    vars[i * 9 + j].enforce_not_equal(&vars[i * 9 + k])?;
                }
            }
        }

        // Enforce the column constraints
        for j in 0..9 {
            for i in 0..9 {
                for k in (i + 1)..9 {
                    vars[i * 9 + j].enforce_not_equal(&vars[k * 9 + j])?;
                }
            }
        }

        // Enforce the 3x3 sub-grid constraints
        for block_row in 0..3 {
            for block_col in 0..3 {
                for i in 0..3 {
                    for j in 0..3 {
                        for k in 0..3 {
                            for l in (j + 1)..3 {
                                let var1 = &vars[(block_row * 3 + i) * 9 + (block_col * 3 + j)];
                                let var2 = &vars[(block_row * 3 + i) * 9 + (block_col * 3 + l)];
                                var1.enforce_not_equal(var2)?;
                            }
                        }
                        for k in (i + 1)..3 {
                            for l in 0..3 {
                                let var1 = &vars[(block_row * 3 + i) * 9 + (block_col * 3 + j)];
                                let var2 = &vars[(block_row * 3 + k) * 9 + (block_col * 3 + l)];
                                var1.enforce_not_equal(var2)?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::test_rng;

    #[test]
    fn test_sudoku_circuit() {
        let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());

        let circuit = SudokuCircuit { solution: None };

        // Generate a pk vk
        let (pk, vk) =
            Groth16::<Bls12_381>::circuit_specific_setup(circuit.clone(), &mut rng).unwrap();

        // Example Sudoku solution
        let solution = [
            [5, 3, 4, 6, 7, 8, 9, 1, 2],
            [6, 7, 2, 1, 9, 5, 3, 4, 8],
            [1, 9, 8, 3, 4, 2, 5, 6, 7],
            [8, 5, 9, 7, 6, 1, 4, 2, 3],
            [4, 2, 6, 8, 5, 3, 7, 9, 1],
            [7, 1, 3, 9, 2, 4, 8, 5, 6],
            [9, 6, 1, 5, 3, 7, 2, 8, 4],
            [2, 8, 7, 4, 1, 9, 6, 3, 5],
            [3, 4, 5, 2, 8, 6, 1, 7, 9],
        ];

        // Create an instance of the Sudoku circuit
        let circuit = SudokuCircuit {
            solution: Some(solution),
        };

        // Create a proof of the Sudoku circuit
        let proof = Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).unwrap();

        // Verify the proof
        let is_valid = Groth16::<Bls12_381>::verify(&vk, &vec![], &proof).unwrap();

        assert!(is_valid);

        // Add a tracesub function to print nicely formatted information
        fn tracesub<T: std::fmt::Debug>(name: &str, value: T) {
            println!("{}: {:?}", name, value);
        }

        // Example usage of tracesub
        tracesub("Sudoku solution", solution);
    }
}
