use {
  crate::client::Error,
  egui::{Rgba, RichText, Ui, collapsing_header::CollapsingState},
};

#[derive(Default)]
pub struct ErrorHeader {
  errors: Vec<Error>,
  debug: bool,
}

impl ErrorHeader {
  pub fn push(&mut self, error: Error) {
    self.errors.push(error);
  }

  pub fn clear(&mut self) {
    self.errors.clear();
  }

  pub fn ui(&mut self, ui: &mut Ui, label: &str) {
    fn red(label: String) -> RichText {
      RichText::new(label).color(Rgba::RED)
    }

    if self.errors.is_empty() {
      return;
    }

    let state_id = ui.id().with("header-state");

    CollapsingState::load_with_default_open(ui.ctx(), state_id, true)
      .show_header(ui, |ui| {
        ui.toggle_value(&mut self.debug, label);
      })
      .body(|ui| {
        for error in &self.errors {
          ui.label(red(if self.debug {
            format!("{error:?}")
          } else {
            error.to_string()
          }));
        }
      });
  }
}
