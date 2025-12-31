//! Terminal UI display using ratatui
//!
//! Provides a visual interface for debugging tracker playback.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Row, Table},
};
use std::io::{self, Stdout};

/// Display state passed from the player
#[derive(Debug, Clone)]
pub struct DisplayState {
    pub filename: String,
    pub playing: bool,
    pub bpm: u16,
    pub speed: u16,
    pub order: u16,
    pub total_orders: u16,
    pub pattern: u8,
    pub row: u16,
    pub total_rows: u16,
    pub tick: u16,
    pub num_channels: u8,
    pub verbose: bool,
    pub channel_mutes: [bool; 64],
}

/// Terminal display handler
pub struct Display {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Display {
    /// Create a new display
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Render the display with the current state
    pub fn render(&mut self, state: &DisplayState) -> io::Result<()> {
        self.terminal.draw(|frame| {
            let area = frame.area();

            // Layout: header, position, channel info (placeholder), help
            let layout = Layout::vertical([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Position
                Constraint::Min(5),    // Channel table (expandable)
                Constraint::Length(3), // Help
            ])
            .split(area);

            // Header
            let status = if state.playing { "PLAYING" } else { "PAUSED" };
            let mode = if state.verbose { "TICK" } else { "ROW" };
            let header_text = format!(
                "{} [{}] {} BPM  Speed: {}  Mode: {}",
                state.filename, status, state.bpm, state.speed, mode
            );
            let header = Paragraph::new(header_text)
                .block(Block::default().borders(Borders::ALL).title("tracker-debug"));
            frame.render_widget(header, layout[0]);

            // Position bar
            let position_text = format!(
                "Order: {:02}/{:02}  Pattern: {:02}  Row: {:02}/{:02}  Tick: {}/{}",
                state.order,
                state.total_orders,
                state.pattern,
                state.row,
                state.total_rows,
                state.tick,
                state.speed
            );
            let position =
                Paragraph::new(position_text).block(Block::default().borders(Borders::ALL));
            frame.render_widget(position, layout[1]);

            // Channel table (simplified - shows channel status)
            let channel_rows: Vec<Row> = (0..state.num_channels.min(16) as usize)
                .map(|i| {
                    let muted = if state.channel_mutes[i] { "M" } else { " " };
                    Row::new([format!("{:02}", i), muted.to_string(), "---".to_string()])
                })
                .collect();

            let table = Table::new(
                channel_rows,
                [
                    Constraint::Length(4),
                    Constraint::Length(3),
                    Constraint::Min(10),
                ],
            )
            .header(Row::new(["CH", "M", "Status"]).style(Style::default().bold()))
            .block(Block::default().borders(Borders::ALL).title("Channels"));
            frame.render_widget(table, layout[2]);

            // Help bar
            let help_text =
                "[Space] Pause  [←/→] Row  [↑/↓] Pattern  [+/-] Tempo  [1-9] Mute  [V] Verbose  [Q] Quit";
            let help = Paragraph::new(help_text).block(Block::default().borders(Borders::ALL));
            frame.render_widget(help, layout[3]);
        })?;

        Ok(())
    }
}
