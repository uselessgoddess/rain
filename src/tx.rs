use {crate::client::Client, reqwest::Url, std::sync::Arc};

pub struct TyCtx {
  pub client: Client,
}

impl TyCtx {
  pub fn new() -> Self {
    Self {
      client: Client::new(Url::parse("http://localhost:6000/api/").unwrap()),
    }
  }
}

pub type Tx = Arc<TyCtx>;
