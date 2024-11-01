use {
  crate::{
    client::Result,
    login::Account,
    repr::session::{SessionInfo, SessionRepr},
    tx,
    widgets::Block,
    Arx,
  },
  egui::{Align2, CollapsingHeader, Context, ScrollArea, Window},
  emu::EmulatorPanel,
};

mod asm;
mod emu;

impl SessionInfo {
  pub fn ui(&self, ui: &mut egui::Ui, idx: usize) {
    let Self { id, name, user: owner, creation, modified } = self;

    if let Some(name) = name {
      ui.strong(name);
    } else {
      ui.weak("untitled");
    }
    ui.horizontal(|ui| {
      ui.label("owner:");
      ui.strong(owner);
    });
    CollapsingHeader::new("meta").id_salt(idx).show(ui, |ui| {
      ui.horizontal(|ui| {
        ui.label("id:");
        ui.strong(id);
      });
      ui.horizontal(|ui| {
        ui.label("creation:");
        ui.strong(creation);
      });
      ui.horizontal(|ui| {
        ui.label("modified:");
        ui.strong(modified);
      });
    });
  }
}

#[derive(Default)]
pub struct Sessions {
  loaded: bool,
  sessions: Vec<SessionInfo>,

  pagex: Arx<Result<Vec<SessionInfo>>>,
  loadx: Arx<Result<SessionRepr>>,
  newx: Arx<Result<SessionRepr>>,
  rmx: Arx<Option<usize>>,
}

impl Sessions {
  pub fn ui(&mut self, ctx: &Context, auth: Account) -> Option<SessionRepr> {
    Window::new("Sessions")
      .collapsible(false)
      .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
      .show(ctx, |ui| {
        if ui.button("New").clicked() {
          let auth = auth.clone();
          let task = self.newx.task();
          tokio::spawn(async move {
            task.send(tx().client.new_session(auth).await);
          });
        }

        let scroll = ScrollArea::vertical()
          .show(ui, |ui| self.show_sessions(ui, auth.clone()));

        if let Some(session) = scroll.inner {
          let auth = auth.clone();
          let task = self.loadx.task();
          tokio::spawn(async move {
            task.send(tx().client.load_session(auth, &session.id).await);
          });
        }

        if !self.loaded {
          self.loaded = true;
          let auth = auth.clone();
          let task = self.pagex.task();
          tokio::spawn(async move {
            task.send(tx().client.sessions(auth, 0, 10).await);
          });
        }

        macro_rules! poll_rx {
          ($task:expr => |$val:ident| $expr:expr) => {
            if let Some(mut arx) = $task.ready() {
              if let Ok($val) = arx.try_recv() {
                $expr
              } else {
                ui.spinner();
              }
            }
          };
        }

        poll_rx!(&mut self.pagex => |sessions| {
          self.sessions.extend(sessions.unwrap());
        });

        poll_rx!(&mut self.newx => |session| {
          return Some(session.unwrap());
        });

        poll_rx!(&mut self.loadx => |session| {
          return Some(session.unwrap());
        });

        None
      })
      .unwrap()
      .inner
      .unwrap()
  }

  fn show_sessions(
    &mut self,
    ui: &mut egui::Ui,
    auth: Account,
  ) -> Option<SessionInfo> {
    for (idx, session) in self.sessions.clone().into_iter().enumerate() {
      let response = Block::show(ui, |ui| {
        session.ui(ui, idx);
        if self.rmx.ready().is_none() {
          if ui.button("remove").clicked() {
            let auth = auth.clone();
            let session = session.clone();
            let task = self.rmx.task();
            tokio::spawn(async move {
              task.send(
                Some(
                  tx().client.remove_session(auth, &session.id).await.unwrap(),
                )
                .map(|_| idx),
              )
            });
          }
        } else {
          ui.spinner();
        }
      });

      if response.clicked() {
        return Some(session);
      }
    }

    if let Some(mut arx) = self.rmx.ready() {
      if let Ok(Some(idx)) = arx.try_recv() {
        self.sessions.remove(idx);
      }
    }

    None
  }
}

pub enum SessionPanel {
  Sessions(Sessions),
  Emulator(EmulatorPanel),
}

impl Default for SessionPanel {
  fn default() -> Self {
    Self::Sessions(Sessions::default())
  }
}

impl SessionPanel {
  pub fn ui(&mut self, ctx: &Context, auth: Account) {
    match self {
      SessionPanel::Sessions(me) => {
        if let Some(session) = me.ui(ctx, auth.clone()) {
          *self = SessionPanel::Emulator(EmulatorPanel::new(auth, session));
        }
      }
      SessionPanel::Emulator(me) => {
        if me.ui(ctx) {
          *self = SessionPanel::Sessions(Sessions::default());
        }
      }
    }
  }
}
