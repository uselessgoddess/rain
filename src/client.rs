use {
  crate::repr::login::Tokens,
  json::{Value, json},
  reqwest::{Response, Url},
  serde::de::DeserializeOwned,
  std::result,
};

pub type Result<T, E = Error> = result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("failed http")]
  Http(#[from] reqwest::Error),
  #[error("deserialize")]
  Serde(#[from] json::Error),
  #[error("{0}")]
  Client(String),
}

impl Error {
  pub async fn parse<T: DeserializeOwned>(res: Response) -> Result<T, Self> {
    let status = res.status();
    let value = json::from_str::<Value>(&res.text().await?)?;

    if status.is_client_error() {
      Err(Self::Client(value["errors"].to_string()))
    } else {
      Ok(json::from_value(value)?)
    }
  }
}

trait ErrorExt {
  async fn parse<T: DeserializeOwned>(self) -> Result<T>;
}

impl ErrorExt for Response {
  async fn parse<T: DeserializeOwned>(self) -> Result<T> {
    Error::parse(self).await
  }
}

pub struct Client {
  client: reqwest::Client,
  url: Url,
}

impl Client {
  pub fn new(url: Url) -> Self {
    Self { client: reqwest::Client::new(), url }
  }

  fn api(&self, method: &str) -> Url {
    self.url.join(method).unwrap()
  }

  pub async fn login(&self, name: &str, pass: &str) -> Result<Tokens> {
    self
      .client
      .post(self.api("account/login"))
      .json(&json!({
        "username": name,
        "password": pass,
      }))
      .send()
      .await?
      .parse()
      .await
  }
}
