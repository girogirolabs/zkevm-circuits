use std:: {
    fs::File,
    path::Path,
    io::{BufReader, BufWriter, Read, Write},
};
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr, G1Affine},
    plonk::distributed_prover::config::{NetworkConfig, WorkloadConfig},
    plonk::{Circuit, ProvingKey, VerifyingKey},
    poly::kzg::commitment::ParamsKZG,
    SerdeFormat,
};

mod path {
    use std::path::PathBuf;

    pub(super) fn artifacts_root() -> PathBuf {
        let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root.push("artifacts");
        root
    }

    pub(super) fn params_kzg(
        degree: u32,
        is_verifier_params: bool,
    ) -> PathBuf {
        let mut path = artifacts_root();
        path.push(format!(
            "params_kzg_deg_{}{}.bin",
            degree,
            if is_verifier_params { "_verifier" } else { "" }
        ));
        path
    }

    pub(super) fn pk(circuit_name: &str) -> PathBuf {
        let mut path = artifacts_root();
        path.push(circuit_name);
        path.push(format!("pk.bin"));
        path
    }

    pub(super) fn vk(circuit_name: &str) -> PathBuf {
        let mut path = artifacts_root();
        path.push(circuit_name);
        path.push("vk.bin");
        path
    }

    pub(super) fn proof(circuit_name: &str) -> PathBuf {
        let mut path = artifacts_root();
        path.push(circuit_name);
        path.push("proof.bin");
        path
    }

    pub(super) fn network_config(circuit_name: &str) -> PathBuf {
        let mut path = artifacts_root();
        path.push(circuit_name);
        path.push("network_config.json");
        path
    }

    pub(super) fn workload_config(circuit_name: &str) -> PathBuf {
        let mut path = artifacts_root();
        path.push(circuit_name);
        path.push("workload_config.json");
        path
    }
}

pub(crate) fn write_params_kzg(
    degree: u32,
    params_kzg: &ParamsKZG<Bn256>,
    is_verifier_params: bool,
) {
    let f = File::create(path::params_kzg(degree, is_verifier_params)).unwrap();
    let mut writer = BufWriter::new(f);
    params_kzg.write_custom(&mut writer, SerdeFormat::RawBytes).unwrap();
}

pub(crate) fn read_params_kzg(
    degree: u32,
    is_verifier_params: bool,
) -> ParamsKZG<Bn256> {
    let f = File::open(path::params_kzg(degree, is_verifier_params)).unwrap();
    let mut reader = BufReader::new(f);
    ParamsKZG::<Bn256>::read_custom(&mut reader, SerdeFormat::RawBytes).unwrap()
}

pub(crate) fn params_kzg_exists(
    degree: u32,
    is_verifier_params: bool,
) -> bool {
    Path::exists(&path::params_kzg(degree, is_verifier_params))
}

pub(crate) fn read_pk<ConcreteCircuit: Circuit<Fr>> (
    circuit_name: &str,
    circuit_params: ConcreteCircuit::Params,
) -> ProvingKey<G1Affine> {
    let f = File::open(path::pk(circuit_name)).unwrap();
    let mut reader = BufReader::new(f);
    ProvingKey::<G1Affine>::read::<_, ConcreteCircuit>(
        &mut reader,
        SerdeFormat::RawBytes,
        circuit_params,
    ).unwrap()
}

pub(crate) fn write_pk(
    circuit_name: &str,
    pk: &ProvingKey<G1Affine>,
) {
    let f = File::create(path::pk(circuit_name)).unwrap();
    let mut writer = BufWriter::new(f);
    pk.write(&mut writer, SerdeFormat::RawBytes).unwrap();
}

pub(crate) fn pk_exists(circuit_name: &str) -> bool {
    Path::exists(&path::pk(circuit_name))
}

pub(crate) fn read_vk<ConcreteCircuit: Circuit<Fr>>(
    circuit_name: &str,
    circuit_params: ConcreteCircuit::Params,
) -> VerifyingKey<G1Affine> {
    let f = File::open(path::vk(circuit_name)).unwrap();
    let mut reader = BufReader::new(f);

    VerifyingKey::<G1Affine>::read::<_, ConcreteCircuit>(
        &mut reader,
        SerdeFormat::RawBytes,
        circuit_params,
    ).unwrap()
}

pub(crate) fn write_vk(
    circuit_name: &str,
    vk: &VerifyingKey<G1Affine>,
) {
    let f = File::create(path::vk(circuit_name)).unwrap();
    let mut writer = BufWriter::new(f);
    vk.write(&mut writer, SerdeFormat::RawBytes).unwrap();
}

pub(crate) fn vk_exists(circuit_name: &str) -> bool {
    Path::exists(&path::vk(circuit_name))
}

pub(crate) fn write_proof(circuit_name: &str, proof: &[u8]) {
    let mut f = File::create(path::proof(circuit_name)).unwrap();
    f.write_all(proof).unwrap();
    f.flush().unwrap();
}

pub(crate) fn read_proof(circuit_name: &str) -> Vec<u8> {
    let f = File::open(path::proof(circuit_name)).unwrap();
    let mut reader = BufReader::new(f);
    let mut proof = Vec::new();
    reader.read_to_end(&mut proof).unwrap();
    proof
}

pub(crate) fn read_network_config(circuit_name: &str) -> NetworkConfig {
    let f = File::open(path::network_config(circuit_name)).unwrap();
    let mut reader = BufReader::new(f);
    NetworkConfig::read(&mut reader).unwrap()
}

pub fn write_network_config(
    circuit_name: &str,
    config: &NetworkConfig,
) {
    let f = File::create(path::network_config(circuit_name)).unwrap();
    let mut writer = BufWriter::new(f);
    config.write(&mut writer).unwrap();
}

pub(crate) fn read_workload_config(circuit_name: &str) -> WorkloadConfig {
    let f = File::open(path::workload_config(circuit_name)).unwrap();
    let mut reader = BufReader::new(f);
    WorkloadConfig::read(&mut reader).unwrap()
}

pub fn write_workload_config(
    circuit_name: &str,
    config: &WorkloadConfig,
) {
    let f = File::create(path::workload_config(circuit_name)).unwrap();
    let mut writer = BufWriter::new(f);
    config.write(&mut writer).unwrap();
}
