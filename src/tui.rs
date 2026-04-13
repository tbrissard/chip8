use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Widget,
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text, ToLine, ToText},
    widgets::{Block, Borders, Padding, Paragraph},
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
        Constraint::Fill(1),
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

    const HORIZONTAL_SPACING: u16 = 3;
    const VERTICAL_SPACING: u16 = 1;

    let layout = Layout::horizontal(vec![Constraint::Percentage(25); 4])
        .spacing(HORIZONTAL_SPACING)
        .split(block_area);
    let inners = layout
        .iter()
        .flat_map(|l| {
            Layout::vertical(vec![Constraint::Length(5); 4])
                .spacing(VERTICAL_SPACING)
                .split(*l)
                .to_vec()
        })
        .collect::<Vec<_>>();

    let regular = Style::default();
    let pressed = regular.add_modifier(Modifier::BOLD | Modifier::REVERSED);

    for k in Ch8Key::VARIANTS.into_iter() {
        let style = if let KeyState::Up = keyboard[k] {
            regular
        } else {
            pressed
        };

        let cell = match k {
            Ch8Key::Zero => inners[7],
            Ch8Key::One => inners[2],
            Ch8Key::Two => inners[6],
            Ch8Key::Three => inners[10],
            Ch8Key::Four => inners[1],
            Ch8Key::Five => inners[5],
            Ch8Key::Six => inners[9],
            Ch8Key::Seven => inners[0],
            Ch8Key::Eight => inners[4],
            Ch8Key::Nine => inners[8],
            Ch8Key::A => inners[3],
            Ch8Key::B => inners[11],
            Ch8Key::C => inners[12],
            Ch8Key::D => inners[13],
            Ch8Key::E => inners[14],
            Ch8Key::F => inners[15],
        };

        let cell_block = Block::bordered().padding(Padding::vertical(1)).style(style);
        let inner = cell_block.inner(cell);
        cell_block.render(cell, buf);

        Paragraph::new(k.to_string())
            .style(Style::new().add_modifier(Modifier::BOLD))
            .centered()
            .render(inner, buf);
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
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
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
