use {
  crate::{
    Arx,
    client::Result,
    repr::login::Tokens,
    widgets::{self, ErrorHeader},
  },
  egui::{Align, Id, Layout, Link, RichText, TextEdit},
  egui_toast::{Toast, ToastKind, ToastOptions, ToastStyle, Toasts},
};

#[derive(Clone)]
pub struct Account {
  pub access: String,
  pub name: Option<String>,
  pub refresh: Option<String>,
}

#[derive(Default)]
pub struct Login {
  pub login: String,
  pub password: String,

  arx: Arx<(Result<Tokens>, String)>,
  errors: ErrorHeader,
}

impl Login {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<Account> {
    horizontal_input(ui, &mut self.login, "login");

    ui.horizontal(|ui| {
      ui.add(widgets::password(&mut self.password));
    });

    if let Some(mut arx) = self.arx.ready() {
      if let Ok((tokens, name)) = arx.try_recv() {
        match tokens {
          Ok(Tokens { access, refresh }) => {
            return Some(Account {
              access,
              name: Some(name),
              refresh: Some(refresh),
            });
          }
          Err(error) => self.errors.push(error),
        }
      } else {
        ui.spinner();
      }
    }

    self.errors.ui(ui, "Something went wrong");

    ui.vertical_centered(|ui| {
      if ui.button("login").clicked() {
        let sender = self.arx.task();

        let tx = crate::CONTEXT.clone();
        let (name, pass) = (self.login.clone(), self.password.clone());

        tokio::spawn(async move {
          sender.send((tx.client.login(&name, &pass).await, name));
        });
      }
    });

    None
  }
}

#[derive(Default)]
pub struct Token {
  token: String,
}

impl Token {
  pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<Account> {
    horizontal_input(ui, &mut self.token, "token");

    ui.vertical_centered(|ui| {
      if ui.button("login").clicked() {
        Some(Account { access: self.token.clone(), name: None, refresh: None })
      } else {
        None
      }
    })
    .inner
  }
}

pub enum Panel {
  Login(Login),
  Token(Token),
}

impl Default for Panel {
  fn default() -> Self {
    Self::Login(Login::new())
  }
}

#[derive(Default)]
pub struct AccountPanel {
  panel: Panel,
  toasts: Toasts,
}

impl AccountPanel {
  pub fn account(&self, ui: &egui::Ui) -> Option<Account> {
    ui.memory(|mem| mem.data.get_temp::<Account>(Id::NULL))
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    if let Some(Account { name, access, refresh }) = self.account(ui) {
      if let Some(name) = name {
        ui.heading("username");
        ui.label(name);
        ui.separator();
      }

      ui.collapsing(RichText::new("access").heading(), |ui| {
        ui.label(access);
      });
      ui.separator();

      if let Some(refresh) = refresh {
        ui.heading("refresh");
        ui.label(refresh);
        ui.separator();
      }

      ui.vertical_centered(|ui| {
        if ui.button("logout").clicked() {
          ui.memory_mut(|mem| mem.data.remove_by_type::<Account>())
        }
      });
    } else {
      let (account, link, panel) = match &mut self.panel {
        Panel::Login(login) => {
          (login.ui(ui), "sign in with token", Panel::Token(Token::default()))
        }
        Panel::Token(token) => (
          token.ui(ui),
          "sign in with password",
          Panel::Login(Login::default()),
        ),
      };

      ui.with_layout(Layout::top_down(Align::Max), |ui| {
        ui.horizontal(|ui| {
          horizontal_link(ui, link, || self.panel = panel);
          horizontal_link(ui, "sign up", || {
            self.toasts.add(Toast {
              kind: ToastKind::Error,
              text: "Error".into(),
              options: ToastOptions::default(),
              style: ToastStyle::default(),
            });
          });
        })
      });

      if let Some(account) = account {
        ui.memory_mut(|mem| mem.data.insert_temp(Id::NULL, account));
      }

      self.toasts.show(ui.ctx());
    }
  }
}

fn horizontal_link(ui: &mut egui::Ui, label: &str, clicked: impl FnOnce()) {
  if ui.add(Link::new(label)).clicked() {
    clicked();
  }
}

fn horizontal_input(ui: &mut egui::Ui, input: &mut String, hint: &str) {
  ui.horizontal(|ui| {
    ui.add_sized(
      ui.available_size(),
      TextEdit::singleline(input).hint_text(hint),
    );
  });
}
