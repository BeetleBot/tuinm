mod app;
mod event;
mod nm;
mod ui;

use anyhow::Result;
use app::{App, FocusedPane};
use crossterm::{
    event::{KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{Event, Events};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;
    let _ = app.refresh_data().await;

    let mut events = Events::new(Duration::from_millis(250));

    loop {
        terminal.draw(|f| ui::render(&mut app, f))?;

        if let Some(event) = events.next().await {
            match event {
                Event::Input(key) => {
                    if key.kind != KeyEventKind::Press { continue; }
                    app.status_msg = None;

                    if app.is_confirming_delete {
                        match key.code {
                            KeyCode::Char('y') => {
                                if let Some(idx) = app.saved_state.selected() {
                                    if let Some(conn) = app.saved.get(idx) {
                                        match app.nm.delete_connection(conn.path.clone()).await {
                                            Ok(()) => { let _ = app.refresh_data().await; }
                                            Err(e) => app.status_msg = Some(format!("Error: {}", e)),
                                        }
                                    }
                                }
                                app.is_confirming_delete = false;
                            }
                            _ => app.is_confirming_delete = false,
                        }
                        continue;
                    }

                    if app.is_inputting_password {
                        match key.code {
                            KeyCode::Char(c) => app.password_input.push(c),
                            KeyCode::Backspace => { app.password_input.pop(); }
                            KeyCode::Enter => {
                                let devices = app.nm.list_wifi_devices().await.unwrap_or_default();
                                if let Some(dev_path) = devices.first() {
                                    if let Some(idx) = app.available_state.selected() {
                                        if let Some(ap) = app.aps.get(idx) {
                                            match app.nm.add_and_activate_wifi(
                                                &ap.ssid,
                                                &app.password_input,
                                                dev_path.clone(),
                                                ap.path.clone(),
                                            ).await {
                                                Ok(()) => {
                                                    app.status_msg = Some(format!("Connecting to {}...", ap.ssid));
                                                }
                                                Err(e) => app.status_msg = Some(format!("Failed: {}", e)),
                                            }
                                        }
                                    }
                                }
                                app.is_inputting_password = false;
                                app.password_input.clear();
                            }
                            KeyCode::Esc => {
                                app.is_inputting_password = false;
                                app.password_input.clear();
                            }
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('r') => { let _ = app.refresh_data().await; }
                        KeyCode::Tab => app.switch_pane(),
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        KeyCode::Char('s') => {
                            let devices = app.nm.list_wifi_devices().await.unwrap_or_default();
                            if let Some(path) = devices.first() {
                                match app.nm.scan(path.clone()).await {
                                    Ok(()) => app.status_msg = Some("Scanning...".to_string()),
                                    Err(e) => app.status_msg = Some(format!("Error: {}", e)),
                                }
                            }
                        }
                        KeyCode::Char('d') => {
                            if app.focused_pane == FocusedPane::Saved && app.saved_state.selected().is_some() {
                                app.is_confirming_delete = true;
                            }
                        }
                        KeyCode::Enter => {
                            let devices = app.nm.list_wifi_devices().await.unwrap_or_default();
                            if let Some(dev_path) = devices.first() {
                                match app.focused_pane {
                                    FocusedPane::Available => {
                                        if let Some(idx) = app.available_state.selected() {
                                            if let Some(ap) = app.aps.get(idx) {
                                                if let Some(saved_conn) = app.saved.iter().find(|c| c.id == ap.ssid) {
                                                    match app.nm.activate_connection(saved_conn.path.clone(), dev_path.clone(), Some(ap.path.clone())).await {
                                                        Ok(()) => app.status_msg = Some(format!("Connecting to {}...", ap.ssid)),
                                                        Err(e) => app.status_msg = Some(format!("Error: {}", e)),
                                                    }
                                                } else {
                                                    app.is_inputting_password = true;
                                                }
                                            }
                                        }
                                    }
                                    FocusedPane::Saved => {
                                        if let Some(idx) = app.saved_state.selected() {
                                            if let Some(conn) = app.saved.get(idx) {
                                                match app.nm.activate_connection(conn.path.clone(), dev_path.clone(), None).await {
                                                    Ok(()) => app.status_msg = Some(format!("Connecting to {}...", conn.id)),
                                                    Err(e) => app.status_msg = Some(format!("Error: {}", e)),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::Tick => {
                    let _ = app.refresh_data().await;
                    if let Some(msg) = &app.status_msg {
                        if msg.starts_with("Connecting") && !app.active_ssids.is_empty() {
                            app.status_msg = None;
                        }
                    }
                }
            }
        }

        if app.should_quit { break; }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
