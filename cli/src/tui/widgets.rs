use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Gauge, Table, Row, Cell, Wrap,
    },
    Frame,
};

pub fn render_banner(f: &mut Frame, area: Rect, title: &str, subtitle: Option<&str>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(area);

    let title_text = vec![Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let title_widget = Paragraph::new(title_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).style(Style::default()));

    f.render_widget(title_widget, chunks[0]);

    if let Some(sub) = subtitle {
        let subtitle_text = vec![Line::from(Span::styled(
            sub,
            Style::default().fg(Color::Gray),
        ))];

        let subtitle_widget = Paragraph::new(subtitle_text)
            .alignment(Alignment::Center);

        f.render_widget(subtitle_widget, chunks[1]);
    }
}

pub fn render_progress(f: &mut Frame, area: Rect, label: &str, progress: f64) {
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(label))
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .ratio(progress);

    f.render_widget(gauge, area);
}

pub fn render_list<'a>(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: Vec<String>,
    selected: Option<usize>,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if Some(i) == selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(item.as_str()).style(style)
        })
        .collect();

    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

pub fn render_table(
    f: &mut Frame,
    area: Rect,
    title: &str,
    headers: Vec<&str>,
    rows: Vec<Vec<String>>,
) {
    let header_cells = headers
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });

    let header = Row::new(header_cells)
        .style(Style::default())
        .height(1);

    let table_rows: Vec<Row> = rows
        .iter()
        .map(|row| {
            let cells = row.iter().map(|c| Cell::from(c.as_str()));
            Row::new(cells).height(1)
        })
        .collect();

    let widths = vec![Constraint::Percentage(100 / headers.len() as u16); headers.len()];

    let table = Table::new(table_rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title))
        .style(Style::default().fg(Color::White));

    f.render_widget(table, area);
}

pub fn render_info_box(f: &mut Frame, area: Rect, title: &str, content: Vec<String>) {
    let text: Vec<Line> = content
        .iter()
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

pub fn render_scrollable_info_box(
    f: &mut Frame,
    area: Rect,
    title: &str,
    content: Vec<String>,
    scroll: u16,
) {
    let text: Vec<Line> = content
        .iter()
        .map(|line| Line::from(Span::raw(line)))
        .collect();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .style(Style::default()),
        )
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

pub fn render_status(f: &mut Frame, area: Rect, message: &str, is_error: bool) {
    let color = if is_error { Color::Red } else { Color::Green };

    let text = vec![Line::from(Span::styled(
        message,
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Status"));

    f.render_widget(paragraph, area);
}

pub fn render_key_hints(f: &mut Frame, area: Rect, hints: Vec<(&str, &str)>) {
    let text: Vec<Line> = hints
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("[{}] ", key),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(*desc),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Controls"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

