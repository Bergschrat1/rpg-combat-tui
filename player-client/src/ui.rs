use core::combat::entity;
use core::dto::PlayerClientEntity;

use color_eyre::Result;
use itertools::Itertools;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::Stylize;
use ratatui::style::palette::material;
use ratatui::style::{Color, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Cell, HighlightSpacing, Row, Table};
use ratatui::Frame;

use crate::app::App;
use crate::utils::get_health_state;

#[allow(unused)]
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

pub fn get_ref_array_string(entity: &PlayerClientEntity) -> Vec<String> {
    // TODO this funciton should not be the responsibility of the Entity
    let display_name = if entity.id != 0 {
        format!("{} ({})", entity.name, entity.id)
    } else {
        entity.name.to_string()
    };
    vec![
        entity.initiative.to_string(),
        display_name,
        format!("{}", get_health_state(entity.max_hp, entity.current_hp)),
        entity.conditions.iter().sorted().join(", "),
    ]
}

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) -> Result<()> {
    let header_style = Style::default().bg(app.colors.header_bg);
    let selected_row_style = Style::default()
        .bg(app.colors.selected_row_style_bg)
        .fg(app.colors.selected_row_style_fg);
    let header = ["Ini", "Name", "Health", "Conditions"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    // let tracker = app.tracker;
    let rows = app.tracker.entities.iter().enumerate().map(|(i, data)| {
        let item = get_ref_array_string(data);
        let color_bg = {
            if i == app.tracker.current_turn {
                // highlight current turn
                app.colors.current_turn_style_bg
            } else {
                ratatui::style::Color::Reset
            }
        };
        let color_fg = {
            if i == app.tracker.current_turn {
                // highlight current turn
                app.colors.current_turn_style_fg
            } else {
                ratatui::style::Color::Reset
            }
        };
        item.into_iter()
            .map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
            .collect::<Row>()
            .style(Style::new().bg(color_bg).fg(color_fg))
            .height(4)
    });
    let bar = " █ ";
    let t = Table::new(
        rows,
        [
            Constraint::Percentage(10), // Initiative
            Constraint::Percentage(30), // Name
            Constraint::Percentage(20), // Health
            Constraint::Fill(1),        // Conditions
        ],
    )
    .header(header)
    .row_highlight_style(selected_row_style)
    // .bg(app.colors.buffer_bg)
    .highlight_spacing(HighlightSpacing::Always)
    .highlight_symbol(Text::from(vec![
        "".into(),
        bar.into(),
        bar.into(),
        "".into(),
    ]));
    // dbg!(highlight_symbol_color);
    frame.render_widget(t, area);
    Ok(())
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
    // let state = Line::from(format!("{:?}", app.tracker));
    // frame.render_widget(state, inner_area);
    draw_table(frame, app, inner_area)?;
    // if app.popup.active {
    //     let popup_area = centered_rect(app.popup.size.0, app.popup.size.1, frame.area());
    //     draw_popup(frame, app, popup_area)?;
    // }
    Ok(())
}
