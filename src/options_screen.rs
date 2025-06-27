use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::Frame;
use tui_big_text::{BigText, PixelSize};
use crate::options::OptionData;

/// Draws the options screen
///
/// # Arguments
/// * `frame` - The frame to draw on
/// * `options_data` - The options data to display
pub fn draw_options(frame: &mut Frame, options_data: &OptionData) {
    // Split the screen into sections for different UI elements
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Fill(1),       // Top spacing
            Constraint::Max(10),       // Dictionary display
            Constraint::Max(10),       // Guesses display
            Constraint::Fill(1),       // Middle spacing
            Constraint::Max(5)         // Controls bar
        ])
        .split(frame.area());

    // Render the dictionary selection
    frame.render_widget(
        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![Line::from(format!(
                "{} - {} Letters", 
                options_data.dictionary_name, 
                options_data.dictionary_length
            ))])
            .centered()
            .build(),
        layout[1]
    );

    // Render the guesses count
    frame.render_widget(
        BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .lines(vec![Line::from(format!(
                "Guesses: {}", 
                options_data.max_tries
            ))])
            .centered()
            .build(),
        layout[2]
    );

    // Render the controls bar
    let controls_bar = Block::default()
        .title(Line::from(
            "Select: Enter, Cancel: ESC, Dictionary: Up/Down, Guesses: Left/Right, Quit: CTRL-Q"
        ).left_aligned());

    frame.render_widget(controls_bar, layout[4]);
}
