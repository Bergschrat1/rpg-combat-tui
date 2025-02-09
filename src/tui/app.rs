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
    pub fn show<F: FnMut(&mut App<'t>, String) + Send + 'static>(
        &mut self,
        prompt: &str,
        show_input: bool,
        action: F,
    ) {
        self.active = true;
        self.prompt = prompt.to_string();
        self.input = TextArea::default();
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
    pub fn new(args: Args) -> Result<Self> {
        let mut combat = CombatTracker::from_yaml(&args.combat_file);
        combat.roll_initiative(true);
        Ok(Self {
            exit: false,
            tracker: combat,
            state: TableState::default().with_selected(0),
            popup: Popup::default(),
            colors: TableColors::new(),
        })
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
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
                if let Some(selected) = self.state.selected() {
                    self.popup
                        .show("Enter damage amount:", true, move |app, damage| {
                            if let Ok(damage) = damage.parse::<i32>() {
                                if let Some(entity) = app.tracker.entities.get_mut(selected) {
                                    entity.current_hp = (entity.current_hp - damage).max(0);
                                }
                            }
                        });
                }
            }
            _text_input => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn confirm_close(&mut self) {
        self.popup.show(
            "Do you want to close the application and print the transaction to stdout?",
            false,
            |app, _| app.exit(),
        );
    }
}
