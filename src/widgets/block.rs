use egui::{Frame, Label, Response, RichText, Sense, UiBuilder};

#[derive(Default)]
pub struct Block;

impl Block {
  pub fn show(ui: &mut egui::Ui, show: impl FnOnce(&mut egui::Ui)) -> Response {
    ui.scope_builder(UiBuilder::new().sense(Sense::click()), |ui| {
      let response = ui.response();
      let visuals = ui.style().interact(&response);

      Frame::canvas(ui.style())
        .fill(visuals.bg_fill.gamma_multiply(0.3))
        .stroke(visuals.bg_stroke)
        .inner_margin(ui.spacing().menu_margin)
        .show(ui, |ui| {
          ui.set_width(ui.available_width());
          show(ui);
        });
    })
    .response
  }
}
