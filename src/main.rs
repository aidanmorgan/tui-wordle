mod dictionary;
mod game;
mod render;
mod options;

use crate::game::{GameData, GameOptions};
use crate::options::{draw_options, OptionData};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::error;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};


fn main() {
    let config = GameOptions::default();
    let mut wordle = Application::new(config);
    wordle.new_game();

    let terminal = ratatui::init();
    if let Err(e) = main_loop(&mut wordle, terminal) {
        eprintln!("Error in main loop: {}", e);
    }
    ratatui::restore();
}



#[derive(Debug)]
enum WordleError {
    NoActiveGame,
    RenderingError(Box<dyn Error>),
    NoActiveOptions,
}

impl Display for WordleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoActiveGame => write!(f, "No active game"),
            Self::RenderingError(e) => write!(f, "Rendering error: {}", e),
            Self::NoActiveOptions => write!(f, "No active options state.")
        }
    }
}

impl error::Error for WordleError {}

enum ScreenMode {
    Game,
    Options,
    Quit,
}

struct Application {
    game_options: GameOptions,
    options_state: Option<OptionData>,
    game_state: Option<GameData>,
    app_state: ScreenMode,
}

impl Application {
    fn new(game_options: GameOptions) -> Self {
        Self {
            game_options,
            options_state: Some(OptionData::new()),
            game_state: None,
            app_state: ScreenMode::Game,
        }
    }

    fn new_game(&mut self) {
        self.game_state = Some(GameData::new(&self.game_options));
    }

    fn options(&mut self) {
        self.app_state = ScreenMode::Options;
    }

    fn quit(&mut self) {
        self.app_state = ScreenMode::Quit;
    }
}

fn main_loop(
    app: &mut Application,
    mut terminal: DefaultTerminal,
) -> Result<(), Box<dyn error::Error>> {
    loop {
        match app.app_state {
            ScreenMode::Game => {
                // Errors in step_game are handled internally, no need to propagate
                let _ = step_game(app, &mut terminal);
            }
            ScreenMode::Options => {
                // Errors in step_options are handled internally, no need to propagate
                let _ = step_options(app, &mut terminal);
            }
            ScreenMode::Quit => {
                return Ok(());
            }
        }
    }
}

fn step_game(app: &mut Application, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
    let game_state = match &mut app.game_state {
        Some(state) => state,
        None => return Err(Box::new(WordleError::NoActiveGame)),
    };

    terminal
        .draw(|frame| {
            render::draw_game(frame, &app.game_options, &game_state)
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    game_state.submit_word()?;
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

                    if to_insert.is_alphabetic() {
                        game_state.add_letter(to_insert)?;
                    }
                }
                KeyCode::Backspace => {
                    game_state.delete_letter()?;
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

fn step_options(app: &mut Application, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    let options_state = match &mut app.options_state {
        Some(state) => state,
        None => return Err(Box::new(WordleError::NoActiveOptions)),
    };

    terminal
        .draw(|frame| {
            draw_options(frame, &options_state);                    
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    options_state.apply(&mut app.game_options)?;
                    app.new_game();
                    app.app_state = ScreenMode::Game;

                    return Ok(());
                }
                KeyCode::Esc => {
                    app.new_game();
                    app.app_state = ScreenMode::Game;

                    return Ok(());
                }
                KeyCode::Up => {
                    options_state.previous();
                }
                KeyCode::Down => {
                    options_state.next();
                }
                KeyCode::Left => {
                    options_state.decrement_tries();
                }
                KeyCode::Right => {
                    options_state.increment_tries();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
