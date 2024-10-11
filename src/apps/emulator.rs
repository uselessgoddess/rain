use {
  crate::{panels::MemoryEditor, widgets::HexEdit},
  egui::{Color32, Context, RichText, Window},
  egui_extras::{Size, StripBuilder},
};

#[derive(Default)]
pub struct EmulatorPanel {
  xregs: Xregs,
  dram: Memory,
}

impl EmulatorPanel {
  pub fn ui(&mut self, ctx: &Context) {
    self.dram.ui(ctx);
    Window::new("Registers")
      .collapsible(false)
      .fixed_size([370.0, 400.0])
      .show(ctx, |ui| {
        self.xregs.ui(ui);
      });
  }

  pub fn show_registers(&mut self, ui: &mut egui::Ui) {}
}

pub struct Memory {
  editor: MemoryEditor,
  memory: Vec<u8>,
}

impl Default for Memory {
  fn default() -> Self {
    let editor = MemoryEditor::new()
      .with_address_range("All", 0..0x100000) // 1MB dram
      .with_address_range("Boot", 0xFF00..0xFF80)
      .with_window_title("Memory");

    Self { editor, memory: vec![123; 1024] }
  }
}

impl Memory {
  pub fn ui(&mut self, ctx: &Context) {
    self.editor.window_ui(
      ctx,
      &mut self.memory,
      |mem, addr| mem.get(addr).copied(),
      |mem, addr, val| {
        if addr < mem.len() {
          mem[addr] = val
        }
      },
    );
  }
}

#[derive(Default)]
pub struct Xregs {
  regs: [(u64, HexEdit); 32],
}

impl Xregs {
  pub fn ui(&mut self, ui: &mut egui::Ui) {
    StripBuilder::new(ui).sizes(Size::remainder(), 16).vertical(|mut strip| {
      let (chunks, _) = self.regs.as_chunks_mut::<16>();
      for i in 0..16 {
        strip.strip(|builder| {
          builder.sizes(Size::remainder(), 2).horizontal(|mut strip| {
            for (xi, chunk) in chunks.iter_mut().enumerate() {
              let (x, edit) = &mut chunk[i];

              let idx = xi * 16 + i;
              strip.cell(|ui| {
                ui.horizontal(|ui| {
                  ui.label(
                    RichText::new(format!("x{idx:02}"))
                      .color(Color32::from_rgb(0, 140, 140)),
                  );
                  if idx == 0 {
                    // ui.label(hint);
                  } else {
                    edit.show(ui, x);
                  }
                });
              });
            }
          });
        });
      }
    });
  }
}
