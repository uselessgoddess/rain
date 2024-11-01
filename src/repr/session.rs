use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SessionInfo {
  pub id: String,
  pub user: String,
  pub name: Option<String>,
  pub creation: String,
  pub modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRepr {
  pub id: String,
  pub user: String,
  pub name: String,
  pub creation: String,
  pub modified: String,
  pub cpu: CpuRepr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuRepr {
  pub pc: u64,
  pub xregs: Vec<u64>,
  pub fregs: Vec<f64>,
  pub bus: Bus,
}

use serde_with::{base64::Base64, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bus {
  #[serde_as(as = "Base64")]
  pub dram: Vec<u8>,
}
