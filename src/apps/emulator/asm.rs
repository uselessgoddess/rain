use {
  egui::{Context, CursorIcon, ScrollArea, Window, text::LayoutJob},
  egui_extras::syntax_highlighting::{self, CodeTheme},
  raki::{Instruction, Isa, OpcodeKind},
};

pub struct Asm {
  asm: Vec<(usize, Option<Instruction>)>,
  pub open: bool,
}

impl Default for Asm {
  fn default() -> Self {
    Self { asm: vec![], open: true }
  }
}

impl Asm {
  pub fn decode(&mut self, bytes: &[u8]) {
    use raki::Decode;

    fn read16(bytes: &[u8], addr: usize) -> u64 {
      return (bytes[addr] as u64) | ((bytes[addr + 1] as u64) << 8);
    }

    fn read32(bytes: &[u8], addr: usize) -> u64 {
      return (bytes[addr] as u64)
        | ((bytes[addr + 1] as u64) << 8)
        | ((bytes[addr + 2] as u64) << 16)
        | ((bytes[addr + 3] as u64) << 24);
    }

    let mut asm = Vec::new();

    let mut pc = 0;
    loop {
      if bytes.len() - pc < 2 {
        break;
      }
      let inst16 = read16(&bytes, pc);
      if let 0 | 1 | 2 = inst16 & 0b11 {
        let inst = (inst16 as u16).decode(Isa::Rv64).ok();
        asm.push((2, inst));
        pc += 2;
      } else {
        if bytes.len() - pc < 4 {
          break;
        }
        let inst = (read32(&bytes, pc) as u32).decode(Isa::Rv64).ok();
        asm.push((4, inst));
        pc += 4;
      }
    }

    self.asm = asm;
  }

  pub fn ui(&mut self, ctx: &Context) -> Option<usize> {
    let mut ret = None;

    if !self.open || self.asm.is_empty() {
      return None;
    }

    Window::new("Instructions").show(ctx, |ui| {
      ScrollArea::vertical().show(ui, |ui| {
        let style = ctx.style();
        let theme = CodeTheme::from_style(&style);

        let mut pc = 0;

        for &(size, ref line) in self.asm.iter() {
          let line = if let Some(inst) = line {
            format!("{inst}")
          } else {
            String::from("unknown instruction")
          };
          let job =
            syntax_highlighting::highlight(ctx, &style, &theme, &line, "rs");
          let galley = ui.fonts(|f| f.layout_job(job));

          let mut response = ui.label(galley);

          if response.hovered() {
            response = response.highlight();
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand)
          }

          if response.clicked() {
            ret = Some(pc);
          }
          pc += size;
        }
      })
    });

    ret
  }
}

fn parse_job(ctx: &Context, asm: &[Option<Instruction>]) -> LayoutJob {
  let asm: String = asm
    .iter()
    .map(|inst| {
      inst
        .as_ref()
        .map(|inst| inst.to_string())
        .unwrap_or("unknown instruction".into())
    })
    .intersperse("\n".into())
    .collect();

  let theme = CodeTheme::from_style(&ctx.style());
  syntax_highlighting::highlight(ctx, &ctx.style(), &theme, &asm, "rs")
}
