use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tokens {
  pub access: String,
  pub refresh: String,
}
