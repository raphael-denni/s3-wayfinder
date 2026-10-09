// SPDX-License-Identifier: MIT OR Apache-2.0

//! # TUI Module
//! This module provides a terminal user interface (TUI) for interacting with AWS S3.
//! It leverages the `ratatui` crate for rendering the interface and handling user input
//! in a terminal environment.
//!
//! The TUI allows users to navigate S3 buckets, view objects, and perform operations
//! such as uploading, downloading, and deleting files directly from the terminal.

use std::io;

use crate::commands::{cp, ls, rm};
use anyhow::{Context, Result};
use aws_sdk_s3::Client;
use crossterm::{event, execute, terminal};
use ratatui::{
    Frame, Terminal,
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
    confirm_delete: bool,
}

struct CurrentSelection {
    bucket: Option<String>,
    object: Option<String>,
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
pub async fn run_tui(client: &Client) -> Result<()> {
    terminal::enable_raw_mode().context("Failed to enable raw mode")?;

    let mut stdout = io::stdout();

    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture
    )
    .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal_instance =
        Terminal::new(backend).context("Failed to create terminal instance")?;

    let (items, status) = match ls::list_buckets(client).await {
        Ok(buckets) => (buckets, "Connected to S3".to_string()),
        Err(e) => (Vec::new(), format!("Error: {:#}", e)),
    };

    let mut app = App {
        items,
        state: ListState::default(),
        status,
        view: View::Buckets,
        confirm_delete: false,
    };

    if !app.items.is_empty() {
        app.state.select(Some(0));
    }

    let app_result = main_loop(&mut terminal_instance, &mut app, client).await;

    terminal::disable_raw_mode().context("Failed to disable raw mode")?;

    execute!(
        terminal_instance.backend_mut(),
        terminal::LeaveAlternateScreen,
        event::DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;

    terminal_instance
        .show_cursor()
        .context("Failed to show cursor")?;

    if let Err(e) = app_result {
        return Err(anyhow::anyhow!(e)).context("Error in TUI main loop");
    }

    Ok(())
}

// Update current frame with the latest application state.
fn update_frame(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(5),
        ])
        .split(frame.area());

    let current_endpoint = match &app.view {
        View::Buckets => "/".to_string(),
        View::Objects { bucket } => format!("/{}", bucket),
    };

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(" S3 Wayfinder ");
    let title_paragraph = Paragraph::new(current_endpoint).block(title_block);

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

    let cursor_color = match &app.confirm_delete {
        true => ratatui::style::Color::Red,
        false => ratatui::style::Color::Blue,
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_symbol(">> ")
        .highlight_style(
            ratatui::style::Style::default()
                .add_modifier(ratatui::style::Modifier::BOLD)
                .fg(cursor_color),
        );

    frame.render_widget(title_paragraph, chunks[0]);
    frame.render_stateful_widget(list, chunks[1], &mut app.state);
    frame.render_widget(status_paragraph, chunks[2]);
}

// Move the cursor up or down in the list of items.
fn move_cursor(app: &mut App, direction: i8) {
    if app.confirm_delete {
        return;
    }
    let i = match app.state.selected() {
        Some(i) => {
            if i >= app.items.len() - 1 {
                0
            } else {
                i + direction as usize
            }
        }
        None => 0,
    };
    app.state.select(Some(i));
}

// The main application loop for the TUI.
// It handles rendering and user input.
async fn main_loop<B: Backend>(
    terminal_instance: &mut Terminal<B>,
    app: &mut App,
    client: &Client,
) -> io::Result<()> {
    let mut current_selection: CurrentSelection = CurrentSelection {
        bucket: None,
        object: None,
    };

    loop {
        terminal_instance.draw(|frame| update_frame(frame, app))?;

        if !event::poll(std::time::Duration::from_millis(250))? {
            continue;
        }
        let event::Event::Key(key) = event::read()? else {
            continue;
        };

        match key.code {
            event::KeyCode::Char('q') => return Ok(()),

            event::KeyCode::Down => {
                move_cursor(app, 1);
            }

            event::KeyCode::Up => {
                move_cursor(app, -1);
            }

            event::KeyCode::Enter => {
                if app.items.is_empty() {
                    continue;
                }
                let Some(i) = app.state.selected() else {
                    continue;
                };

                let selected = app.items[i].clone();

                match &app.view {
                    View::Buckets => {
                        app.status = format!("Loading objects for {}...", selected);

                        match ls::list_objects(client, &selected).await {
                            Ok(objects) => {
                                app.status = format!("Viewing bucket: {}", selected);

                                app.items = objects;
                                app.state
                                    .select(if app.items.is_empty() { None } else { Some(0) });

                                app.view = View::Objects { bucket: selected };
                            }

                            Err(e) => app.status = format!("Error: {:#}", e),
                        }
                    }

                    View::Objects { bucket } => {
                        app.status = format!("Selected object {} in {}", selected, bucket);
                    }
                }
            }

            event::KeyCode::Esc | event::KeyCode::Backspace => {
                if app.confirm_delete {
                    app.confirm_delete = false;
                    app.status = "Delete operation cancelled.".to_string();
                    continue;
                }

                match &app.view {
                    View::Buckets => {}
                    View::Objects { .. } => {
                        app.status = "Loading buckets...".to_string();

                        match ls::list_buckets(client).await {
                            Ok(buckets) => {
                                app.items = buckets;
                                app.view = View::Buckets;
                                app.state
                                    .select(if app.items.is_empty() { None } else { Some(0) });
                                app.status = "Connected to S3".to_string();
                            }

                            Err(e) => app.status = format!("Error: {:#}", e),
                        }
                    }
                }
            }

            event::KeyCode::Char('r') => match &app.view {
                View::Buckets => {
                    app.status = "Refreshing buckets...".to_string();

                    match ls::list_buckets(client).await {
                        Ok(buckets) => {
                            app.items = buckets;
                            app.state
                                .select(if app.items.is_empty() { None } else { Some(0) });
                            app.status = "Connected to S3".to_string();
                        }

                        Err(e) => app.status = format!("Error: {:#}", e),
                    }
                }
                View::Objects { bucket } => {
                    app.status = format!("Refreshing objects in bucket {}...", bucket);

                    match ls::list_objects(client, bucket).await {
                        Ok(objects) => {
                            app.items = objects;
                            app.state
                                .select(if app.items.is_empty() { None } else { Some(0) });
                            app.status = format!("Viewing bucket: {}", bucket);
                        }

                        Err(e) => app.status = format!("Error: {:#}", e),
                    }
                }
            },

            event::KeyCode::Char('c') => match &app.view {
                View::Buckets => {}
                View::Objects { bucket } => {
                    let object_key = match app.state.selected() {
                        Some(i) => &app.items[i],
                        None => continue,
                    };

                    current_selection = CurrentSelection {
                        bucket: Some(bucket.clone()),
                        object: Some(object_key.clone()),
                    };
                }
            },

            event::KeyCode::Char('p') => {
                let Some(src_object) = &current_selection.object else {
                    continue;
                };

                let View::Objects {
                    bucket: dest_bucket,
                } = &app.view
                else {
                    continue;
                };

                let src_bucket: &str;

                if let Some(bucket) = &current_selection.bucket {
                    src_bucket = bucket;
                } else {
                    app.status = "No source bucket selected for copy.".to_string();
                    continue;
                }

                app.status = format!(
                    "Copying object {} from bucket {}...",
                    src_object, src_bucket
                );

                // For demonstration, we'll just copy the object to a new key with "_copy" suffix
                let dest_key = format!("{}_copy", src_object);

                match cp::copy_object(client, src_bucket, src_object, dest_bucket, &dest_key).await
                {
                    Ok(_) => {
                        app.status = format!(
                            "Copied object {} to {} in bucket {}",
                            src_object, dest_key, dest_bucket
                        );

                        // Refresh the object list
                        match ls::list_objects(client, dest_bucket).await {
                            Ok(objects) => {
                                app.items = objects;
                                app.state
                                    .select(if app.items.is_empty() { None } else { Some(0) });
                            }

                            Err(e) => app.status = format!("Error: {:#}", e),
                        }
                    }

                    Err(e) => app.status = format!("Error: {:#}", e),
                }
            }

            event::KeyCode::Delete | event::KeyCode::Char('d') => {
                if !app.confirm_delete {
                    app.confirm_delete = true;
                    app.status =
                        "Press 'd' again to confirm deletion. Press \"Esc\" to cancel.".to_string();
                    continue;
                }

                let key = match app.state.selected() {
                    Some(i) => &app.items[i],
                    None => continue,
                };

                match &app.view {
                    View::Buckets => {
                        let bucket_name = key;
                        app.status = format!("Deleting bucket {}...", bucket_name);

                        match rm::delete_bucket(client, bucket_name).await {
                            Ok(_) => {
                                app.status = format!("Deleted bucket {}", bucket_name);

                                match ls::list_buckets(client).await {
                                    Ok(buckets) => {
                                        app.items = buckets;
                                        app.state.select(if app.items.is_empty() {
                                            None
                                        } else {
                                            Some(0)
                                        });
                                    }

                                    Err(e) => app.status = format!("Error: {:#}", e),
                                }
                            }

                            Err(e) => app.status = format!("Error: {:#}", e),
                        }

                        app.confirm_delete = false;
                    }
                    View::Objects { bucket } => {
                        let object_key = key;
                        app.status =
                            format!("Deleting object {} from bucket {}...", object_key, bucket);

                        match rm::delete_object(client, bucket, object_key).await {
                            Ok(_) => {
                                app.status =
                                    format!("Deleted object {} from bucket {}", object_key, bucket);

                                match ls::list_objects(client, bucket).await {
                                    Ok(objects) => {
                                        app.items = objects;
                                        app.state.select(if app.items.is_empty() {
                                            None
                                        } else {
                                            Some(0)
                                        });
                                    }

                                    Err(e) => app.status = format!("Error: {:#}", e),
                                }
                            }

                            Err(e) => app.status = format!("Error: {:#}", e),
                        }

                        app.confirm_delete = false;
                    }
                }
            }

            _ => {
                if app.confirm_delete {
                    app.confirm_delete = false;
                    app.status = "Delete operation cancelled.".to_string();
                }
            }
        }
    }
}
