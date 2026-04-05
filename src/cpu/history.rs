use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text, ToLine},
    widgets::{Block, Paragraph, Widget},
};

use crate::cpu::Instruction;

#[derive(Debug, Default)]
pub(crate) struct History {
    inner: Vec<Instruction>,
}

impl History {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push(&mut self, instr: Instruction) {
        self.inner.push(instr);
    }
}

impl Widget for &History {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Instructions").centered().bold();
        let block = Block::bordered().border_set(border::THICK).title(title);

        let available_height = area.height as usize - 2;

        let history = Text::from(
            self.inner[self.inner.len().saturating_sub(available_height)..self.inner.len()]
                .iter()
                .map(|instr| instr.to_line())
                .collect::<Vec<_>>(),
        );

        Paragraph::new(history)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
