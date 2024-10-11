use {
  eframe::emath::Align,
  egui::{FontId, TextEdit},
};

#[derive(Default)]
pub struct HexEdit {
  edit: String,
  imm: bool,
}

impl HexEdit {
  pub fn show(&mut self, ui: &mut egui::Ui, value: &mut u64) {
    let mut imm = String::new();
    let output =
      TextEdit::singleline(if self.imm { &mut imm } else { &mut self.edit })
        .font(FontId::monospace(14.0))
        .hint_text("0x0000000000000000")
        .horizontal_align(Align::Max)
        .show(ui);

    let raw = self.edit.trim_start_matches("0x");

    if let Some(id) = ui.memory(|mem| mem.focused())
      && id == output.response.id
    {
    } else if let Ok(new) = u64::from_str_radix(raw, 16) {
      *value = new;
      let radix = format!("0x{:016x}", *value);
      self.edit = if *value == 0 { String::new() } else { radix };
    } else {
      let radix = format!("0x{:016x}", *value);
      self.edit = if *value == 0 { String::new() } else { radix };
    }
  }
}
