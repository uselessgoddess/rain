use {
  crate::{
    Arx,
    client::Result,
    repr::login::Tokens,
    widgets::{self, ErrorHeader},
  },
  egui::{Id, RichText, TextEdit},
};

#[derive(Clone)]
pub struct Account {
  pub name: String,
  pub access: String,
  pub refresh: String,
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
    ui.horizontal(|ui| {
      ui.add_sized(
        ui.available_size(),
        TextEdit::singleline(&mut self.login).hint_text("login"),
      );
    });

    ui.horizontal(|ui| {
      ui.add(widgets::password(&mut self.password));
    });

    if let Some(mut arx) = self.arx.ready() {
      if let Ok((tokens, name)) = arx.try_recv() {
        match tokens {
          Ok(Tokens { access, refresh }) => {
            return Some(Account { name, access, refresh });
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
pub struct AccountPanel {
  pub login: Login,
}

impl AccountPanel {
  pub fn account(&self, ui: &egui::Ui) -> Option<Account> {
    ui.memory(|mem| mem.data.get_temp::<Account>(Id::NULL))
  }

  pub fn ui(&mut self, ui: &mut egui::Ui) {
    if let Some(Account { name, access, refresh }) = self.account(ui) {
      ui.heading("username");
      ui.label(name);
      ui.separator();

      ui.collapsing(RichText::new("access").heading(), |ui| {
        ui.label(access);
      });
      ui.separator();

      ui.heading("refresh");
      ui.label(refresh);
      ui.separator();

      ui.vertical_centered(|ui| {
        if ui.button("logout").clicked() {
          ui.memory_mut(|mem| mem.data.remove_by_type::<Account>())
        }
      });
    } else if let Some(account) = self.login.ui(ui) {
      ui.memory_mut(|mem| mem.data.insert_temp(Id::NULL, account));
    }
  }
}
