use halo2_proofs::halo2curves::bn256::Fr;
use bus_mapping::{circuit_input_builder::FixedCParams, mock::BlockData};
use eth_types::geth_types::GethData;
use mock::TestContext;
use zkevm_circuits::evm_circuit::{EvmCircuit, witness::block_convert, TestEvmCircuit};

use crate::circuits::common::CircuitHelper;

pub struct EvmCircuitHelper;

impl CircuitHelper for EvmCircuitHelper {
    type ConcreteCircuit = EvmCircuit<Fr>;

    const NAME: &'static str = "evm";
    const DEGREE: u32 = 18;
    const RNG_SEED: [u8; 16] = [0x59, 0x62, 0xbe, 0x5d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06, 0xbc, 0xe5];

    fn circuit() -> Self::ConcreteCircuit {
        let empty_data: GethData = TestContext::<0, 0>::new(None, |_| {}, |_, _| {}, |b, _| b).unwrap().into();
        let mut builder = BlockData::new_from_geth_data_with_params(empty_data.clone(), FixedCParams::default()).new_circuit_input_builder();
        builder.handle_block(&empty_data.eth_block, &empty_data.geth_traces).unwrap();
        let block = block_convert(&builder).unwrap();
        TestEvmCircuit::<Fr>::new(block)
    }
}
