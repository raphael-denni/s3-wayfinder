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
use crossterm::{event, execute, terminal};
use ratatui::{
    Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::{Backend, CrosstermBackend},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

enum View {
    Buckets,
    Objects { bucket: String },
}

struct App {
    items: Vec<String>,
    state: ListState,
    status: String,
    view: View,
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

    let (items, status) = match crate::commands::ls::list_buckets(client).await {
        Ok(buckets) => (buckets, "Connected to S3".to_string()),
        Err(e) => (Vec::new(), format!("Error: {}", e)),
    };

    let mut app = App {
        items,
        state: ListState::default(),
        status,
        view: View::Buckets,
    };

    if !app.items.is_empty() {
        app.state.select(Some(0));
    }

    let app_result = run_app(&mut terminal_instance, &mut app, client).await;

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
async fn run_app<B: Backend>(
    terminal_instance: &mut Terminal<B>,
    app: &mut App,
    client: &Client,
) -> io::Result<()> {
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

            let title_block = Block::default()
                .borders(Borders::ALL)
                .title(" S3 Wayfinder ");
            //let main_block = Block::default().borders(Borders::ALL).title(" Content ");
            let status_block = Block::default().borders(Borders::ALL).title(" Status ");
            let status_paragraph = Paragraph::new(app.status.as_str()).block(status_block);

            let items: Vec<ListItem> = app
                .items
                .iter()
                .map(|name| ListItem::new(name.as_str()))
                .collect();

            let list_title = match &app.view {
                View::Buckets => " Buckets ".to_string(),
                View::Objects { bucket } => format!(" Objects in {} ", bucket),
            };

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(list_title))
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
                                if i >= app.items.len() - 1 {
                                    0
                                } else if i >= app.items.len() - 1 {
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
                                if app.items.is_empty() {
                                    0
                                } else if i == 0 {
                                    app.items.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        app.state.select(Some(i));
                    }

                    event::KeyCode::Enter => {
                        if let Some(i) = app.state.selected() {
                            if app.items.is_empty() {
                                continue;
                            }

                            let selected = app.items[i].clone();

                            match &app.view {
                                View::Buckets => {
                                    app.status = format!("Loading objects for {}...", selected);

                                    match crate::commands::ls::list_objects(client, &selected).await
                                    {
                                        Ok(objects) => {
                                            app.items = objects;
                                            app.view = View::Objects {
                                                bucket: selected.clone(),
                                            };
                                            app.state.select(if app.items.is_empty() {
                                                None
                                            } else {
                                                Some(0)
                                            });
                                            app.status = format!("Viewing bucket: {}", selected);
                                        }

                                        Err(e) => app.status = format!("Error: {}", e),
                                    }
                                }

                                View::Objects { bucket } => {
                                    app.status =
                                        format!("Selected object {} in {}", selected, bucket);
                                }
                            }
                        }
                    }

                    event::KeyCode::Esc | event::KeyCode::Backspace => {
                        if let View::Objects { .. } = app.view {
                            app.status = "Loading buckets...".to_string();

                            match crate::commands::ls::list_buckets(client).await {
                                Ok(buckets) => {
                                    app.items = buckets;
                                    app.view = View::Buckets;
                                    app.state.select(if app.items.is_empty() {
                                        None
                                    } else {
                                        Some(0)
                                    });
                                    app.status = "Connected to S3".to_string();
                                }

                                Err(e) => app.status = format!("Error: {}", e),
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
