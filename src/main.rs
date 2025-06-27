mod dictionary;
mod game;
mod game_screen;
mod options_screen;
mod options;

use crate::game::{GameData, GameOptions};
use crate::options_screen::{draw_options};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::error;
use std::error::Error;
use std::fmt::{Debug};
use thiserror::Error;
use crate::options::OptionData;

/// Entry point for the Wordle TUI application
///
/// Initializes the game with default options, sets up the terminal,
/// runs the main application loop, and restores the terminal state on exit.
fn main() {
    // Initialize game with default options
    let config = GameOptions::default();
    let mut wordle = Application::new(config);
    wordle.new_game();

    // Set up terminal
    let terminal = ratatui::init();

    // Run main loop and handle any errors
    if let Err(e) = main_loop(&mut wordle, terminal) {
        eprintln!("Error in main loop: {}", e);
    }

    // Restore terminal state
    ratatui::restore();
}



#[derive(Debug, Error)]
pub enum WordleError {
    #[error("No active game")]
    NoActiveGame,
    #[error("Rendering error: {0}")]
    RenderingError(Box<dyn Error>),
    #[error("No active options state")]
    NoActiveOptions,
}

/// Represents the current screen being displayed in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenMode {
    /// Main game screen where the player guesses words
    Game,
    /// Options screen for configuring game settings
    Options,
    /// Exit the application
    Quit,
}

/// Main application state container
#[derive(Debug)]
pub struct Application {
    /// Current game configuration options
    pub game_options: GameOptions,
    /// State for the options screen, if active
    pub options_state: Option<OptionData>,
    /// State for the current game, if active
    pub game_state: Option<GameData>,
    /// Current screen being displayed
    pub app_state: ScreenMode,
}

impl Application {
    /// Creates a new application with the given game options
    pub fn new(game_options: GameOptions) -> Self {
        Self {
            game_options,
            options_state: None,
            game_state: None,
            app_state: ScreenMode::Game,
        }
    }

    /// Starts a new game with the current game options
    pub fn new_game(&mut self) {
        self.game_state = Some(GameData::new(&self.game_options));
    }

    /// Switches to the options screen, initializing it if needed
    pub fn options(&mut self) {
        // Initialize options_state if it doesn't exist
        self.options_state.get_or_insert_with(OptionData::new);
        self.app_state = ScreenMode::Options;
    }

    /// Sets the application to quit
    pub fn quit(&mut self) {
        self.app_state = ScreenMode::Quit;
    }
}

/// Main application loop that handles screen transitions and error recovery
///
/// This function runs until the application is set to quit. It handles errors
/// by logging them and continuing execution to prevent the game from crashing.
pub fn main_loop(
    app: &mut Application,
    mut terminal: DefaultTerminal,
) -> Result<(), Box<dyn error::Error>> {
    loop {
        match app.app_state {
            ScreenMode::Game => {
                // Log errors but continue execution to prevent game from crashing
                if let Err(_e) = step_game(app, &mut terminal) {
//                    eprintln!("Game error: {}", e);
                }
            }
            ScreenMode::Options => {
                // Log errors but continue execution to prevent game from crashing
                if let Err(_e) = step_options(app, &mut terminal) {
//                    eprintln!("Options error: {}", e);
                }
            }
            ScreenMode::Quit => {
                return Ok(());
            }
        }
    }
}

/// Processes a single frame of the game screen
///
/// This function:
/// 1. Gets the active game state
/// 2. Renders the game screen
/// 3. Processes keyboard input for game actions
///
/// Returns an error if there's no active game or if rendering fails.
pub fn step_game(app: &mut Application, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn error::Error>> {
    // Use the ? operator with Option to handle the None case more idiomatically
    let game_state = app.game_state.as_mut().ok_or(WordleError::NoActiveGame)?;

    // Draw the game state
    terminal
        .draw(|frame| {
            game_screen::draw_game(frame, &app.game_options, &game_state)
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    // Handle keyboard input
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    game_state.submit_word()?;
                }
                KeyCode::Char(to_insert) => {
                    if key.modifiers == KeyModifiers::CONTROL {
                        // Handle control key combinations
                        match to_insert.to_ascii_uppercase() {
                            'N' => app.new_game(),
                            'O' => app.options(),
                            'Q' => app.quit(),
                            _ => {}
                        }
                        return Ok(());
                    }

                    // Add letter if it's alphabetic
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

/// Processes a single frame of the options screen
///
/// This function:
/// 1. Gets the active options state
/// 2. Renders the options screen
/// 3. Processes keyboard input for options navigation and selection
///
/// Returns an error if there's no active options state or if rendering fails.
pub fn step_options(app: &mut Application, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
    // Use the ? operator with Option to handle the None case more idiomatically
    let options_state = app.options_state.as_mut().ok_or(WordleError::NoActiveOptions)?;

    // Draw the options screen
    terminal
        .draw(|frame| {
            draw_options(frame, &options_state);                    
        })
        .map_err(|e| WordleError::RenderingError(Box::new(e)))?;

    // Handle keyboard input
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                // Apply options and return to game
                KeyCode::Enter => {
                    options_state.apply(&mut app.game_options)?;
                    app.new_game();
                    app.app_state = ScreenMode::Game;
                    return Ok(());
                }
                // Cancel and return to game
                KeyCode::Esc => {
                    app.new_game();
                    app.app_state = ScreenMode::Game;
                    return Ok(());
                }
                // Navigation keys
                KeyCode::Up => options_state.previous(),
                KeyCode::Down => options_state.next(),
                KeyCode::Left => options_state.decrement_tries(),
                KeyCode::Right => options_state.increment_tries(),
                _ => {}
            }
        }
    }

    Ok(())
}
