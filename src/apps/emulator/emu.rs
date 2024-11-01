use {
  super::asm::Asm,
  egui_toast::{Toast, ToastKind},
  std::{fs, time::Duration},
  tokio::time::Instant,
};

use crate::{
  client::Result,
  panels::MemoryEditor,
  repr::session::{Bus, CpuRepr, SessionRepr},
  tx,
  widgets::HexEdit,
  Arx,
};

use {
  crate::login::Account,
  egui::{
    Align2, Button, Color32, Context, Direction, Key, KeyboardShortcut,
    Modifiers, RichText, WidgetText, Window,
  },
  egui_extras::{Size, StripBuilder},
  egui_file_dialog::FileDialog,
  egui_toast::Toasts,
};

#[derive(Default, Clone)]
struct State {
  pc: u64,
  memory: Vec<u8>,
}

#[derive(Default)]
pub struct Panel {
  xregs: Xregs,
  dram: Memory,
  asm: Asm,

  exit: bool,
  state: State,
  dialog: FileDialog,

  name: String,
}

pub struct EmulatorPanel {
  panel: Panel,
  repr: SessionRepr,

  instant: Instant,
  toasts: Toasts,

  auth: Account,
  synx: Arx<Result<()>>,
}

impl EmulatorPanel {
  pub fn new(auth: Account, repr: SessionRepr) -> Self {
    let mut panel = Panel::default();

    panel.store_repr(repr.clone());

    Self {
      panel,
      repr,
      auth,
      instant: Instant::now(),
      toasts: Toasts::new()
        .anchor(Align2::RIGHT_TOP, (10.0, 10.0))
        .direction(Direction::TopDown),
      synx: Default::default(),
    }
  }

  pub fn ui(&mut self, ctx: &Context) -> bool {
    if let Some(mut arx) = self.synx.ready() {
      if let Ok(Err(err)) = arx.try_recv() {
        self
          .toasts
          .add(Toast::new().kind(ToastKind::Error).text(err.to_string()));
      }
    } else if self.instant.elapsed() >= Duration::from_secs(1) {
      self.instant = Instant::now();
      self.sync_repr();
    }

    self.toasts.show(ctx);

    self.panel.ui(ctx)
  }

  pub fn sync_repr(&mut self) {
    let State { pc, memory } = self.panel.state.clone();
    let repr = SessionRepr {
      name: self.panel.name.clone(),
      cpu: CpuRepr {
        pc,
        bus: Bus { dram: memory },
        fregs: vec![],
        xregs: self.panel.xregs.regs.iter().map(|(x, _)| *x).collect(),
      },
      ..self.repr.clone()
    };

    let auth = self.auth.clone();
    let task = self.synx.task();
    tokio::spawn(async move {
      task.send(tx().client.store_session(auth, &repr).await)
    });
  }
}

impl Panel {
  pub fn store_repr(&mut self, SessionRepr { name, cpu, .. }: SessionRepr) {
    self.state.memory = cpu.bus.dram;
    self.state.pc = cpu.pc;
    for i in 0..cpu.xregs.len().min(32) {
      self.xregs.regs[i].0 = cpu.xregs[i];
    }

    self.asm.decode(&self.state.memory);
    self.name = name;
  }

  pub fn ui(&mut self, ctx: &Context) -> bool {
    egui::TopBottomPanel::top("emulator-menu").show(ctx, |ui| {
      egui::menu::bar(ui, |ui| {
        self.file_menu_button(ui);

        ui.text_edit_singleline(&mut self.name);
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
    });

    self.exit
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

      fn button(
        ui: &mut egui::Ui,
        name: impl Into<WidgetText>,
        short: impl Into<Option<(Modifiers, Key)>>,
        clicked: impl FnOnce(&mut egui::Ui),
      ) {
        let mut button = Button::new(name);
        if let Some((md, key)) = short.into() {
          let short = KeyboardShortcut::new(md, key);
          button = button.shortcut_text(ui.ctx().format_shortcut(&short));

          if ui.input_mut(|i| i.consume_shortcut(&short)) {
            return clicked(ui);
          }
        }
        if ui.add(button).clicked() {
          return clicked(ui);
        }
      }

      button(ui, "Open file", (Modifiers::CTRL, Key::O), |_| {
        self.dialog.select_file();
      });

      button(ui, "Toggle asm", (Modifiers::ALT, Key::A), |_| {
        self.asm.open = !self.asm.open;
      });

      button(ui, "Leave session", (Modifiers::ALT, Key::X), |_| {
        self.exit = true;
      });

      button(ui, "Organize windows", None, |ui| {
        ui.ctx().memory_mut(|mem| mem.reset_areas());
      });
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
