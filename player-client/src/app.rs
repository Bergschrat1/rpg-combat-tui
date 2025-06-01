use crate::ui;
use crate::ui::TableColors;
use color_eyre::{
    eyre::{Context, ContextCompat},
    Result,
};
use log::debug;

pub struct App {
    pub exit: bool,
    pub colors: TableColors,
    pub state: PlayerState,
}

impl App {
    pub fn new(args: &Args) -> Result<Self> {
        Ok(Self {
            exit: false,
            colors: TableColors::new(),
        })
    }

    pub fn run(&mut self, terminal: &mut terminal::Tui) -> Result<()> {
        debug!("Running program.");
        while !self.exit {
            terminal.draw(|frame| ui::draw(frame, self).expect("Couldn't draw ui!"))?;
        }
        Ok(())
    }
}
