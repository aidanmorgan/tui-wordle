mod dictionary;
mod game;
mod render;

use crate::game::{GameData, GameOptions};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::error;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
enum WordleError {
    NoActiveGame,
    RenderingError(Box<dyn Error>),
    GameLogicError,
}

impl Display for WordleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveGame => write!(f, "No active game"),
            Self::RenderingError(e) => write!(f, "Rendering error: {}", e),
            Self::GameLogicError => write!(f, "Game logic error"),
        }
    }
}

impl error::Error for WordleError {}

enum ScreenMode {
    Game,
    Options,
    Quit,
}

fn main() {
    let config = GameOptions::default();
    let mut wordle = Application::new(&config);
    wordle.new_game();

    let terminal = ratatui::init();
    if let Err(e) = main_loop(&mut wordle, terminal) {
        eprintln!("Error in main loop: {}", e);
    }
    ratatui::restore();
}

struct Application {
    game_options: GameOptions,
    current_game: Option<GameData>,
    app_state: ScreenMode,
}

impl Application {
    fn new(opts: &GameOptions) -> Self {
        Self {
            game_options: opts.clone(),
            current_game: None,
            app_state: ScreenMode::Game,
        }
    }

    fn new_game(&mut self) {
        self.current_game = Some(GameData::new(&self.game_options));
    }

    fn options(&mut self) {
        self.app_state = ScreenMode::Options;
    }

    fn quit(&mut self) {
        self.app_state = ScreenMode::Quit;
    }

    pub fn submit_word(&mut self) -> Result<(), WordleError> {
        let game = self
            .current_game
            .as_mut()
            .ok_or(WordleError::NoActiveGame)?;
        game.submit_word()
            .map_err(|_| WordleError::GameLogicError)?;
        Ok(())
    }

    pub fn add_letter(&mut self, letter: char) -> Result<(), WordleError> {
        let game = self
            .current_game
            .as_mut()
            .ok_or(WordleError::NoActiveGame)?;
        game.add_letter(letter)
            .map_err(|_| WordleError::GameLogicError)?;
        Ok(())
    }

    pub fn remove_letter(&mut self) -> Result<(), WordleError> {
        let game = self
            .current_game
            .as_mut()
            .ok_or(WordleError::NoActiveGame)?;
        game.delete_letter()
            .map_err(|_| WordleError::GameLogicError)?;
        Ok(())
    }
}

fn main_loop(
    app: &mut Application,
    mut terminal: DefaultTerminal,
) -> Result<(), Box<dyn error::Error>> {
    loop {
        match app.app_state {
            ScreenMode::Game => {
                if let Err(e) = step_game(app, &mut terminal) {
                    // no-op
                }
            }
            ScreenMode::Options => {
                if let Err(e) = step_options(app, &mut terminal) {
                    // no-op
                }
            }
            ScreenMode::Quit => {
                return Ok(());
            }
        }
    }
}

fn step_game(
    app: &mut Application,
    terminal: &mut DefaultTerminal,
) -> Result<(), Box<dyn error::Error>> {
    if app.current_game.is_none() {
        return Ok(());
    }

    terminal
        .draw(|frame| {
            let game = app
                .current_game
                .as_ref()
                .expect("Current game should be Some at this point");
            render::draw_game(frame, &app.game_options, &game)
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
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
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn step_options(
    app: &mut Application,
    terminal: &mut DefaultTerminal,
) -> Result<(), Box<dyn Error>> {
    terminal
        .draw(|frame| {
            // Options screen rendering would go here
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    app.new_game();
                }
                KeyCode::Esc => {
                    app.quit();
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    Ok(())
}
