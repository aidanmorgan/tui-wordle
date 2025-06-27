use crate::game::{GameData, GameOptions, GameState, LetterResult};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::widgets::Block;
use ratatui::Frame;
use std::collections::HashMap;
use tui_big_text::{BigText, PixelSize};

/// Configuration options for rendering the game
#[derive(Debug)]
pub struct RenderOpts {
    /// The width of the grid lines in pixels
    grid_line_width: u16,

    /// The offset of the start of the grid from the left edge of the screen
    grid_left_border: u16,
    /// The offset of the end of the grid from the right edge of the screen
    grid_right_border: u16,

    /// The offset of the start of the grid from the top edge of the screen
    grid_top_border: u16,
    /// The offset of the end of the grid from the bottom edge of the screen
    grid_bottom_border: u16,

    /// The width of each letter cell
    pub letter_cell_width: u16,
    /// The height of each letter cell
    pub letter_cell_height: u16,

    /// The spacing between letter boxes
    pub box_spacing: u16,

    /// The background color of the game screen
    pub background_colour: Color,
    /// The color of the grid lines
    pub grid_colour: Color,

    /// Mapping of letter results to background colors
    cell_background_colours: HashMap<LetterResult, Option<Color>>,
}
impl RenderOpts {
    /// Gets the background color for a letter result
    ///
    /// # Arguments
    /// * `letter_result` - The letter result to get the background color for
    ///
    /// # Returns
    /// The background color for the letter result, or None if no color is defined
    pub fn background_colour(&self, letter_result: &LetterResult) -> Option<Color> {
        self.cell_background_colours
            .get(letter_result)
            .copied()
            .flatten() // Simplify by flattening Option<Option<Color>> to Option<Color>
    }

    /// Creates a new RenderOpts instance sized for the given area
    ///
    /// # Arguments
    /// * `game_options` - The game options to use for sizing
    /// * `area` - The area to size the render options for
    ///
    /// # Returns
    /// A new RenderOpts instance sized for the given area
    pub fn for_rect(game_options: &GameOptions, area: &Rect) -> Self {
        let mut render_opts = RenderOpts {
            background_colour: Color::Rgb(255, 255, 255),
            grid_colour: Color::Rgb(211, 214, 218),
            grid_bottom_border: 1,
            grid_top_border: 1,
            grid_left_border: 1,
            grid_right_border: 1,

            grid_line_width: 1,
            box_spacing: 2,

            letter_cell_height: 6,
            letter_cell_width: 6,

            cell_background_colours: HashMap::from([
                (LetterResult::Correct, Some(Color::LightGreen)),
                (LetterResult::Empty, None),
                (LetterResult::Absent, None),
                (LetterResult::Present, Some(Color::LightYellow)),
            ]),
        };

        // Always calculate the optimal cell size based on available space
        // Calculate available width and height for cells
        let available_width = (area.width.saturating_sub(
            render_opts.grid_left_border + 
            render_opts.grid_right_border + 
            (game_options.word_length - 1) * render_opts.box_spacing +
            game_options.word_length * 2 * render_opts.grid_line_width
        )) as f32;

        let available_height = (area.height.saturating_sub(
            render_opts.grid_top_border + 
            render_opts.grid_bottom_border + 
            (game_options.max_guesses - 1) * render_opts.box_spacing +
            game_options.max_guesses * 2 * render_opts.grid_line_width
        )) as f32;

        // Calculate cell width and height based on available space
        let cell_width = (available_width / game_options.word_length as f32).max(1.0) as u16;
        let cell_height = (available_height / game_options.max_guesses as f32).max(1.0) as u16;

        // Use the smaller dimension to keep cells square
        let cell_size = cell_width.min(cell_height);

        render_opts.letter_cell_width = cell_size;
        render_opts.letter_cell_height = cell_size;

        render_opts
    }
}

/// Draws the game screen
///
/// # Arguments
/// * `frame` - The frame to draw on
/// * `game_options` - The game options
/// * `game_data` - The game data
pub fn draw_game(frame: &mut Frame, game_options: &GameOptions, game_data: &GameData) {
    // Split the screen into a content area and a status bar
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(98), Constraint::Percentage(2)])
        .split(frame.area());

    let content_panel = layout[0];
    let status_bar_panel = layout[1];

    // Create the status bar with controls and dictionary info
    let status_bar = Block::default()
        .title(Line::from("New Game: CTRL-N, Quit: CTRL-Q | ESC, Options: CTRL-O").left_aligned())
        .title(Line::from(format!("{}", game_options.dictionary)).right_aligned());

    // Render the status bar
    frame.render_widget(status_bar, status_bar_panel);

    match game_data.game_state {
        GameState::Won => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Percentage(30), Constraint::Fill(1)])
                .split(content_panel);

            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Full)
                    .style(Style::new())
                    .lines(vec![Line::from("You Won!".green())])
                    .centered()
                    .build(),
                layout[1],
            );
        }
        GameState::Lost => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Fill(1), Constraint::Percentage(30), Constraint::Percentage(30), Constraint::Fill(1)])
                .split(content_panel);

            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Full)
                    .lines(vec![Line::from("You Lost!".red())])
                    .centered()
                    .build(),
                layout[1],
            );

            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Quadrant)
                    .lines(vec![Line::from(
                        format!("The word was {}", game_data.answer).white(),
                    )])
                    .centered()
                    .build(),
                layout[2],
            );
        }
        _ => {
            let render_opts = RenderOpts::for_rect(game_options, &content_panel);
            // there's a minimum size we can't render below, if we are getting a cell that is zero
            // or lower, then we should just not even attempt to render.
            if render_opts.letter_cell_height <= 0 || render_opts.letter_cell_width <= 0 {
                return;
            }

            let canvas = Canvas::default()
                .background_color(render_opts.background_colour)
                .marker(Marker::Block)
                .x_bounds([0.0, content_panel.width as f64])
                .y_bounds([0.0, content_panel.height as f64])
                .paint(|ctx| {
                    for y in 0..game_options.max_guesses {
                        // Flip the index so that the first guess is at the top
                        let guess =
                            &game_data.guesses[(game_options.max_guesses - y - 1) as usize].values();

                        for x in 0..game_options.word_length {
                            let letter = &guess[x as usize];

                            let x_cell = render_opts.grid_left_border
                                + (x * render_opts.letter_cell_width)
                                + (x * render_opts.box_spacing)
                                + (x * 2 * render_opts.grid_line_width);

                            let y_cell = render_opts.grid_top_border
                                + (y * render_opts.letter_cell_height)
                                + (y * render_opts.box_spacing)
                                + (y * 2 * render_opts.grid_line_width);

                            let mut colour = render_opts.grid_colour;

                            if let (Some(_), Some(lr)) = (letter.0, &letter.1) {
                                // if there is a result provided then check that we might want to change
                                // the cell background colour
                                colour = render_opts.background_colour(&lr).unwrap_or(colour);
                            }

                            let cell = &Rectangle {
                                x: x_cell as f64,
                                y: y_cell as f64,
                                width: render_opts.letter_cell_width as f64,
                                height: render_opts.letter_cell_height as f64,
                                color: colour,
                            };

                            ctx.draw(cell);

                            ctx.print(
                                (x_cell + (render_opts.letter_cell_width / 2) - 1) as f64,
                                (y_cell + (render_opts.letter_cell_height / 2) + 1) as f64,
                                String::from(letter.0.unwrap_or(' ')),
                            );
                        }
                    }
                });

            frame.render_widget(canvas, content_panel);
        }
    }
}
