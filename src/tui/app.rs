use crate::combat::tracker::CombatTracker;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::widgets::TableState;
use tui_textarea::{Input, Key, TextArea};

use super::{
    cli::Args,
    terminal,
    ui::{self, TableColors},
};

#[derive(Default)]
pub struct Popup<'t> {
    pub active: bool,
    pub prompt: String,
    pub input: TextArea<'t>,
    pub show_input: bool,
    pub confirm_action: Option<Box<dyn FnMut(&mut App<'t>, String) + Send>>,
}

impl<'t> Popup<'t> {
    pub fn new() -> Self {
        let mut input = TextArea::default();
        input.set_alignment(ratatui::layout::Alignment::Center);
        Self {
            active: false,
            prompt: "".to_string(),
            input,
            show_input: false,
            confirm_action: None,
        }
    }
    pub fn show<F: FnMut(&mut App<'t>, String) + Send + 'static>(
        &mut self,
        prompt: &str,
        show_input: bool,
        action: F,
    ) {
        // clear the input
        self.input.select_all();
        self.input.cut();

        self.active = true;
        self.prompt = prompt.to_string();
        self.show_input = show_input;
        self.confirm_action = Some(Box::new(action));
    }

    pub fn hide(&mut self) {
        self.active = false;
    }
}

pub struct App<'t> {
    pub exit: bool,
    pub tracker: CombatTracker,
    pub state: TableState,
    pub popup: Popup<'t>,
    pub colors: TableColors,
}

impl<'t> App<'t> {
    pub fn new(args: &Args) -> Result<Self> {
        let mut combat = CombatTracker::from_yaml(&args.combat_file);
        combat.roll_initiative(true);
        Ok(Self {
            exit: false,
            tracker: combat,
            state: TableState::default().with_selected(0),
            popup: Popup::new(),
            colors: TableColors::new(),
        })
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<CombatTracker> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(self.tracker.clone())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if self.popup.active {
                    self.handle_popup_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                } else {
                    self.handle_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                }
            }
            _ => Ok(()),
        }
    }

    fn handle_popup_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.into() {
            Input { key: Key::Esc, .. } => self.popup.hide(),
            Input {
                key: Key::Enter, ..
            } => {
                if let Some(mut action) = self.popup.confirm_action.take() {
                    action(self, self.popup.input.lines().join(""));
                }
                self.popup.hide();
            }
            input => {
                self.popup.input.input(input);
            }
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> Result<()> {
        match key_event.into() {
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('q'),
                ctrl: true,
                ..
            } => {
                self.confirm_close();
            }
            Input {
                key: Key::Right, ..
            } => {
                self.tracker.next_turn();
            }
            Input { key: Key::Left, .. } => {
                self.tracker.prev_turn();
            }
            Input { key: Key::Down, .. } => {
                self.state.select_next();
            }
            Input { key: Key::Up, .. } => {
                self.state.select_previous();
            }
            Input {
                key: Key::Char('d'),
                ..
            } => {
                self.damage_heal(false);
            }
            Input {
                key: Key::Char('h'),
                ..
            } => {
                self.damage_heal(true);
            }
            _text_input => {}
        }
        Ok(())
    }

    fn damage_heal(&mut self, heal: bool) {
        let prompt = if heal {
            "Enter heal amount:"
        } else {
            "Enter damage amount:"
        };
        if let Some(selected) = self.state.selected() {
            self.popup.show(prompt, true, move |app, input_amount| {
                if let Ok(amount) = input_amount.parse::<i32>() {
                    if let Some(entity) = app.tracker.entities.get_mut(selected) {
                        entity.current_hp = if heal {
                            (entity.current_hp + amount).max(0).min(entity.max_hp)
                        } else {
                            (entity.current_hp - amount).max(0).min(entity.max_hp)
                        }
                    }
                }
            });
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn confirm_close(&mut self) {
        self.popup
            .show("Do you want to close the application?", false, |app, _| {
                app.exit()
            });
    }
}
