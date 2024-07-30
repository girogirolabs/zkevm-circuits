use halo2_proofs::halo2curves::bn256::Fr;
use zkevm_circuits::{
    keccak_circuit::{KeccakCircuit, TestKeccakCircuit},
    util::SubCircuit
};

use crate::circuits::common::CircuitHelper;

pub struct KeccakCircuitHelper;

impl CircuitHelper for KeccakCircuitHelper {
    type ConcreteCircuit = KeccakCircuit<Fr>;

    const NAME: &'static str = "keccak";
    const DEGREE: u32 = 11;
    const RNG_SEED: [u8; 16] = [0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06, 0xbc, 0xe5];

    fn circuit() -> Self::ConcreteCircuit {
        let num_rows = 2usize.pow(Self::DEGREE) - TestKeccakCircuit::<Fr>::unusable_rows();
        let inputs = vec![(0u8..135u8).collect::<Vec<_>>(); 3];
        TestKeccakCircuit::new(num_rows, inputs)
    }
}
