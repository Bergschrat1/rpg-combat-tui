use std::{
    fs::{self},
    path::PathBuf,
    sync::Arc,
};

use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use core::combat::{entity::Condition, tracker::CombatTracker};
use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use log::{debug, info};
use ratatui::widgets::TableState;
use tokio::sync::Mutex;
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
    pub tracker: Arc<Mutex<CombatTracker>>,
    pub state: TableState,
    pub popup: Popup<'t>,
    pub colors: TableColors,
    pub output_file: PathBuf,
}

impl App<'_> {
    pub fn new_with_tracker(args: &Args, tracker: Arc<Mutex<CombatTracker>>) -> Result<Self> {
        let mut save_file = args.output.clone().unwrap_or({
            args.combat_file.with_extension(
                args.combat_file
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    + ".bkp",
            )
        });
        save_file = save_file.with_file_name(
            ".".to_string()
                + save_file
                    .file_name()
                    .wrap_err("File doesn't have a filename")?
                    .to_str()
                    .wrap_err("Something wrong with filename")?,
        );
        info!("Using {} as a save file.", &save_file.display());
        tracker.blocking_lock().roll_initiative(true, false);

        Ok(Self {
            exit: false,
            tracker,
            state: TableState::default().with_selected(0),
            popup: Popup::new(),
            colors: TableColors::new(),
            output_file: save_file,
        })
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<CombatTracker> {
        debug!("Running program.");
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(self.tracker.blocking_lock().clone())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                let res: Result<()> = if self.popup.active {
                    self.handle_popup_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                } else {
                    self.handle_key_event(key_event)
                        .wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))
                };
                self.backup()?;
                res
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
            }
            | Input {
                key: Key::Char(' '),
                ..
            } => {
                self.tracker.blocking_lock().next_turn();
            }
            Input { key: Key::Left, .. }
            | Input {
                key: Key::Backspace,
                ..
            } => {
                self.tracker.blocking_lock().prev_turn();
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
                key: Key::Char('i'),
                ..
            } => {
                self.set_initiative();
            }
            Input {
                key: Key::Char('r'),
                ctrl: true,
                ..
            } => {
                self.tracker.blocking_lock().roll_initiative(true, true);
            }
            Input {
                key: Key::Char('c'),
                ctrl: true,
                ..
            } => {
                self.tracker.blocking_lock().reset_combat();
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
                        if let Some(entity) = app.tracker.blocking_lock().entities.get_mut(selected)
                        {
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

    fn set_initiative(&mut self) {
        let prompt = "Enter Initiative:";
        if let Some(selected) = self.state.selected() {
            self.popup
                .show(prompt, true, (30, 20), move |app, input_amount| {
                    if let Ok(new_ini) = input_amount.parse::<i32>() {
                        let mut tracker = app.tracker.blocking_lock();
                        if let Some(entity) = tracker.entities.get_mut(selected) {
                            info!(
                                "Setting initiative of entity {} ({}) to {}",
                                entity.name, entity.id, &new_ini
                            );
                            entity.initiative = Some(new_ini);
                            tracker.sort_by_initiative();
                        }
                    }
                });
        };
    }

    fn change_conditions(&mut self) {
        let selected = match self.state.selected() {
            Some(s) => s,
            None => return,
        };

        let tracker = self.tracker.blocking_lock();
        let entity = match tracker.entities.get(selected) {
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
                    if let Some(entity) = app.tracker.blocking_lock().entities.get_mut(selected) {
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
        info!("Application stopped.");
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

    fn backup(&mut self) -> Result<()> {
        debug!("Writing state to file {}", &self.output_file.display());
        fs::write(&self.output_file, self.tracker.blocking_lock().to_yaml())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tui::utils::load_combat_yaml;

    use super::*;

    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_player_character_yaml() {
        let dir = temp_dir();
        let combat_file_path = dir.join("combat1.yaml");
        let mut combat_file = File::create(&combat_file_path).unwrap();
        writeln!(
            combat_file,
            r#"
monsters:
  - stats:
        name: Orc
        entity_type: Monster
        initiative_modifier: 1
        ac: 13
        max_hp: 15
        conditions: [Blinded, Grappled]
"#
        )
        .unwrap();

        // Create temp player characters file
        let players_file_path = dir.join("player.yaml");
        let mut players_file = File::create(&players_file_path).unwrap();
        writeln!(
            players_file,
            r#"
players:
  - name: Arthas
    entity_type: Player
    initiative_modifier: 30
    ac: 18
    max_hp: 45
    current_hp: 45
    conditions: []
"#
        )
        .unwrap();

        let args = Args {
            combat_file: combat_file_path,
            player_characters: Some(players_file_path),
            output: None,
            stdout: false,
        };

        let tracker = Arc::new(Mutex::new(CombatTracker::from_yaml(
            load_combat_yaml(&args).unwrap(),
        )));
        let app = App::new_with_tracker(&args, Arc::clone(&tracker)).unwrap();

        assert_eq!(app.tracker.blocking_lock().entities.len(), 2);
        assert_eq!(
            app.tracker.blocking_lock().entities.first().unwrap().name,
            "Arthas".to_string()
        );
        assert_eq!(
            app.tracker.blocking_lock().entities.last().unwrap().name,
            "Orc".to_string()
        );
    }

    #[test]
    fn test_loads_from_save_file_if_exists() {
        let dir = temp_dir();
        let combat_file_path = dir.join("combat2.yaml");

        // Create a combat file (normal)
        let mut combat_file = File::create(&combat_file_path).unwrap();
        writeln!(
            combat_file,
            r#"
players:
  - name: Frodo
    initiative_modifier: 2
    ac: 16
    max_hp: 63
    current_hp: 26
monsters:
  - stats:
        name: Orc
        entity_type: Monster
        initiative_modifier: 1
        ac: 13
        max_hp: 15
        conditions: []
"#
        )
        .unwrap();

        // Create a save file (combat.yaml.bkp)
        let mut save_file_path = combat_file_path.clone();
        if let Some(file_name) = combat_file_path.file_name().and_then(|f| f.to_str()) {
            let backup_name = format!(".{}.bkp", file_name);
            save_file_path.set_file_name(backup_name);
        };
        dbg!(&save_file_path);
        let mut save_file = File::create(&save_file_path).unwrap();
        writeln!(
            save_file,
            r#"
players:
  - name: Samwise
    initiative_modifier: 2
    ac: 16
    max_hp: 63
    current_hp: 26
monsters:
  - count: 2
    stats:
        name: Goblin
        entity_type: Monster
        initiative_modifier: 2
        ac: 12
        max_hp: 10
        conditions: []
"#
        )
        .unwrap();

        let args = Args {
            combat_file: combat_file_path,
            player_characters: None,
            output: None,
            stdout: false,
        };

        let tracker = Arc::new(Mutex::new(CombatTracker::from_yaml(
            load_combat_yaml(&args).unwrap(),
        )));
        let app = App::new_with_tracker(&args, Arc::clone(&tracker)).unwrap();

        println!("{}", app.tracker.blocking_lock().to_yaml());
        // It should load the Goblin from the .bkp file, not the Orc from combat.yaml
        assert_eq!(app.tracker.blocking_lock().entities.len(), 3);
        assert_eq!(
            app.tracker.blocking_lock().entities[0].name,
            "Goblin".to_string()
        );
    }
}
