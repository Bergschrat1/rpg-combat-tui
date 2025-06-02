use color_eyre::Result;
use ratatui::prelude::Stylize;
use ratatui::style::palette::material;
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

use crate::app::App;

#[derive(Debug)]
pub struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_row_style_bg: Color,
    current_turn_style_fg: Color,
    current_turn_style_bg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    pub const fn new() -> Self {
        Self {
            buffer_bg: material::GREEN.c900,
            header_bg: material::GREEN.c800,
            header_fg: material::GREEN.c200,
            row_fg: material::GREEN.c200,
            selected_row_style_fg: material::BLACK,
            selected_row_style_bg: material::GREEN.c100,
            current_turn_style_fg: material::BLACK,
            current_turn_style_bg: material::BLUE.c200,
            normal_row_color: material::GREEN.c900,
            alt_row_color: material::GREEN.c700,
            footer_border_color: material::GREEN.c400,
        }
    }
}

pub fn draw(frame: &mut Frame, app: &mut App) -> Result<()> {
    let title = Line::from(format!("RPG Combat TUI, Round: {}", app.tracker.round).bold());

    let instructions = Line::from(vec![
        " Prev Turn ".into(),
        "<Left>".blue().bold(),
        " Next Turn ".into(),
        "<Right>".blue().bold(),
        " Select Up ".into(),
        "<Up>".blue().bold(),
        " Select Down ".into(),
        "<Down>".blue().bold(),
        " Toggle Condition ".into(),
        "C".blue().bold(),
        " Assign Damage ".into(),
        "D".blue().bold(),
        " Heal ".into(),
        "H".blue().bold(),
        " Re-roll Initiative ".into(),
        "<Ctrl>+R".blue().bold(),
        " Quit ".into(),
        "<Esc> ".blue().bold(),
        " Reset Combat ".into(),
        "<Ctrl>+C".blue().bold(),
    ]);
    let block = Block::default()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .borders(Borders::ALL)
        .border_set(border::THICK);
    frame.render_widget(&block, frame.area());
    let inner_area = block.inner(frame.area());
    let state = Line::from(format!("{:?}", app.tracker));
    frame.render_widget(state, inner_area);
    // draw_table(frame, app, inner_area)?;
    // if app.popup.active {
    //     let popup_area = centered_rect(app.popup.size.0, app.popup.size.1, frame.area());
    //     draw_popup(frame, app, popup_area)?;
    // }
    Ok(())
}
