use std::sync::{Arc, Mutex};

use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    prelude::Widget,
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text, ToLine, ToText},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    emulator::{Instruction, Registers, Shared, Stats},
    input::{self},
    keyboard::{Ch8Key, Ch8Keyboard, KeyState},
    screen::StandardScreen,
};

pub(crate) fn draw(shared: &Arc<Mutex<Shared>>, frame: &mut Frame) {
    let shared = shared.lock().unwrap();
    let layout = Layout::horizontal(vec![
        Constraint::Length(StandardScreen::WIDTH as u16 + 2),
        Constraint::Min(29),
        Constraint::Length(17),
        Constraint::Length(35),
    ])
    .split(frame.area());

    let second_column = Layout::vertical(vec![
        Constraint::Length(15),
        Constraint::Length(input::KEYBINDS.len() as u16 + 2),
    ])
    .split(layout[1]);

    let buf = frame.buffer_mut();
    render_screen(&shared.screen, layout[0], buf);
    render_keyboard(
        &shared.keyboard,
        second_column[0].centered_horizontally(Constraint::Length(29)),
        buf,
    );
    render_keybinds(second_column[1], buf);
    render_history(&shared.history, layout[2], buf);
    render_registers(&shared.registers, &shared.stats, layout[3], buf);
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

fn render_keybinds(area: Rect, buf: &mut Buffer) {
    let title = Line::from("Keybinds").bold().centered();
    let block = Block::bordered().title(title).border_set(border::THICK);
    let text = input::KEYBINDS
        .iter()
        .map(|(key_code, (_, description))| format!("{key_code}: {description}"))
        .collect::<Text>();
    Paragraph::new(text).block(block).render(area, buf);
}

fn render_keyboard(keyboard: &Ch8Keyboard, area: Rect, buf: &mut Buffer) {
    const HORIZONTAL_SPACING: u16 = 3;
    const VERTICAL_SPACING: u16 = 1;

    let columns = Layout::horizontal(vec![Constraint::Length(5); 4])
        .spacing(HORIZONTAL_SPACING)
        .split(area);
    let cells = columns
        .iter()
        .flat_map(|l| {
            Layout::vertical(vec![Constraint::Length(3); 4])
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
            Ch8Key::Zero => cells[7],
            Ch8Key::One => cells[2],
            Ch8Key::Two => cells[6],
            Ch8Key::Three => cells[10],
            Ch8Key::Four => cells[1],
            Ch8Key::Five => cells[5],
            Ch8Key::Six => cells[9],
            Ch8Key::Seven => cells[0],
            Ch8Key::Eight => cells[4],
            Ch8Key::Nine => cells[8],
            Ch8Key::A => cells[3],
            Ch8Key::B => cells[11],
            Ch8Key::C => cells[12],
            Ch8Key::D => cells[13],
            Ch8Key::E => cells[14],
            Ch8Key::F => cells[15],
        };

        let cell_block = Block::bordered().style(style);
        let inner = cell_block.inner(cell);
        cell_block.render(cell, buf);

        Paragraph::new(k.to_string())
            .style(Style::new().add_modifier(Modifier::BOLD))
            .centered()
            .render(inner, buf);
    }
}

fn render_screen(screen: &StandardScreen, area: Rect, buf: &mut Buffer) {
    let title = Line::from("Display".bold());
    let layout = Layout::vertical(vec![
        Constraint::Length(StandardScreen::HEIGHT as u16 + 2),
        Constraint::Fill(1),
    ])
    .split(area);

    let block = Block::bordered()
        .title(title.centered())
        .border_set(border::THICK);

    let pixels = screen.to_text();

    Paragraph::new(pixels)
        .centered()
        .block(block)
        .render(layout[0], buf);
}

fn render_registers(registers: &Registers, stats: &Stats, area: Rect, buf: &mut Buffer) {
    let title = Line::from("Registers".bold());
    let block = Block::default()
        .borders(Borders::TOP | Borders::BOTTOM | Borders::RIGHT)
        .title(title.centered())
        .border_set(border::THICK);
    let block_area = block.inner(area);

    let layout = Layout::horizontal(vec![Constraint::Length(8), Constraint::Length(22)])
        .spacing(3)
        .split(block_area);

    let layout2 =
        Layout::vertical(vec![Constraint::Fill(1), Constraint::Length(4)]).split(layout[1]);

    let v_registers = Text::from(
        registers
            .values
            .iter()
            .enumerate()
            .map(|(i, vreg)| Line::from(format!("V{i:2}: {vreg:3}")))
            .collect::<Vec<_>>(),
    )
    .centered();

    let mut others = vec![
        Line::from(format!("Program Counter: {:#05X}", registers.pc)),
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

    let stats = Text::from(vec![
        Line::from(format!("Uptime: {}ms", stats.uptime.as_millis())),
        Line::from(format!("Cycles count: {}", stats.cycles)),
    ]);

    v_registers.render(layout[0], buf);
    others.render(layout[1], buf);
    block.render(area, buf);
    stats.render(layout2[1], buf);
}
