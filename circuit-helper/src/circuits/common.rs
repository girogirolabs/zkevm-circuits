use ark_std::{end_timer, start_timer};
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr, G1Affine}, plonk::{
        create_proof as create_proof_local,
        distributed_prover::prover::create_proof as create_proof_distributed,
        keygen_pk, keygen_vk, verify_proof,
        Circuit, ConstraintSystem,
    }, poly::{
        commitment::ParamsProver,
        kzg::{
            commitment::{KZGCommitmentScheme, ParamsKZG},
            multiopen::{ProverSHPLONK, VerifierSHPLONK},
            strategy::SingleStrategy,
        },
    }, transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    }
};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;

use crate::artifacts::*;

pub trait CircuitHelper
{
    type ConcreteCircuit: Circuit<Fr>;

    const NAME: &'static str;
    const DEGREE: u32;
    const RNG_SEED: [u8; 16];

    fn circuit() -> Self::ConcreteCircuit;

    fn constraint_system() -> ConstraintSystem<Fr> {
        let vk = read_vk::<Self::ConcreteCircuit>(&Self::NAME, Self::circuit().params());
        vk.cs().clone()
    }

    fn setup() {
        let circuit = Self::circuit();
        let timer = start_timer!(|| "Set up params");
        let mut rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let general_params = ParamsKZG::<Bn256>::setup(Self::DEGREE, &mut rng);
        let verifier_params = general_params.verifier_params().clone();
        end_timer!(timer);

        let timer = start_timer!(|| "Generate verfication key");
        let vk = keygen_vk(&general_params, &circuit).unwrap();
        end_timer!(timer);

        let timer = start_timer!(|| "Generate proving key");
        let pk = keygen_pk(&general_params, vk.clone(), &circuit).unwrap();
        end_timer!(timer);

        let timer = start_timer!(|| "Artifact serialization");
        write_params_kzg(Self::DEGREE, &general_params, false);
        write_params_kzg(Self::DEGREE, &verifier_params, true);
        write_vk(Self::NAME, &vk);
        write_pk(Self::NAME, &pk);
        end_timer!(timer);
    }

    fn prove(prover_index: usize) {
        let rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let circuit = Self::circuit();
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

        let timer = start_timer!(|| "Artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let mut pk = read_pk::<Self::ConcreteCircuit>(&Self::NAME, circuit.params());
        let network_config = read_network_config(Self::NAME);
        let workload_config = read_workload_config(Self::NAME);
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
            Self::ConcreteCircuit,
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

        // Only leader should serialize the proof
        if prover_index == 0 {
            let proof = transcript.finalize();
            let timer = start_timer!(|| "Artifact serialization");
            write_proof(Self::NAME, &proof);
            end_timer!(timer);
        }
    }

    fn prove_local() {
        let rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let circuit = Self::circuit();
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

        let timer = start_timer!(|| "Artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let pk = read_pk::<Self::ConcreteCircuit>(&Self::NAME, circuit.params());
        end_timer!(timer);

        let timer = start_timer!(|| format!("Prover {} create_proof", 0));
        create_proof_local::<
            KZGCommitmentScheme<Bn256>,
            ProverSHPLONK<'_, Bn256>,
            Challenge255<G1Affine>,
            XorShiftRng,
            Blake2bWrite<Vec<u8>, G1Affine, Challenge255<G1Affine>>,
            Self::ConcreteCircuit,
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
        write_proof(&Self::NAME, &proof);
        end_timer!(timer);
    }

    fn verify() {
        let timer = start_timer!(|| "Artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let verifier_params = read_params_kzg(Self::DEGREE, true);
        let vk = read_vk::<Self::ConcreteCircuit>(&Self::NAME, Self::circuit().params());
        let proof = read_proof(Self::NAME);
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
}
