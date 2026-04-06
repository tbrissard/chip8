use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    prelude::Widget,
    style::Stylize,
    symbols::border,
    text::{Line, Text, ToLine},
    widgets::{Block, Paragraph},
};

use crate::{app::App, screen::StandardScreen};

pub(crate) fn draw(app: &App, frame: &mut Frame) {
    let layout = Layout::horizontal(vec![
        Constraint::Length(StandardScreen::WIDTH as u16 + 2),
        Constraint::Length(17),
        Constraint::Length(35),
    ])
    .split(frame.area());

    let inner_layout = Layout::vertical(vec![
        Constraint::Length(StandardScreen::HEIGHT as u16 + 2),
        Constraint::Length(5),
    ])
    .split(layout[0]);

    frame.render_widget(&app.cpu.screen, inner_layout[0]);
    frame.render_widget(&app.cpu.keyboard, inner_layout[1]);
    render_history(app, layout[1], frame.buffer_mut());
    frame.render_widget(&app.cpu.registers, layout[2]);
}

fn render_history(app: &App, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
    let title = Line::from("Instructions").centered().bold();
    let block = Block::bordered().border_set(border::THICK).title(title);

    let available_height = area.height as usize - 2;

    let history = app.history();
    let len = history.len();
    let text = Text::from(
        history[len.saturating_sub(available_height)..len]
            .iter()
            .map(|instr| instr.to_line())
            .collect::<Vec<_>>(),
    );

    Paragraph::new(text)
        .centered()
        .block(block)
        .render(area, buf);
}
