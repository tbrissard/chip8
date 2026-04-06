use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Widget,
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text, ToLine, ToText},
    widgets::{Block, Paragraph},
};

use crate::{
    app::App,
    cpu::{Instruction, Registers},
    keyboard::{Ch8Key, Ch8Keyboard, KeyState},
    screen::StandardScreen,
};

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

    let buf = frame.buffer_mut();
    render_screen(app.screen(), inner_layout[0], buf);
    render_keyboard(app.keyboard(), inner_layout[1], buf);
    render_history(app.history(), layout[1], buf);
    render_registers(app.registers(), layout[2], buf);
}

fn render_history(history: &[Instruction], area: Rect, buf: &mut Buffer) {
    let title = Line::from("Instructions").centered().bold();
    let block = Block::bordered().border_set(border::THICK).title(title);

    let available_height = area.height as usize - 2;

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

fn render_keyboard(keyboard: &Ch8Keyboard, area: Rect, buf: &mut Buffer) {
    let title = Line::from("Keyboard").bold().centered();
    let block = Block::bordered().title(title).border_set(border::THICK);
    let block_area = block.inner(area);

    let layout = Layout::horizontal(vec![Constraint::Length(3); 16])
        .spacing(3)
        .split(block_area);

    let regular = Style::default();
    let pressed = regular.add_modifier(Modifier::BOLD | Modifier::REVERSED);

    for (i, k) in Ch8Key::VARIANTS.into_iter().enumerate() {
        Span::styled(
            k.to_string(),
            if keyboard[k] == KeyState::Up {
                regular
            } else {
                pressed
            },
        )
        .render(layout[i], buf);
    }

    block.render(area, buf);
}

fn render_screen(screen: &StandardScreen, area: Rect, buf: &mut Buffer) {
    let title = Line::from("Display".bold());

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    let pixels = screen.to_text();

    Paragraph::new(pixels)
        .centered()
        .block(block)
        .render(area, buf);
}

fn render_registers(registers: &Registers, area: Rect, buf: &mut Buffer) {
    let title = Line::from("Registers".bold());
    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);
    let block_area = block.inner(area);

    let layout = Layout::horizontal(vec![Constraint::Length(8), Constraint::Length(22)])
        .spacing(3)
        .split(block_area);

    let v_registers = Text::from(
        registers
            .v_registers
            .iter()
            .enumerate()
            .map(|(i, vreg)| Line::from(format!("V{i:2}: {vreg:3}")))
            .collect::<Vec<_>>(),
    )
    .centered();

    let mut others = vec![
        Line::from(format!(
            "Program Counter: {:#05X}",
            registers.program_counter
        )),
        Line::from(format!("I: {:#05X}", registers.i)),
        Line::from(""),
        Line::from(format!("Delay Timer: {}", registers.delay_timer)),
        Line::from(format!("Sound Timer: {}", registers.sound_timer)),
        Line::from(""),
        Line::from(format!("Stack Pointer: {}", registers.stack_pointer)),
    ];
    others.extend(
        registers
            .stack
            .iter()
            .map(|addr| format!("{addr:#X}"))
            .map(Line::from),
    );
    let others = Text::from(others);

    v_registers.render(layout[0], buf);
    others.render(layout[1], buf);
    block.render(area, buf);
}
