use ark_std::{end_timer, start_timer};
use bus_mapping::{circuit_input_builder::FixedCParams, mock::BlockData};
use eth_types::geth_types::GethData;
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{
        create_proof as create_proof_local, distributed_prover::{
            config:: WorkloadConfig,
            prover::create_proof as create_proof_distributed,
        },
        keygen_pk, keygen_vk, verify_proof,
        Circuit,
    },
    poly::{
        commitment::ParamsProver,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use mock::TestContext;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use zkevm_circuits::{
    evm_circuit::{EvmCircuit, witness::block_convert, TestEvmCircuit},
    util::SubCircuit
};

use crate::util::artifacts::*;
use crate::util::constants::RNG_SEED;

pub(crate) const CIRCUIT_NAME: &str = "evm";
const CIRCUIT_DEGREE: u32 = 18;

// What is the <Fr> notation?
// What is the pub(crate) notation?
pub(crate) fn circuit() -> EvmCircuit<Fr> { 
    let timer = start_timer!(|| "Create circuit");  // What is the "||" Notation?

    let empty_data: GethData = TestContext::<0, 0>::new(None, |_| {}, |_, _| {}, |b, _| b)
        .unwrap()
        .into();

    let mut builder =
        BlockData::new_from_geth_data_with_params(empty_data.clone(), FixedCParams::default())
            .new_circuit_input_builder();

    builder
        .handle_block(&empty_data.eth_block, &empty_data.geth_traces)
        .unwrap();

    let block = block_convert(&builder).unwrap();

    let circuit = TestEvmCircuit::<Fr>::new(block);

    end_timer!(timer);

    circuit
}

pub(crate) fn setup() {
    let circuit = circuit();

    // Setup params
    let timer = start_timer!(|| "Set up params");
    let mut rng = XorShiftRng::from_seed(RNG_SEED);
    let general_params = ParamsKZG::<Bn256>::setup(CIRCUIT_DEGREE, &mut rng);
    let verifier_params = general_params.verifier_params().clone();     // What's the purpose of cloning here?
    end_timer!(timer);

    // Generate Verification Key
    let timer = start_timer!(|| "Generate Verification Key");
    let vk = keygen_vk(&general_params, &circuit).unwrap();
    end_timer!(timer);

    // Read Network Configuration
    let timer = start_timer!(|| "Read Network Configuration");
    let network_config = read_workload_config(CIRCUIT_NAME);
    end_timer!(timer);
    let num_prover = network_config.num_prover();

    // Generate Workload Configuration
    let timer = start_timer!(|| "Generate Workload Configuration");
    // What is in a worklaod config? What is G1Affine? Is it an algorithm?
    let workload_config = WorkloadConfig::new_even_distribution::<G1Affine>(vk.cs(), num_prover);
    end_timer!(timer);

    // Generate Proving Key
    let timer = start_timer!(|| "Generate Proving Key");
    // Why we need to clone the vk to generate pk? Why can't we pass &vk to make it immutable
    let pk = keygen_pk(&general_params, vk.clone(), &circuit).unwrap();
    end_timer!(timer);

    // Artifact Serialization
    let timer = start_timer!(|| "Artifact Serialization");
    write_params_kzg(CIRCUIT_DEGREE, &general_params, false);
    write_params_kzg(CIRCUIT_DEGREE, &verifier_params, true);
    write_vk(CIRCUIT_NAME, &vk);
    write_workload_config(CIRCUIT_NAME, &workload_config);
    write_pk(CIRCUIT_NAME, &pk);
    end_timer!(timer);
}

pub(crate) fn prove(prover_index: usize) {
    let rng = XorShiftRng::from_seed(RNG_SEED);
    let circuit = circuit();
    // What is special about blake2b here and why are we using this instead of other hashing?
    // What is <_, G1Affine, Challenge255<_>> in between ::s?
    // What does this transcript contain?
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

    let timer = start_timer!(|| "Artifact Deserialization");
    let general_params = read_params_kzg(CIRCUIT_DEGREE, false);
    
    // TODO: check if the EVM circuit has the right syntax
    let mut pk = read_pk::<EvmCircuit<Fr>>(CIRCUIT_NAME, circuit.params());

    let network_config = read_network_config(CIRCUIT_NAME);
    let workload_config = read_workload_config(CIRCUIT_NAME);
    end_timer!(timer);

    let timer = start_timer!(|| "Evaluator configuration");
    pk.configure_evalutor(workload_config.for_prover(prover_index));
    end_timer!(timer);

    let timer = start_timer!(|| format!("Prover {} create_proof", prover_index));
    create_proof_distributed::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        XorShiftRng,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        TestEvmCircuit<Fr>,
    >(
        &general_params,
        &pk,
        &[circuit],
        &[&[]],
        rng,
        &mut transcript,
        &network_config,
        &workload_config,
        prover_index,
    ).unwrap();
    end_timer!(timer);
}

pub(crate) fn prove_local() {
    let rng = XorShiftRng::from_seed(RNG_SEED);
    let circuit = circuit();
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

    let timer = start_timer!(|| "Artifact deserialization");
    let general_params = read_params_kzg(CIRCUIT_DEGREE, false);
    // TODO: check if the EVM circuit has the right syntax
    let pk = read_pk::<EvmCircuit<Fr>>(CIRCUIT_NAME, circuit.params());
    end_timer!(timer);

    let timer = start_timer!(|| format!("Prover {} create_proof", 0));
    create_proof_local::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        XorShiftRng,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        TestEvmCircuit<Fr>,
    >(
        &general_params,
        &pk,
        &[circuit],
        &[&[]],
        rng,
        &mut transcript,
    ).unwrap();
    end_timer!(timer);

    let proof = transcript.finalize();
    let timer = start_timer!(|| "Artifact serialization");
    write_proof(CIRCUIT_NAME, &proof);
    end_timer!(timer);
}

pub(crate) fn verify() {
    let timer = start_timer!(|| "Artifact deserialization");
    let general_params = read_params_kzg(CIRCUIT_DEGREE, false);
    let verifier_params = read_params_kzg(CIRCUIT_DEGREE, true);

    let vk = read_vk::<EvmCircuit<Fr>>(CIRCUIT_NAME, circuit().params());
    let proof = read_proof(CIRCUIT_NAME);
    end_timer!(timer);

    // What does this verifier transcript contain?
    let mut verifier_transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&proof[..]);
    let strategy = SingleStrategy::new(&general_params);

    let timer = start_timer!(|| "Proof verification");
    verify_proof::<
        KZGCommitmentScheme<Bn256>,
        VerifierSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        Blake2bRead<&[u8], G1Affine, Challenge255<G1Affine>>,
        SingleStrategy<'_, Bn256>,
    >(
        &verifier_params,
        &vk,
        strategy,
        &[&[]],
        &mut verifier_transcript,
    ).unwrap();
    end_timer!(timer);
}