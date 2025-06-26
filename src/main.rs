mod game;
mod render;

use std::error;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::game::{GameData, GameOptions};
use crate::render::RenderOpts;
use crate::WordleError::NoActiveGame;

#[derive(Debug)]
enum WordleError {
    NoActiveGame,
    RenderingError(Box<dyn Error>),
}

impl Display for WordleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for WordleError {}

enum ScreenMode {
    Game,
    Options,
    Quit,
}


fn main() {
    let config = &GameOptions::default();
    let mut wordle = Application::new(&config);
    wordle.new_game();

    let terminal = ratatui::init();
    run(&mut wordle, terminal);
    ratatui::restore();
}

struct Application {
    game_options: GameOptions,
    current_game: Option<GameData>,
    app_state: ScreenMode,
}

impl Application {
    fn new(opts: &GameOptions) -> Self {
        Application {
            game_options: opts.clone(),
            current_game: Option::None,
            app_state: ScreenMode::Game,
        }
    }

    fn new_game(&mut self) {
        if self.current_game.is_none() {
            self.current_game = Option::Some(GameData::new(&self.game_options));
        }
    }

    fn options(&mut self) {
        self.app_state = ScreenMode::Options;
    }

    fn quit(&mut self) {
        self.app_state = ScreenMode::Quit;
    }

    pub fn submit_word(&mut self) -> Result<(), WordleError> {
        if self.current_game.is_none() {
            return Err(WordleError::NoActiveGame);
        }

        self.current_game.as_mut().unwrap().submit_word();
        Ok(())
    }

    pub fn add_letter(&mut self, letter:char) -> Result<(), Box<dyn error::Error>> {
        if self.current_game.is_none() {
            return Err(Box::new(NoActiveGame));
        }

        self.current_game.as_mut().unwrap().add_letter(letter);
        Ok(())
    }

    pub fn remove_letter(&mut self) -> Result<(), WordleError> {
        if self.current_game.is_none() {
            return Err(NoActiveGame);
        }

        self.current_game.as_mut().unwrap().delete_letter();
        Ok(())
    }


}

fn run(app: &mut Application, mut terminal: DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
    loop {
        match (app.app_state) {
            ScreenMode::Game => {
                run_game(app, &mut terminal)?;
            }
            ScreenMode::Options => {
                run_options(app, &mut terminal)?;
            }
            ScreenMode::Quit => {
                return Ok(());
            }
        }
    }
}

fn run_game(app: &mut Application, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
    if let Event::Key(key) = event::read()? {
        if (key.kind == KeyEventKind::Press) {
            match key.code {
                KeyCode::Enter => {
                    app.submit_word()?;
                }
                KeyCode::Char(to_insert) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        match to_insert.to_ascii_uppercase() {
                            'N' => app.new_game(),
                            'O' => app.options(),
                            'Q' => app.quit(),
                            _ => {}
                        }

                        return Ok(());
                    }

                    app.add_letter(to_insert)?;
                }
                KeyCode::Backspace => {
                    app.remove_letter()?;
                }
                KeyCode::Esc => {
                    app.quit();
                    return Ok(())
                }
                _ => {}
            }
        }
    }

    if (app.current_game.is_none()) {
        return Ok(());
    }

    let frame_result = terminal.draw(|frame| {
        let game = app.current_game.as_ref().unwrap();

        render::draw_game(
            frame,
            &app.game_options,
            &RenderOpts::for_frame(&app.game_options, frame),
            &game.guesses,
        )
    });

    if frame_result.is_err() {
        return Err(frame_result.err().unwrap().into());
    }

    Ok(())
}

fn run_options(
    app: &mut Application,
    terminal: &mut DefaultTerminal,
) -> Result<(), Box<WordleError>> {
    Ok(())
}
