use serde::{Serialize, Deserialize};

pub mod circuits;
pub mod artifacts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Circuit {
    EVM,
    Keccak,
}
