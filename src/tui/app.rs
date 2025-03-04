use crate::combat::{entity::Condition, tracker::CombatTracker};
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
    pub size: (u16, u16),
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
            size: (30, 20),
        }
    }
    pub fn show<F: FnMut(&mut App<'t>, String) + Send + 'static>(
        &mut self,
        prompt: &str,
        show_input: bool,
        size: (u16, u16),
        action: F,
    ) {
        // clear the input
        self.input.select_all();
        self.input.cut();

        self.active = true;
        self.prompt = prompt.to_string();
        self.show_input = show_input;
        self.confirm_action = Some(Box::new(action));
        self.size = size;
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

impl App<'_> {
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
            Input {
                key: Key::Char('c'),
                ..
            } => {
                self.change_conditions();
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
            self.popup
                .show(prompt, true, (30, 20), move |app, input_amount| {
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

    fn change_conditions(&mut self) {
        let selected = match self.state.selected() {
            Some(s) => s,
            None => return,
        };

        let entity = match self.tracker.entities.get(selected) {
            Some(e) => e,
            None => return,
        };

        let all_conditions: Vec<Condition> = vec![
            Condition::Blinded,
            Condition::Charmed,
            Condition::Deafened,
            Condition::Frightened,
            Condition::Grappled,
            Condition::Incapacitated,
            Condition::Invisible,
            Condition::Paralyzed,
            Condition::Petrified,
            Condition::Poisoned,
            Condition::Prone,
            Condition::Restrained,
            Condition::Stunned,
            Condition::Unconscious,
        ];

        let prompt_text = all_conditions
            .iter()
            .enumerate()
            .map(|(i, condition)| {
                let marker = if entity.conditions.contains(condition) {
                    "[X]"
                } else {
                    "[ ]"
                };
                format!("{} {} - {}", marker, i + 1, condition)
            })
            .collect::<Vec<String>>()
            .join("\n");

        self.popup.show(
            &format!("Select condition to toggle:\n\n{}", prompt_text),
            true,
            (10, 30),
            move |app, input| {
                if let Ok(index) = input.parse::<usize>() {
                    if let Some(entity) = app.tracker.entities.get_mut(selected) {
                        if index > 0 && index <= all_conditions.len() {
                            let condition = &all_conditions[index - 1];
                            if entity.conditions.contains(condition) {
                                entity.conditions.remove(condition);
                            } else {
                                entity.conditions.insert(condition.clone());
                            }
                        }
                    }
                }
            },
        );
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn confirm_close(&mut self) {
        self.popup.show(
            "Do you want to close the application?",
            false,
            (30, 20),
            |app, _| app.exit(),
        );
    }
}
