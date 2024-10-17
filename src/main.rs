#![feature(let_chains, array_chunks, slice_as_chunks, iter_intersperse)]
#![deny(clippy::all)]
#![forbid(unsafe_code)] // Who left this here???
#![allow(irrefutable_let_patterns)]

mod app;
mod apps;
mod client;
mod panels;
mod repr;
mod tx;
mod utils;
mod widgets;

use {
  crate::{app::App, tx::TyCtx},
  eframe::egui,
  lazy_static::lazy_static,
  std::sync::Arc,
};

pub use {apps::*, utils::Arx};

lazy_static! {
  pub static ref CONTEXT: Arc<TyCtx> = Arc::new(TyCtx::new());
}

#[tokio::main]
async fn main() -> eframe::Result {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([1280.0, 1024.0])
      .with_drag_and_drop(true),

    renderer: eframe::Renderer::Wgpu,

    ..Default::default()
  };
  eframe::run_native("rain", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}
