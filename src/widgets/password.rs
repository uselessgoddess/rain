use egui::{Layout, TextEdit};

pub fn password_ui(ui: &mut egui::Ui, password: &mut String) -> egui::Response {
  let show_plaintext = if let Some(id) = ui.memory(|mem| mem.focused())
    && id == ui.id()
    && ui.input_mut(|i| i.modifiers.alt)
  {
    true
  } else {
    false
  };

  ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
    ui.add_sized(
      ui.available_size(),
      TextEdit::singleline(password)
        .password(!show_plaintext)
        .hint_text("password"),
    );
  })
  .response
}

pub fn password(password: &mut String) -> impl egui::Widget + '_ {
  move |ui: &mut egui::Ui| password_ui(ui, password)
}
