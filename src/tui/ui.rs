use color_eyre::Result;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{palette::material, Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Table, Wrap},
    Frame,
};

use super::app::App;

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

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

fn draw_popup(frame: &mut Frame, app: &App, area: Rect) -> Result<()> {
    frame.render_widget(Clear, area);
    let popup_block = Block::default()
        .title(Line::from("Confirm").centered())
        .title_bottom(
            Line::from(vec![
                " Confirm ".into(),
                "<Enter>".blue().bold(),
                " Decline ".into(),
                "<Esc>".blue().bold(),
            ])
            .centered(),
        )
        .borders(Borders::ALL);

    // the `trim: false` will stop the text from being cut off when over the edge of the block
    let lines = app.popup.prompt.lines().count();
    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(popup_block.inner(area));
    let vertical_padding = (vertical_layout[1].height.saturating_sub(lines as u16) / 2).max(1); // Ensure at least 1 line padding

    // Add vertical padding manually to center the text
    let padded_text = format!(
        "{}{}{}",
        "\n".repeat(vertical_padding as usize),
        app.popup.prompt,
        "\n".repeat(vertical_padding as usize)
    );

    let exit_paragraph = Paragraph::new(padded_text)
        .alignment(Alignment::Center)
        // .block(popup_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(popup_block, area);
    frame.render_widget(exit_paragraph, vertical_layout[1]);
    if app.popup.show_input {
        frame.render_widget(&app.popup.input, vertical_layout[2]);
    }
    Ok(())
}

fn draw_table(frame: &mut Frame, app: &mut App, area: Rect) -> Result<()> {
    let header_style = Style::default().bg(app.colors.header_bg);
    let selected_row_style = Style::default()
        .bg(app.colors.selected_row_style_bg)
        .fg(app.colors.selected_row_style_fg);

    let header = ["Ini", "Name", "HP", "AC", "Conditions"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);
    let rows = app.tracker.entities.iter().enumerate().map(|(i, data)| {
        let item = data.ref_array_string();
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
            Constraint::Percentage(20), // HP
            Constraint::Percentage(10), // AC
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
    frame.render_stateful_widget(t, area, &mut app.state);

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
        " Quit ".into(),
        "<Esc> ".blue().bold(),
    ]);
    let block = Block::default()
        .title(title.centered())
        .title_bottom(instructions.centered())
        .borders(Borders::ALL)
        .border_set(border::THICK);
    frame.render_widget(&block, frame.area());
    let inner_area = block.inner(frame.area());
    draw_table(frame, app, inner_area)?;
    if app.popup.active {
        let popup_area = centered_rect(app.popup.size.0, app.popup.size.1, frame.area());
        draw_popup(frame, app, popup_area)?;
    }
    Ok(())
}
