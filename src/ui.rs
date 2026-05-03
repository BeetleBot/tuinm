use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};
use crate::app::{App, FocusedPane};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let status_text = if let Some(msg) = &app.status_msg {
        format!("  {} ", msg)
    } else if !app.active_ssids.is_empty() {
        format!(" 󰈀 Online: {} ", app.active_ssids[0])
    } else {
        " 󱘖 Offline ".to_string()
    };
    let status_color = if app.status_msg.is_some() { 
        Color::Yellow 
    } else if !app.active_ssids.is_empty() { 
        Color::Green 
    } else { 
        Color::Red 
    };

    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let title = Paragraph::new(" tuinm 󰀂 ")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(title, header_chunks[0]);

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(status, header_chunks[1]);

    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_saved_pane(app, frame, body_chunks[0]);
    render_available_pane(app, frame, body_chunks[1]);

    let footer = Paragraph::new(" [q] Quit | [r] Refresh | [s] Scan | [Tab] Switch | [Enter] Connect | [d] Delete ")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    frame.render_widget(footer, chunks[2]);

    if app.is_confirming_delete {
        render_modal(frame, " Confirm Delete ", " Delete this connection?\n\n [y] Yes  [n] No ", Color::Red);
    }

    if app.is_inputting_password {
        let masked: String = app.password_input.chars().map(|_| '*').collect();
        render_modal(frame, " Enter Password ", &format!("\n Password: {}\n\n [Enter] Connect  [Esc] Cancel", masked), Color::Yellow);
    }
}

fn render_saved_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    let focus_color = if app.focused_pane == FocusedPane::Saved { Color::Magenta } else { Color::DarkGray };
    let rows = app.saved.iter().map(|c| {
        let is_active = app.active_ssids.contains(&c.id);
        let style = if is_active {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_active { " " } else { "  " };
        Row::new(vec![Cell::from(format!("{}{}", prefix, c.id)).style(style)])
    });

    let table = Table::new(rows, [Constraint::Min(0)])
        .block(
            Block::default()
                .title(" 󰒄 Saved Connections ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(focus_color)),
        )
        .row_highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
        .highlight_symbol("󰁔 ");
    frame.render_stateful_widget(table, area, &mut app.saved_state);
}

fn render_available_pane(app: &mut App, frame: &mut Frame, area: Rect) {
    let focus_color = if app.focused_pane == FocusedPane::Available { Color::Magenta } else { Color::DarkGray };
    let rows = app.aps.iter().map(|ap| {
        let is_active = app.active_ssids.contains(&ap.ssid);
        let icon = match ap.strength {
            s if s >= 80 => "󰤨",
            s if s >= 60 => "󰤥",
            s if s >= 40 => "󰤢",
            _ => "󰤟",
        };
        let color = match ap.strength {
            s if s >= 80 => Color::Green,
            s if s >= 50 => Color::Yellow,
            _ => Color::Red,
        };
        
        let style = if is_active {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_active { " " } else { "  " };

        Row::new(vec![
            Cell::from(icon).style(Style::default().fg(color)),
            Cell::from(format!("{}{}", prefix, ap.ssid)).style(style),
            Cell::from(format!("{}%", ap.strength)).style(Style::default().fg(Color::DarkGray)),
        ])
    });

    let table = Table::new(rows, [Constraint::Length(4), Constraint::Min(0), Constraint::Length(8)])
        .header(Row::new(vec!["", "SSID", "Signal"]).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .block(
            Block::default()
                .title(" 󰖩 Available Networks ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(focus_color)),
        )
        .row_highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
        .highlight_symbol("󰁔 ");
    frame.render_stateful_widget(table, area, &mut app.available_state);
}

fn render_modal(frame: &mut Frame, title: &str, content: &str, color: Color) {
    let area = centered_rect(50, 20, frame.area());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));
    let text = Paragraph::new(content)
        .block(block)
        .alignment(Alignment::Center);
    frame.render_widget(Clear, area);
    frame.render_widget(text, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup[1])[1]
}
