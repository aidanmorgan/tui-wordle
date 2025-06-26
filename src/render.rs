use crate::game::{GameOptions, LetterResult};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::collections::HashMap;

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
        self.cell_background_colours[ls]
    }

    pub fn for_frame(g: &GameOptions, f: &Frame) -> Self {
        let area = f.area();

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
                (LetterResult::Correct, Option::Some(Color::LightGreen)),
                (LetterResult::Empty, Option::None),
                (LetterResult::Absent, Option::None),
                (LetterResult::Present, Option::Some(Color::LightYellow)),
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

pub fn draw_game(
    frame: &mut Frame,
    game_opts: &GameOptions,
    render_opts: &RenderOpts,
    words: &Vec<crate::game::Guess>,
) {
    let area = frame.area();

    // there's a minimum size we can't render below, if we are getting a cell that is zero
    // or lower, then we should just not even attempt to render.
    if render_opts.letter_cell_height <= 0 || render_opts.letter_cell_width <= 0 {
        return;
    }

    let canvas = Canvas::default()
        .background_color(render_opts.background_colour)
        .marker(Marker::Block)
        .x_bounds([0.0, area.width as f64])
        .y_bounds([0.0, area.height as f64])
        .paint(|ctx| {
            for y in 0..game_opts.max_guesses {
                let guess = &words[y as usize];

                for x in 0..game_opts.word_length {
                    let x_cell = (render_opts.grid_left_border
                        + (x * render_opts.letter_cell_width)
                        + (x * render_opts.box_spacing))
                        + (x * 2 * render_opts.grid_line_width);

                    let y_cell = (render_opts.grid_top_border
                        + (y * render_opts.letter_cell_height)
                        + (y * render_opts.box_spacing))
                        + (y * 2 * render_opts.grid_line_width);

                    let cell = &Rectangle {
                        x: x_cell as f64,
                        y: y_cell as f64,
                        width: render_opts.letter_cell_width as f64,
                        height: render_opts.letter_cell_height as f64,
                        color: render_opts.grid_colour,
                    };

                    ctx.draw(cell);

                    ctx.layer();

                    // render the grid cell
                    let letter = guess.value_at(x);
                    if letter.0.is_some() && letter.1.is_some() {
                        let colour = render_opts
                            .background_colour(&letter.1.unwrap())
                            .unwrap_or(render_opts.background_colour);

                        // this doesn't fill the cell, need to implement a version that fills the cell
                        ctx.draw(&Rectangle {
                            x: (x_cell + render_opts.grid_line_width) as f64,
                            y: (y_cell + render_opts.grid_line_width) as f64,
                            width: (render_opts.letter_cell_width
                                - (2 * render_opts.grid_line_width))
                                as f64,
                            height: (render_opts.letter_cell_height
                                - (2 * render_opts.grid_line_width))
                                as f64,
                            color: colour,
                        });
                        // fill the cell with the appropriate background colour for the letter
                    }

                    ctx.layer();

                    ctx.print(
                        (x_cell + (render_opts.letter_cell_width / 2) - 1) as f64,
                        (y_cell + (render_opts.letter_cell_height / 2) + 1) as f64,
                        String::from(letter.0.unwrap_or(' ')),
                    );
                }
            }
        });

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
        .split(frame.area());

    frame.render_widget(canvas, layout[0]);

    let p = Paragraph::new("New Game: CTRL-N, Quit: CTRL-Q | ESC, Options: CTRL-O");
    frame.render_widget(p, layout[1]);
}
