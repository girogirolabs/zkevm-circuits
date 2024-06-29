use ark_std::{end_timer, start_timer};
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
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use zkevm_circuits::{
    keccak_circuit::{KeccakCircuit, TestKeccakCircuit},
    util::SubCircuit
};

use crate::util::artifacts::*;
use crate::util::constants::RNG_SEED;

pub(crate) const CIRCUIT_NAME: &str = "keccak";
const CIRCUIT_DEGREE: u32 = 11;

pub(crate) fn circuit() -> KeccakCircuit<Fr> {
    let timer = start_timer!(|| "Create circuit");
    let num_rows = 2usize.pow(CIRCUIT_DEGREE) - TestKeccakCircuit::<Fr>::unusable_rows();
    let inputs = vec![(0u8..135u8).collect::<Vec<_>>(); 3];
    let circuit = TestKeccakCircuit::new(num_rows, inputs);
    end_timer!(timer);

    circuit
}

pub(crate) fn setup() {
    let circuit = circuit();

    let timer = start_timer!(|| "Set up params");
    let mut rng = XorShiftRng::from_seed(RNG_SEED);
    let general_params = ParamsKZG::<Bn256>::setup(CIRCUIT_DEGREE, &mut rng);
    let verifier_params = general_params.verifier_params().clone();
    end_timer!(timer);

    let timer = start_timer!(|| "Generate verfication key");
    let vk = keygen_vk(&general_params, &circuit).unwrap();
    end_timer!(timer);

    let timer = start_timer!(|| "Read network configuration");
    let network_config = read_network_config(CIRCUIT_NAME);
    end_timer!(timer);
    let num_prover = network_config.num_prover();

    let timer = start_timer!(|| "Generate workload configuration");
    let workload_config = WorkloadConfig::new_even_distribution::<G1Affine>(vk.cs(), num_prover);
    end_timer!(timer);

    let timer = start_timer!(|| "Generate proving key");
    let pk = keygen_pk(&general_params, vk.clone(), &circuit).unwrap();
    end_timer!(timer);

    let timer = start_timer!(|| "Artifact serialization");
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
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

    let timer = start_timer!(|| "Artifact deserialization");
    let general_params = read_params_kzg(CIRCUIT_DEGREE, false);
    let mut pk = read_pk::<KeccakCircuit<Fr>>(CIRCUIT_NAME, circuit.params());
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
        TestKeccakCircuit<Fr>,
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

    let proof = transcript.finalize();
    let timer = start_timer!(|| "Artifact serialization");
    write_proof(CIRCUIT_NAME, &proof);
    end_timer!(timer);
}

pub(crate) fn prove_local() {
    let rng = XorShiftRng::from_seed(RNG_SEED);
    let circuit = circuit();
    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

    let timer = start_timer!(|| "Artifact deserialization");
    let general_params = read_params_kzg(CIRCUIT_DEGREE, false);
    let pk = read_pk::<KeccakCircuit<Fr>>(CIRCUIT_NAME, circuit.params());
    end_timer!(timer);

    let timer = start_timer!(|| format!("Prover {} create_proof", 0));
    create_proof_local::<
        KZGCommitmentScheme<Bn256>,
        ProverSHPLONK<'_, Bn256>,
        Challenge255<G1Affine>,
        XorShiftRng,
        Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
        TestKeccakCircuit<Fr>,
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
    let vk = read_vk::<KeccakCircuit<Fr>>(CIRCUIT_NAME, circuit().params());
    let proof = read_proof(CIRCUIT_NAME);
    end_timer!(timer);

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
