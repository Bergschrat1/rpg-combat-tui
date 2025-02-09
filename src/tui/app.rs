use crate::combat::tracker::CombatTracker;
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::widgets::TableState;
use tui_textarea::{Input, Key};

use super::{
    cli::Args,
    terminal,
    ui::{self, TableColors},
};

#[derive(Debug, Clone, Default)]
pub struct Popup {
    pub active: bool,
    pub prompt: String,
}

impl Popup {
    pub fn show(&mut self, prompt: &str) {
        self.active = true;
        self.prompt = prompt.to_string();
    }
    pub fn hide(&mut self) {
        self.active = false;
    }
}

#[derive(Debug)]
pub struct App {
    pub exit: bool,
    pub tracker: CombatTracker,
    pub state: TableState,
    pub popup: Popup,
    pub colors: TableColors,
}

impl App {
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
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
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
            } => self.exit(),
            _ => (),
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
            // TODO add shortcuts Up and Down to select an entity for damage/heal
            _text_input => {

                // current_field.input(text_input);
            }
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn confirm_close(&mut self) {
        self.popup
            .show("Do you want to close the application and print the transaction to stdout?")
    }
}
