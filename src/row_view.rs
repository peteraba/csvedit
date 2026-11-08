use crate::{csv::Row, view::Header, wrap::LineWrapper};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

/// A snapshot keeps inspection stable even when a watched file changes.
pub struct RowView {
    record_num: usize,
    fields: Vec<(String, String)>,
    offset: usize,
    max_offset: usize,
}

impl RowView {
    pub fn new(headers: &[Header], row: &Row) -> Self {
        Self {
            record_num: row.record_num,
            fields: headers
                .iter()
                .enumerate()
                .map(|(i, header)| {
                    (
                        header.name.clone(),
                        row.fields.get(i).cloned().unwrap_or_default(),
                    )
                })
                .collect(),
            offset: 0,
            max_offset: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        self.offset = self.offset.saturating_add(1).min(self.max_offset);
    }
    pub fn scroll_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }
    pub fn scroll_top(&mut self) {
        self.offset = 0;
    }
    pub fn scroll_bottom(&mut self) {
        self.offset = self.max_offset;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Row {} ", self.record_num))
            .title_bottom(" Esc / Enter: return | ↑/↓: scroll ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let columns = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);
        let mut names = Vec::new();
        let mut values = Vec::new();
        for (name, value) in &self.fields {
            let mut name_lines = wrapped(name, columns[0].width.saturating_sub(1));
            let mut value_lines = wrapped(value, columns[1].width);
            let height = name_lines.len().max(value_lines.len());
            name_lines.resize(height, Line::default());
            value_lines.resize(height, Line::default());
            names.extend(name_lines);
            values.extend(value_lines);
            names.push(Line::default());
            values.push(Line::default());
        }
        let height = inner.height.saturating_sub(1) as usize;
        self.max_offset = names.len().saturating_sub(1).saturating_sub(height);
        self.offset = self.offset.min(self.max_offset);
        for (column, heading, lines) in
            [(columns[0], "Column", names), (columns[1], "Value", values)]
        {
            frame.render_widget(
                Paragraph::new(heading).style(Style::default().add_modifier(Modifier::BOLD)),
                Rect {
                    height: column.height.min(1),
                    ..column
                },
            );
            frame.render_widget(
                Paragraph::new(lines.into_iter().skip(self.offset).collect::<Vec<_>>()),
                Rect {
                    y: column.y.saturating_add(1),
                    height: column.height.saturating_sub(1),
                    ..column
                },
            );
        }
    }
}

fn wrapped(value: &str, width: u16) -> Vec<Line<'static>> {
    let spans = [Span::raw(value.to_owned())];
    let mut wrapper = LineWrapper::new(&spans, width.max(1) as usize, false);
    let mut lines = Vec::new();
    while let Some(line) = wrapper.next() {
        lines.push(Line::from(
            line.spans
                .into_iter()
                .map(|span| Span::styled(span.content.into_owned(), span.style))
                .collect::<Vec<_>>(),
        ));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn long_values_scroll_and_resize() {
        let mut view = RowView {
            record_num: 1,
            fields: vec![("Name".into(), "line\n".repeat(20) + "last value")],
            offset: 0,
            max_offset: 0,
        };
        let mut terminal = Terminal::new(TestBackend::new(40, 8)).unwrap();
        terminal.draw(|f| view.render(f, f.area())).unwrap();
        assert!(view.max_offset > 0);
        view.scroll_bottom();
        terminal.draw(|f| view.render(f, f.area())).unwrap();
        let buffer = terminal.backend().buffer();
        let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
        assert!(text.contains("last value"));
        for size in [(1, 1), (0, 0), (80, 50)] {
            terminal.backend_mut().resize(size.0, size.1);
            terminal.draw(|f| view.render(f, f.area())).unwrap();
        }
        assert_eq!(view.offset, 0);
    }
}
