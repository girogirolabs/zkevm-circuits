use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::{
        create_proof as create_proof_local,
        distributed_prover::prover::create_proof as create_proof_distributed,
        keygen_pk, keygen_vk, verify_proof,
        Circuit, ConstraintSystem, Error,
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
        Blake2bRead, Blake2bWrite,
        Challenge255,
        TranscriptReadBuffer, TranscriptWriterBuffer,
    },
    timer::Timer,
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
        let timer = Timer::new("set up params");
        let mut rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let general_params = ParamsKZG::<Bn256>::setup(Self::DEGREE, &mut rng);
        let verifier_params = general_params.verifier_params().clone();
        timer.end();

        let timer = Timer::new("generate verfication key");
        let vk = keygen_vk(&general_params, &circuit).unwrap();
        timer.end();

        let timer = Timer::new("generate proving key");
        let pk = keygen_pk(&general_params, vk.clone(), &circuit).unwrap();
        timer.end();

        let timer = Timer::new("artifact serialization");
        write_params_kzg(Self::DEGREE, &general_params, false);
        write_params_kzg(Self::DEGREE, &verifier_params, true);
        write_vk(Self::NAME, &vk);
        write_pk(Self::NAME, &pk);
        timer.end();
    }

    fn prove(prover_index: usize) -> Result<(), Error> {
        let rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let circuit = Self::circuit();
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

        let timer = Timer::new("artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let mut pk = read_pk::<Self::ConcreteCircuit>(&Self::NAME, circuit.params());
        let network_config = read_network_config(Self::NAME);
        let workload_config = read_workload_config(Self::NAME);
        timer.end();

        let timer = Timer::new("evaluator configuration");
        pk.configure_evalutor(workload_config.for_prover(prover_index));
        timer.end();

        let timer = Timer::new(&format!("Prover {} create_proof", prover_index));
        let result = create_proof_distributed::<
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
        );
        timer.end();

        // Only leader should serialize the proof
        if prover_index == 0 {
            let proof = transcript.finalize();
            let timer = Timer::new("artifact serialization");
            write_proof(Self::NAME, &proof);
            timer.end();
        }

        result
    }

    fn prove_local() -> Result<(), Error> {
        let rng = XorShiftRng::from_seed(Self::RNG_SEED);
        let circuit = Self::circuit();
        let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

        let timer = Timer::new("artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let pk = read_pk::<Self::ConcreteCircuit>(&Self::NAME, circuit.params());
        timer.end();

        let timer = Timer::new(&format!("Prover {} create_proof", 0));
        let result = create_proof_local::<
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
        );
        timer.end();

        let proof = transcript.finalize();
        let timer = Timer::new("artifact serialization");
        write_proof(&Self::NAME, &proof);
        timer.end();

        result
    }

    fn verify() -> Result<(), Error> {
        let timer = Timer::new("artifact deserialization");
        let general_params = read_params_kzg(Self::DEGREE, false);
        let verifier_params = read_params_kzg(Self::DEGREE, true);
        let vk = read_vk::<Self::ConcreteCircuit>(&Self::NAME, Self::circuit().params());
        let proof = read_proof(Self::NAME);
        timer.end();

        let mut verifier_transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(&proof[..]);
        let strategy = SingleStrategy::new(&general_params);

        let timer = Timer::new("proof verification");
        let result = verify_proof::<
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
        );
        timer.end();

        result
    }
}
