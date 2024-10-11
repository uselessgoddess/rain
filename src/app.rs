use {
  crate::{emulator::EmulatorPanel, login::AccountPanel, panels::SystemPanel},
  eframe::{CreationContext, Frame},
  egui::{
    Context, Key, Modifiers, RichText, SidePanel, ThemePreference,
    TopBottomPanel, Visuals, Window, widgets,
  },
  std::fmt,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Anchor {
  Account,
  Emulator,
}

impl fmt::Display for Anchor {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{self:?}")
  }
}

impl From<Anchor> for egui::WidgetText {
  fn from(value: Anchor) -> Self {
    Self::RichText(RichText::new(value.to_string()))
  }
}

impl Default for Anchor {
  fn default() -> Self {
    Self::Account
  }
}

impl eframe::App for AccountPanel {
  fn update(&mut self, ctx: &Context, _: &mut Frame) {
    Window::new("Account")
      .collapsible(false)
      .resizable(false)
      .show(ctx, |ui| self.ui(ui));
  }
}
impl eframe::App for EmulatorPanel {
  fn update(&mut self, ctx: &Context, _: &mut Frame) {
    self.ui(ctx);
  }
}

#[derive(Default)]
pub struct State {
  system_panel: SystemPanel,
  anchor: Anchor,

  account: AccountPanel,
  emulator: EmulatorPanel,
}

pub struct App {
  state: State,
}

impl App {
  pub fn new(cc: &CreationContext<'_>) -> Self {
    egui_extras::install_image_loaders(&cc.egui_ctx);

    // Use dark theme by default in the name of chaos
    cc.egui_ctx
      .memory_mut(|mem| mem.options.theme_preference = ThemePreference::Dark);

    cc.egui_ctx.style_mut(|style| {
      style.visuals.hyperlink_color = style.visuals.weak_text_color();
    });

    Self { state: State::default() }
  }

  fn apps_iter_mut(
    &mut self,
  ) -> impl Iterator<Item = (&str, Anchor, &mut dyn eframe::App)> {
    [
      (
        "Account",
        Anchor::Account,
        &mut self.state.account as &mut dyn eframe::App,
      ),
      (
        "Emulator",
        Anchor::Emulator,
        &mut self.state.emulator as &mut dyn eframe::App,
      ),
    ]
    .into_iter()
  }
}

impl eframe::App for App {
  fn clear_color(&self, visuals: &Visuals) -> [f32; 4] {
    let color = egui::lerp(
      egui::Rgba::from(visuals.panel_fill)
        ..=egui::Rgba::from(visuals.extreme_bg_color),
      0.5,
    );
    let color = egui::Color32::from(color);
    color.to_normalized_gamma_f32()
  }

  fn update(&mut self, ctx: &Context, frame: &mut Frame) {
    TopBottomPanel::top("app-top-bar")
      .frame(egui::Frame::none().inner_margin(4.0))
      .show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
          ui.visuals_mut().button_frame = false;
          self.bar_contents(ui, frame);
        });
      });

    if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F11)) {
      ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
        !ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)),
      ));
    }

    {
      self.state.system_panel.update(ctx, frame);

      self.system_panel(ctx, frame);

      self.show_selected(ctx, frame);

      self.state.system_panel.end_of_frame(ctx);
    }
  }
}

impl App {
  fn show_selected(&mut self, ctx: &Context, frame: &mut Frame) {
    let selected = self.state.anchor;
    for (_, anchor, app) in self.apps_iter_mut() {
      if anchor == selected || ctx.memory(|mem| mem.everything_is_visible()) {
        app.update(ctx, frame);
      }
    }
  }

  fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
    widgets::global_theme_preference_switch(ui);
    ui.separator();

    ui.toggle_value(&mut self.state.system_panel.open, "💻 System");
    ui.separator();

    let mut selected = self.state.anchor;
    for (name, anchor, _app) in self.apps_iter_mut() {
      if ui.selectable_label(selected == anchor, name).clicked() {
        selected = anchor;
        if frame.is_web() {
          ui.ctx().open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
        }
      }
    }
    self.state.anchor = selected;
  }

  fn system_panel(&mut self, ctx: &Context, frame: &mut Frame) {
    let is_open = self.state.system_panel.open
      || ctx.memory(|mem| mem.everything_is_visible());

    SidePanel::left("system_panel").resizable(false).show_animated(
      ctx,
      is_open,
      |ui| {
        ui.add_space(4.0);
        ui.vertical_centered(|ui| {
          ui.heading("💻 System");
        });

        ui.separator();
        self.system_panel_contents(ui, frame);
      },
    );
  }

  fn system_panel_contents(&mut self, ui: &mut egui::Ui, frame: &mut Frame) {
    self.state.system_panel.ui(ui, frame);

    ui.separator();

    ui.horizontal(|ui| {
      if ui
        .button("Reset egui")
        .on_hover_text("Forget scroll, positions, sizes etc")
        .clicked()
      {
        ui.ctx().memory_mut(|mem| *mem = Default::default());
        ui.close_menu();
      }
    });
  }
}
