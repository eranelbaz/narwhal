//! Full-screen, scrollable JSON viewer modal.
//!
//! Used by the result pane when the user requests a deep look at a cell
//! that carries structured JSON. The host pretty-prints the payload
//! with `serde_json` (or, on parse failure, falls back to the raw cell
//! text) and hands us the resulting [`JsonViewerView`] each render.
//!
//! The widget itself owns no state — scroll offset and lifecycle live on
//! the host side. The TUI crate only paints.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

/// Read-only snapshot of the JSON viewer at render time.
///
/// `pretty` is the formatted output the user actually sees; `raw` is
/// kept around so the yank-raw shortcut can return bytes exactly as
/// they came back from the engine.
#[derive(Debug, Clone)]
pub struct JsonViewerView<'a> {
    /// Header label, usually the source column name plus its SQL type.
    pub title: &'a str,
    /// Pretty-printed JSON text, line-broken with `\n`.
    pub pretty: &'a str,
    /// Untouched cell text. Surfaced via the `Y`/raw-yank shortcut and
    /// in the footer hint when parse failed.
    pub raw: &'a str,
    /// First visible line (0-based). Clamped at render time so a stale
    /// host-side value cannot scroll past the buffer.
    pub scroll: u16,
    /// `Some` when the original cell text didn't parse as JSON. The
    /// modal still opens — the user just sees the raw text plus a
    /// muted footer noting the fallback.
    pub parse_error: Option<&'a str>,
    /// Line under the cursor (0-based).
    pub cursor: usize,
    /// Inclusive line range of the visual selection.
    pub selection: Option<(usize, usize)>,
}

/// Render the modal on top of `area`. Always centred; the modal claims
/// 80% of width and height so the underlying result pane stays visible
/// at the edges as context. Returns the body height in rows and the
/// scroll actually used, nudged so the cursor line stays visible.
pub fn render_json_viewer(
    frame: &mut Frame<'_>,
    area: Rect,
    view: &JsonViewerView<'_>,
    theme: &Theme,
) -> (u16, u16) {
    let modal = centred_rect(80, 80, area);
    frame.render_widget(Clear, modal);

    let title = format!(" JSON — {} ", view.title);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(modal);
    frame.render_widget(block, modal);

    // Reserve one line at the bottom for the footer hint.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    // Clamp scroll against the actual content length so we never scroll
    // past the end (host might lag a frame behind a buffer change).
    let total_lines = view.pretty.lines().count();
    let mut scroll = usize::from(view.scroll).min(total_lines.saturating_sub(1));
    let width = usize::from(chunks[0].width.max(1));
    // chisle: char-width estimate; word wrap can use a row more on long lines.
    let rows = |l: &str| l.width().div_ceil(width).max(1);
    let all: Vec<&str> = view.pretty.lines().collect();
    while scroll < view.cursor
        && all[scroll..=view.cursor.min(all.len().saturating_sub(1))]
            .iter()
            .map(|l| rows(l))
            .sum::<usize>()
            > usize::from(chunks[0].height)
    {
        scroll += 1;
    }
    let lines: Vec<Line<'_>> = view
        .pretty
        .lines()
        .enumerate()
        .skip(scroll)
        .map(|(i, l)| {
            let selected = view.selection.is_some_and(|(a, b)| (a..=b).contains(&i));
            let style = if selected {
                Style::default().bg(theme.accent).fg(theme.background)
            } else if i == view.cursor {
                Style::default().bg(theme.muted)
            } else {
                Style::default()
            };
            Line::styled(l, style)
        })
        .collect();
    let body = Paragraph::new(lines)
        .style(Style::default().fg(theme.foreground))
        .wrap(Wrap { trim: false });
    frame.render_widget(body, chunks[0]);

    let hint_text = match view.parse_error {
        Some(err) => format!(
            "  raw cell text shown (parse failed: {err}) · j/k move · V select · y copy · q/Esc close",
        ),
        None => "  j/k move · V select · y copy · c copy value · Y raw · q/Esc close".into(),
    };
    let hint = Paragraph::new(Line::from(Span::styled(
        hint_text,
        Style::default().fg(theme.muted),
    )));
    frame.render_widget(hint, chunks[1]);
    (chunks[0].height, u16::try_from(scroll).unwrap_or(u16::MAX))
}

/// Compute a centred rectangle that occupies `pct_x`/`pct_y` of `area`.
fn centred_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn scroll_follows_cursor_past_wrapped_lines() {
        let mut terminal = Terminal::new(TestBackend::new(40, 20)).expect("backend");
        let pretty = format!("{}\n{}", "x".repeat(200), "y\n".repeat(12));
        let view = JsonViewerView {
            title: "t",
            pretty: &pretty,
            raw: &pretty,
            scroll: 0,
            parse_error: None,
            cursor: 8,
            selection: None,
        };
        let mut out = (0, 0);
        terminal
            .draw(|f| out = render_json_viewer(f, f.area(), &view, &Theme::default()))
            .expect("draw");
        assert!(
            out.1 > 0,
            "long wrapped line must scroll off to keep cursor visible"
        );
    }
}
