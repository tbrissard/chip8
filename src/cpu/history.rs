use std::fmt::Display;

use ratatui::{
    style::Stylize,
    text::{Line, ToText},
    widgets::{Block, Paragraph, Widget},
};

use crate::cpu::Instructions;

#[derive(Debug, Default)]
pub(crate) struct History {
    inner: Vec<Instructions>,
}

impl History {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn push(&mut self, instr: Instructions) {
        self.inner.push(instr);
    }

    // pub(super) fn pop(&mut self) -> Option<Instructions> {
    //     self.inner.pop()
    // }
}

impl Widget for &History {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Instructions").centered().bold();
        let block = Block::bordered().title(title);

        let history = self.to_text();
        let offset = (history.height() as u16).saturating_sub(area.height);

        Paragraph::new(history)
            .centered()
            .scroll((offset, 0))
            .block(block)
            .render(area, buf);
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for instr in &self.inner {
            writeln!(f, "{instr}")?;
        }
        Ok(())
    }
}
