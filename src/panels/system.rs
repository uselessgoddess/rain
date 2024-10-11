use {
  super::FrameHistory,
  eframe::Frame,
  egui::{Grid, Window},
  std::{thread, time::Duration},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
  Reactive,
  Continuous,
}

impl Default for RunMode {
  fn default() -> Self {
    Self::Continuous
  }
}

#[derive(Default)]
pub struct SystemPanel {
  pub open: bool,
  mode: RunMode,
  frame_history: FrameHistory,
  egui_windows: EguiWindows,
}

impl SystemPanel {
  pub fn update(&mut self, ctx: &egui::Context, frame: &Frame) {
    self
      .frame_history
      .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

    match self.mode {
      RunMode::Continuous => {
        ctx.request_repaint();
      }
      RunMode::Reactive => {}
    }
  }

  pub fn end_of_frame(&mut self, ctx: &egui::Context) {
    self.egui_windows.windows(ctx);
  }

  pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
    integration_ui(ui, frame);
    ui.separator();

    self.run_mode_ui(ui);
    ui.separator();

    self.frame_history.ui(ui);
    ui.separator();

    self.egui_windows.checkboxes(ui);
    ui.separator();
  }

  fn run_mode_ui(&mut self, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
      let run_mode = &mut self.mode;
      ui.label("Mode:");
      ui.radio_value(run_mode, RunMode::Reactive, "Reactive").on_hover_text(
        "Repaint when there are animations or input (e.g. mouse movement)",
      );
      ui.radio_value(run_mode, RunMode::Continuous, "Continuous")
        .on_hover_text("Repaint everything each frame");
    });

    if self.mode == RunMode::Continuous {
      ui.label(format!(
        "Repainting the UI each frame. FPS: {:.1}",
        self.frame_history.fps()
      ));
    } else {
      ui.label("Only running UI code when there are animations or input.");
    }
  }
}

use {eframe::wgpu, tracing::info};

fn integration_ui(ui: &mut egui::Ui, _frame: &mut Frame) {
  if let Some(render_state) = _frame.wgpu_render_state() {
    let wgpu_adapter_details_ui =
      |ui: &mut egui::Ui, adapter: &wgpu::Adapter| {
        let info = &adapter.get_info();

        let wgpu::AdapterInfo {
          name,
          vendor,
          device,
          device_type,
          driver,
          driver_info,
          backend,
        } = &info;

        Grid::new("adapter_info").show(ui, |ui| {
          ui.label("Backend:");
          ui.label(format!("{backend:?}"));
          ui.end_row();

          ui.label("Device Type:");
          ui.label(format!("{device_type:?}"));
          ui.end_row();

          if !name.is_empty() {
            ui.label("Name:");
            ui.label(format!("{name:?}"));
            ui.end_row();
          }
          if !driver.is_empty() {
            ui.label("Driver:");
            ui.label(format!("{driver:?}"));
            ui.end_row();
          }
          if !driver_info.is_empty() {
            ui.label("Driver info:");
            ui.label(format!("{driver_info:?}"));
            ui.end_row();
          }
          if *vendor != 0 {
            ui.label("Vendor:");
            ui.label(format!("0x{vendor:04X}"));
            ui.end_row();
          }
          if *device != 0 {
            ui.label("Device:");
            ui.label(format!("0x{device:02X}"));
            ui.end_row();
          }
        });
      };

    let wgpu_adapter_ui = |ui: &mut egui::Ui, adapter: &wgpu::Adapter| {
      let info = &adapter.get_info();
      ui.label(format!("{:?}", info.backend)).on_hover_ui(|ui| {
        wgpu_adapter_details_ui(ui, adapter);
      });
    };

    Grid::new("wgpu_info").num_columns(2).show(ui, |ui| {
      ui.label("Renderer:");
      ui.hyperlink_to("wgpu", "https://wgpu.rs/");
      ui.end_row();

      ui.label("Backend:");
      wgpu_adapter_ui(ui, &render_state.adapter);
      ui.end_row();

      if 1 < render_state.available_adapters.len() {
        ui.label("Others:");
        ui.vertical(|ui| {
          for adapter in &*render_state.available_adapters {
            if adapter.get_info() != render_state.adapter.get_info() {
              wgpu_adapter_ui(ui, adapter);
            }
          }
        });
        ui.end_row();
      }
    });
  }
}

struct EguiWindows {
  settings: bool,
  inspection: bool,
  memory: bool,
}

impl Default for EguiWindows {
  fn default() -> Self {
    Self::none()
  }
}

impl EguiWindows {
  fn none() -> Self {
    Self { settings: false, inspection: false, memory: false }
  }

  fn checkboxes(&mut self, ui: &mut egui::Ui) {
    let Self { settings, inspection, memory } = self;

    // ui.checkbox(settings, "🔧 Settings");
    ui.checkbox(inspection, "🔍 Inspection");
    ui.checkbox(memory, "📝 Memory");
  }
  fn windows(&mut self, ctx: &egui::Context) {
    let Self { settings, inspection, memory } = self;

    //Window::new("🔧 Settings").open(settings).vscroll(true).show(ctx, |ui| {
    //  ctx.settings_ui(ui);
    //});

    Window::new("🔍 Inspection").open(inspection).vscroll(true).show(
      ctx,
      |ui| {
        ctx.inspection_ui(ui);
      },
    );

    Window::new("📝 Memory").open(memory).resizable(false).show(ctx, |ui| {
      ctx.memory_ui(ui);
    });
  }
}
