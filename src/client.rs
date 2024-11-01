use {
  crate::{
    login::Account,
    repr::{
      login::Tokens,
      session::{SessionInfo, SessionRepr},
    },
  },
  json::{json, Value},
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
  #[error("{0}")]
  Internal(String),
}

impl Error {
  pub async fn parse<T: DeserializeOwned>(res: Response) -> Result<T, Self> {
    let status = res.status();

    let text = res.text().await?;
    if text.is_empty() {
      return Err(Self::Internal(format!(
        "internal error while building a request: {status}"
      )));
    }
    let value = json::from_str::<Value>(&text)?;

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

macro_rules! parse {
  ($expr:expr) => {
    $expr.send().await?.parse().await
  };
}

macro_rules! poll {
  ($expr:expr) => {
    Ok({
      $expr.send().await?;
    })
  };
}

pub struct Client {
  client: reqwest::Client,
  url: Url,
}

impl Client {
  pub fn new(url: Url) -> Self {
    Self { client: reqwest::Client::new(), url }
  }

  fn api(&self, method: impl AsRef<str>) -> Url {
    self.url.join(method.as_ref()).unwrap()
  }

  pub async fn login(&self, name: &str, pass: &str) -> Result<Tokens> {
    parse! {
      self
      .client
      .post(self.api("account/login"))
      .json(&json!({
        "username": name,
        "password": pass,
      }))
    }
  }

  pub async fn new_session(&self, auth: Account) -> Result<SessionRepr> {
    parse! {
      self
      .client
      .post(self.api("states/new"))
      .bearer_auth(auth.access)
    }
  }

  pub async fn load_session(
    &self,
    auth: Account,
    id: &str,
  ) -> Result<SessionRepr> {
    parse! {
      self
      .client
      .get(self.api("states/load"))
      .bearer_auth(auth.access)
      .query(&[("id", id)])
    }
  }

  pub async fn store_session(
    &self,
    auth: Account,
    repr: &SessionRepr,
  ) -> Result<()> {
    poll! {
      self
      .client
      .put(self.api("states/store"))
      .bearer_auth(auth.access)
      .json(repr)
    }
  }

  pub async fn remove_session(&self, auth: Account, id: &str) -> Result<()> {
    poll! {
      self
      .client
      .delete(self.api("states/remove"))
      .bearer_auth(auth.access)
      .json(id)
    }
  }

  pub async fn sessions(
    &self,
    auth: Account,
    page: usize,
    size: usize,
  ) -> Result<Vec<SessionInfo>> {
    parse! {
      self
        .client
        .get(self.api("states/sessions"))
        .bearer_auth(auth.access)
        .query(&[("page", page), ("size", size)])
    }
  }
}
