use crate::dictionary::{get_dictionaries, Dictionary};
use crate::game::GameOptions;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::Frame;
use std::sync::Arc;
use tui_big_text::{BigText, PixelSize};

#[derive(Debug)]
pub struct OptionData {
    dictionary_name: String,
    dictionary_length: u8,
    max_tries: u16,
    dictionaries: Vec<Arc<Dictionary>>,
}

impl OptionData {
    pub fn new() -> Self {
        Self {
            dictionary_name: String::from("Wordle"),
            dictionary_length: 5,
            max_tries: 6,
            dictionaries: get_dictionaries()
        }
    }
    pub fn next(&mut self) {
        let idx = self.dictionaries
            .iter()
            .position(|x| x.name == self.dictionary_name && self.dictionary_length == x.length)
            .expect("Current dictionary not found");
        let next = (idx + 1) % self.dictionaries.len();

        let dict = &self.dictionaries[next];
        self.dictionary_name = dict.name.clone();
        self.dictionary_length = dict.length;
    }

    pub fn previous(&mut self) {
        let mut idx = self.dictionaries
            .iter()
            .position(|x| x.name == self.dictionary_name && self.dictionary_length == x.length)
            .expect("Current dictionary not found");

        if idx == 0 {
            idx = self.dictionaries.len() - 1;
        } else {
            idx -= 1;
        }

        let dict = &self.dictionaries[idx];
        self.dictionary_name = dict.name.clone();
        self.dictionary_length = dict.length;
    }

    pub fn apply(&self, opts: &mut GameOptions) -> Result<(), Box<dyn std::error::Error>> {
        opts.set_dictionary(&self.dictionary_name, self.dictionary_length)?;
        opts.max_guesses = self.max_tries;
        
        Ok(())
    }

    pub fn increment_tries(&mut self) {
        self.max_tries += 1;
        self.max_tries = self.max_tries.min(10);
    }

    pub fn decrement_tries(&mut self) {
        self.max_tries -= 1;
        self.max_tries = self.max_tries.max(3);
    }
}

pub fn draw_options(frame: &mut Frame, options_data: &OptionData) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(48), Constraint::Percentage(47), Constraint::Percentage(5)])
        .split(frame.area());

        frame.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Quadrant)
                .lines(vec![Line::from(format!("{} - {} Letters", options_data.dictionary_name, options_data.dictionary_length))])
                .centered()
                .build(),
            layout[0]
        );
    
        frame.render_widget(
            BigText::builder()
                .pixel_size(PixelSize::Quadrant)
                .lines(vec![Line::from(format!("Guesses: {}", options_data.max_tries))])
                .centered()
                .build(),
            layout[1]
        );

    let p = Block::default()
        .title(Line::from("Select: Enter, Cancel: ESC, Dictionary: Up/Down, Guesses: Left/Right, Quit: CTRL-Q").left_aligned());
    frame.render_widget(p, layout[2]);
}
