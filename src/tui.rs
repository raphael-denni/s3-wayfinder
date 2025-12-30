// SPDX-License-Identifier: MIT OR Apache-2.0

//! # TUI Module
//! This module provides a terminal user interface (TUI) for interacting with AWS S3.
//! It leverages the `ratatui` crate for rendering the interface and handling user input
//! in a terminal environment.
//!
//! The TUI allows users to navigate S3 buckets, view objects, and perform operations
//! such as uploading, downloading, and deleting files directly from the terminal.

use std::io;

use aws_sdk_s3::Client;
use aws_sdk_s3::error::ProvideErrorMetadata;
use crossterm::{event, execute, terminal};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::{Backend, CrosstermBackend},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

struct App {
    buckets: Vec<String>,
    state: ListState,
    status: String,
}

/// # TUI Application Entry Point
/// This function initializes and runs the terminal user interface (TUI)
///
/// # Arguments
/// * `client` - An AWS S3 client instance for interacting with S3 services.
///
/// # Returns
/// A Result indicating success or failure of the TUI application.
///
/// # Errors
/// This function may return an error if terminal initialization or TUI execution fails.
pub async fn run(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    terminal::enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal_instance = Terminal::new(backend).unwrap();

    let response = client.list_buckets().send().await;
    let (buckets, status) = match response {
        Ok(output) => {
            let b = output
                .buckets
                .unwrap_or_default()
                .iter()
                .map(|b| b.name.clone().unwrap_or_default())
                .collect();
            (b, "Connected to S3".to_string())
        }
        Err(e) => {
            let err_msg = e.into_service_error();
            let msg = if err_msg.code() == Some("InvalidAccessKeyId") {
                "Error: Invalid Access Key ID. Check config.toml".to_string()
            } else if err_msg.code() == Some("SignatureDoesNotMatch") {
                "Error: Invalid Secret Key. Check config.toml".to_string()
            } else {
                format!("S3 Error: {}", err_msg.message().unwrap_or("Unknown error"))
            };
            (Vec::new(), msg)
        }
    };

    let mut app = App {
        buckets,
        state: ListState::default(),
        status,
    };

    if !app.buckets.is_empty() {
        app.state.select(Some(0));
    }

    let app_result = run_app(&mut terminal_instance, &mut app);

    terminal::disable_raw_mode()?;

    execute!(
        terminal_instance.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )?;

    terminal_instance.show_cursor()?;

    if let Err(e) = app_result {
        eprintln!("Error in app: {:?}", e);
        return Err(Box::new(e));
    }

    Ok(())
}

// The main application loop for the TUI.
// It handles rendering and user input.
fn run_app<B: Backend>(terminal_instance: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal_instance.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(5),
                ])
                .split(frame.area());

            let title_block = Block::default().borders(Borders::ALL).title(" Sherpa S3 ");
            //let main_block = Block::default().borders(Borders::ALL).title(" Content ");
            let status_block = Block::default().borders(Borders::ALL).title(" Status ");
            let status_paragraph = Paragraph::new(app.status.as_str()).block(status_block);

            let items: Vec<ListItem> = app
                .buckets
                .iter()
                .map(|name| ListItem::new(name.as_str()))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(" Buckets "))
                .highlight_symbol(">> ")
                .highlight_style(
                    ratatui::style::Style::default()
                        .add_modifier(ratatui::style::Modifier::BOLD)
                        .fg(ratatui::style::Color::Yellow),
                );

            frame.render_widget(title_block, chunks[0]);
            frame.render_stateful_widget(list, chunks[1], &mut app.state);
            frame.render_widget(status_paragraph, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    event::KeyCode::Char('q') => return Ok(()),

                    event::KeyCode::Down => {
                        let i = match app.state.selected() {
                            Some(i) => {
                                if i >= app.buckets.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        app.state.select(Some(i));
                    }
                    event::KeyCode::Up => {
                        let i = match app.state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    app.buckets.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }
}
