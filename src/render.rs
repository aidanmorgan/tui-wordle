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

pub struct RenderOpts {
    grid_line_width: u16, // the width of the line to draw (in pixels)

    grid_left_border: u16, // the offset of the start of the grid from the left edge of the screen
    grid_right_border: u16, // the offset of the end of the grid from the right edge of the screen

    grid_top_border: u16, // the offset of the start of the grid from the top edge of the screen
    grid_bottom_border: u16, // the offset of the end of the grid from the bottom edge of the screen

    pub letter_cell_width: u16, // the width of the cell that a letter is within
    pub letter_cell_height: u16, // the height of the cell that a letter is within

    pub box_spacing: u16,

    pub background_colour: Color,
    pub grid_colour: Color,

    cell_background_colours: HashMap<LetterResult, Option<Color>>,
}
impl RenderOpts {
    pub fn background_colour(&self, ls: &LetterResult) -> Option<Color> {
        self.cell_background_colours
            .get(ls)
            .copied()
            .unwrap_or(None)
    }

    pub fn for_rect(g: &GameOptions, area: &Rect) -> Self {
        let mut r = RenderOpts {
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

        // if the window is taller than it is wide, then we're constrained by width, so we need to compute the sizing
        // to fit the width appropriately
        if area.height > area.width {
            // try and determine the number of cells we can use for rendering the cell, taking out all other
            // values such as borders, grid lines, padding between the boxes etc.
            let temp = (area.width
                - ((g.word_length * 2 * r.grid_line_width) + ((g.word_length - 1) * r.box_spacing))
                + r.grid_left_border
                + r.grid_right_border) as f32;

            r.letter_cell_width = (temp / (g.word_length) as f32) as u16;
            r.letter_cell_height = r.letter_cell_width;
        } else {
            let temp = (area.height
                - ((g.max_guesses * 2 * r.grid_line_width) + ((g.max_guesses - 1) * r.box_spacing))
                + r.grid_top_border
                + r.grid_bottom_border) as f32;

            r.letter_cell_height = (temp / (g.max_guesses) as f32) as u16;
            r.letter_cell_width = r.letter_cell_height;
        }

        r
    }
}

pub fn draw_game(frame: &mut Frame, game_opts: &GameOptions, game_data: &GameData) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(98), Constraint::Percentage(2)])
        .split(frame.area());

    let content_panel = layout[0];

    let p = Block::default()
        .title(Line::from("New Game: CTRL-N, Quit: CTRL-Q | ESC, Options: CTRL-O").left_aligned())
        .title(Line::from(format!("{}", game_opts.dictionary)).right_aligned());
    frame.render_widget(p, layout[1]);

    match game_data.game_state {
        GameState::Won => {
            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Full)
                    .style(Style::new())
                    .lines(vec![Line::from("You Won!".green())])
                    .centered()
                    .build(),
                content_panel,
            );
        }
        GameState::Lost => {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_panel);

            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Full)
                    .lines(vec![Line::from("You Lost!".red())])
                    .centered()
                    .build(),
                layout[0],
            );

            frame.render_widget(
                BigText::builder()
                    .pixel_size(PixelSize::Quadrant)
                    .lines(vec![Line::from(
                        format!("The word was {}", game_data.answer).white(),
                    )])
                    .centered()
                    .build(),
                layout[1],
            );
        }
        _ => {
            let render_opts = RenderOpts::for_rect(game_opts, &content_panel);
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
                    for y in 0..game_opts.max_guesses {
                        // Flip the index so that the first guess is at the top
                        let guess =
                            &game_data.guesses[(game_opts.max_guesses - y - 1) as usize].values();

                        for x in 0..game_opts.word_length {
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
