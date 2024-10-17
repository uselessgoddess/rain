mod asm;

use {
  crate::{panels::MemoryEditor, widgets::HexEdit},
  asm::Asm,
  egui::{
    Button, Color32, Context, Key, KeyboardShortcut, Modifiers, RichText,
    Window,
  },
  egui_extras::{Size, StripBuilder},
  egui_file_dialog::FileDialog,
  std::fs,
};

#[derive(Default)]
struct State {
  pc: u64,
  memory: Vec<u8>,
}

#[derive(Default)]
pub struct EmulatorPanel {
  xregs: Xregs,
  dram: Memory,
  asm: Asm,

  state: State,
  dialog: FileDialog,
}

impl EmulatorPanel {
  pub fn ui(&mut self, ctx: &Context) {
    egui::TopBottomPanel::top("emulator-menu").show(ctx, |ui| {
      egui::menu::bar(ui, |ui| {
        self.file_menu_button(ui);
      });
    });

    self.dram.ui(ctx, &mut self.state);
    if let Some(pc) = self.asm.ui(ctx) {
      self.dram.editor.frame_data.set_highlight_address(pc);
    }
    Window::new("Registers")
      .collapsible(false)
      .fixed_size([370.0, 400.0])
      .show(ctx, |ui| {
        self.xregs.ui(ui);
      });

    self.dialog.update(ctx);

    if let Some(path) = self.dialog.take_selected() {
      match fs::read(path) {
        Ok(bytes) => {
          self.asm.decode(&bytes);
          self.state.memory = bytes;
        }
        Err(err) => {}
      }
    }

    self.dram.if_changed(|| {
      self.asm.decode(&self.state.memory);
    })
  }

  fn file_menu_button(&mut self, ui: &mut egui::Ui) {
    let open_shortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);

    if ui.input_mut(|i| i.consume_shortcut(&open_shortcut)) {
      self.dialog.select_file();
    }

    ui.menu_button("File", |ui| {
      {
        egui::gui_zoom::zoom_menu_buttons(ui);
        ui.weak(format!(
          "Current zoom: {:.0}%",
          100.0 * ui.ctx().zoom_factor()
        ))
        .on_hover_text(
          "The UI zoom level, on top of the operating system's default value",
        );
        ui.separator();
      }

      let button = Button::new("Open File")
        .shortcut_text(ui.ctx().format_shortcut(&open_shortcut));
      if ui.add(button).clicked() {
        self.dialog.select_file();
      }

      let button = Button::new("Toggle Asm");
      if ui.add(button).clicked() {
        self.asm.open = !self.asm.open;
      }
    });
  }
}

pub struct Memory {
  editor: MemoryEditor,
  changed: bool,
}

impl Default for Memory {
  fn default() -> Self {
    let editor = MemoryEditor::new()
      .with_address_range("All", 0..0x100000) // 1MB dram
      .with_address_range("Boot", 0xFF00..0xFF80)
      .with_window_title("Memory");

    Self { editor, changed: false }
  }
}

impl Memory {
  pub fn if_changed(&mut self, f: impl FnOnce()) {
    if self.changed {
      self.changed = false;
      f();
    }
  }

  pub fn ui(&mut self, ctx: &Context, state: &mut State) {
    self.editor.window_ui(
      ctx,
      &mut state.memory,
      |mem, addr| mem.get(addr).copied(),
      |mem, addr, val| {
        self.changed = true;
        if addr < mem.len() {
          mem[addr] = val
        }
      },
      |pc| {
        state.pc = pc as u64;
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
