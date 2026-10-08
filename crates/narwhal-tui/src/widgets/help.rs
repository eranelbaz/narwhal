//! Help-panel modal renderer and static cheatsheet data.
//!
//! The cheatsheet is a compile-time constant — no introspection from the
//! keymap struct in v1. When bindings change, update this file by hand so
//! the docs stay in sync. The snapshot test (`snapshot_help_modal`) will
//! catch accidental drift.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use crate::theme::Theme;

/// One row in the cheatsheet table.
pub struct CheatsheetEntry {
    pub keys: &'static str,
    pub description: &'static str,
}

/// One section of the cheatsheet (e.g. "Global", "Editor").
pub struct CheatsheetSection {
    pub title: &'static str,
    pub entries: &'static [CheatsheetEntry],
}

/// All sections, in display order.
///
/// Bindings listed here are verified against the actual key-handling code.
/// When a new binding is added to `AppCore::handle_global_key`,
/// `handle_editor_key`, or `handle_results_key`, update the matching
/// section below and re-run the snapshot test.
pub const CHEATSHEET: &[CheatsheetSection] = &[
    CheatsheetSection {
        title: "Global",
        entries: &[
            CheatsheetEntry {
                keys: "F5 / Alt-Enter / Ctrl-;",
                description: "run statement under cursor",
            },
            CheatsheetEntry {
                keys: "F6",
                description: "run whole buffer",
            },
            CheatsheetEntry {
                keys: "F7",
                description: "stream cursor statement",
            },
            CheatsheetEntry {
                keys: "F4 / Ctrl-C",
                description: "cancel running query (while one runs)",
            },
            CheatsheetEntry {
                keys: "Ctrl-W",
                description: "cycle pane focus (basic/emacs: not from editor)",
            },
            CheatsheetEntry {
                keys: ":",
                description: "command palette (any pane)",
            },
            CheatsheetEntry {
                keys: "Ctrl-T",
                description: "new editor tab",
            },
            CheatsheetEntry {
                keys: "Ctrl-Tab / Ctrl-Shift-Tab",
                description: "cycle tabs",
            },
            CheatsheetEntry {
                keys: "F1 / ? (outside editor) / :help",
                description: "this help (j/k, Ctrl-D/U, g/G scroll)",
            },
            CheatsheetEntry {
                keys: "Ctrl-Shift-W",
                description: "cycle pane focus backwards",
            },
            CheatsheetEntry {
                keys: "Ctrl-S (vim editor)",
                description: "stream statement under cursor",
            },
            CheatsheetEntry {
                keys: "Ctrl-R",
                description: "query history (Shift-Enter inserts + runs)",
            },
            CheatsheetEntry {
                keys: "Ctrl-N",
                description: "goto: fuzzy jump to table / column",
            },
            CheatsheetEntry {
                keys: "Ctrl-PgDn / Ctrl-PgUp",
                description: "next / prev result tab",
            },
            CheatsheetEntry {
                keys: "Ctrl-P / Ctrl-Shift-P",
                description: "goto / command palette (vscode preset)",
            },
            CheatsheetEntry {
                keys: "Ctrl-B / Ctrl-Enter",
                description: "focus sidebar / run (datagrip, intellij preset)",
            },
            CheatsheetEntry {
                keys: ":q",
                description: "quit",
            },
            CheatsheetEntry {
                keys: ":refresh",
                description: "re-fetch schema tree for active connection",
            },
            CheatsheetEntry {
                keys: ":format / :fmt",
                description: "pretty-print the statement under the cursor",
            },
            CheatsheetEntry {
                keys: ":format-all / :fmtall",
                description: "pretty-print every statement in the buffer",
            },
        ],
    },
    CheatsheetSection {
        title: "Editor (vim)",
        entries: &[
            CheatsheetEntry {
                keys: "i / a",
                description: "enter insert mode",
            },
            CheatsheetEntry {
                keys: "Esc / Ctrl-C / Ctrl-[ / Ctrl-G",
                description: "back to normal mode",
            },
            CheatsheetEntry {
                keys: "Tab / Ctrl-Space (insert)",
                description: "completion; Ctrl-N / Ctrl-P / Esc in popup",
            },
            CheatsheetEntry {
                keys: "↑ ↓ / Shift-Tab",
                description: "cycle popup items",
            },
            CheatsheetEntry {
                keys: "Enter / Tab (in popup)",
                description: "accept completion",
            },
            CheatsheetEntry {
                keys: "h j k l / arrows",
                description: "move cursor",
            },
            CheatsheetEntry {
                keys: "w / b",
                description: "word forward / backward",
            },
            CheatsheetEntry {
                keys: "0 / $",
                description: "line start / end",
            },
            CheatsheetEntry {
                keys: "gg / G / {N}G",
                description: "first / last / Nth line",
            },
            CheatsheetEntry {
                keys: "Ctrl-D / Ctrl-U",
                description: "10 lines down / up",
            },
            CheatsheetEntry {
                keys: "{N} before a motion",
                description: "repeat count (3j, 2dw)",
            },
            CheatsheetEntry {
                keys: "/ / ? / n / N",
                description: "search forward / backward / next / prev",
            },
            CheatsheetEntry {
                keys: "v / V",
                description: "visual / visual-line mode",
            },
            CheatsheetEntry {
                keys: "d / y / c + motion",
                description: "operator: dd yy cc, dw, d$, dG, dgg …",
            },
            CheatsheetEntry {
                keys: "d / y / c (visual)",
                description: "delete / yank / change selection",
            },
            CheatsheetEntry {
                keys: "y / d (mouse selection)",
                description: "yank / delete the dragged selection",
            },
            CheatsheetEntry {
                keys: "x / Del",
                description: "delete char under cursor",
            },
            CheatsheetEntry {
                keys: "Alt-N / Alt-A",
                description: "multi-cursor: add next / all matches",
            },
            CheatsheetEntry {
                keys: "Esc (multi-cursor)",
                description: "collapse to primary cursor",
            },
        ],
    },
    CheatsheetSection {
        title: "Sidebar",
        entries: &[
            CheatsheetEntry {
                keys: "j / k / ↑ / ↓",
                description: "navigate",
            },
            CheatsheetEntry {
                keys: "Ctrl-D / Ctrl-U / PgDn / PgUp",
                description: "page down / up",
            },
            CheatsheetEntry {
                keys: "Home / End",
                description: "first / last item",
            },
            CheatsheetEntry {
                keys: "Enter",
                description: "connect / describe table",
            },
            CheatsheetEntry {
                keys: "o",
                description: "preview table data",
            },
            CheatsheetEntry {
                keys: "d",
                description: "inject DDL into editor",
            },
            CheatsheetEntry {
                keys: "D / gd",
                description: "ER diagram focused on table",
            },
        ],
    },
    CheatsheetSection {
        title: "Results",
        entries: &[
            CheatsheetEntry {
                keys: "h j k l / arrows",
                description: "move selection",
            },
            CheatsheetEntry {
                keys: "Enter",
                description: "open cell popup",
            },
            CheatsheetEntry {
                keys: "e",
                description: "edit cell value (Enter commit, Esc cancel)",
            },
            CheatsheetEntry {
                keys: "y / Y",
                description: "yank cell / row to clipboard",
            },
            CheatsheetEntry {
                keys: "R / Shift-Enter",
                description: "row detail modal",
            },
            CheatsheetEntry {
                keys: "s",
                description: "cycle sort on column: asc / desc / off",
            },
            CheatsheetEntry {
                keys: "/",
                description: "filter rows (Enter apply, Esc clear)",
            },
            CheatsheetEntry {
                keys: "Esc",
                description: "clear filter",
            },
            CheatsheetEntry {
                keys: "g / G",
                description: "jump to first / last row",
            },
            CheatsheetEntry {
                keys: ":next / :prev",
                description: "page through results",
            },
            CheatsheetEntry {
                keys: "]r / [r",
                description: "next / prev statement result",
            },
            CheatsheetEntry {
                keys: "f",
                description: "follow foreign key to parent row",
            },
            // ─── L36: row CRUD + pending changes ──────────────
            CheatsheetEntry {
                keys: "o / O",
                description: "queue INSERT (empty / duplicate row)",
            },
            CheatsheetEntry {
                keys: "d",
                description: "queue DELETE for the focused row",
            },
            CheatsheetEntry {
                keys: "Ctrl-S",
                description: "commit every staged mutation in a txn",
            },
            CheatsheetEntry {
                keys: "Ctrl-X",
                description: "discard the staged-mutation queue",
            },
            CheatsheetEntry {
                keys: "Ctrl-P",
                description: "toggle the pending-changes preview modal",
            },
            // ─── L36: metadata tabs ────────────────────────────
            CheatsheetEntry {
                keys: "1 / 2 / 3 / 4 / 5",
                description: "switch metadata tab: Records / Columns / Constraints / FKs / Indexes",
            },
            // ─── L36: JSON viewer ──────────────────────────────
            CheatsheetEntry {
                keys: "z",
                description: "open JSON viewer on cell",
            },
        ],
    },
    CheatsheetSection {
        title: "Cell popup / row detail",
        entries: &[
            CheatsheetEntry {
                keys: "Esc / q / Enter",
                description: "close cell popup",
            },
            CheatsheetEntry {
                keys: "j / k / PgDn / PgUp / g / G",
                description: "move between columns (row detail)",
            },
            CheatsheetEntry {
                keys: "Z",
                description: "open JSON viewer on column (row detail)",
            },
            CheatsheetEntry {
                keys: "Esc / R / Shift-Enter",
                description: "close row detail",
            },
        ],
    },
    CheatsheetSection {
        title: "JSON viewer",
        entries: &[
            CheatsheetEntry {
                keys: "j / k / Ctrl-D / Ctrl-U / g / G",
                description: "move cursor",
            },
            CheatsheetEntry {
                keys: "V / v",
                description: "toggle line selection",
            },
            CheatsheetEntry {
                keys: "y",
                description: "yank selection (or whole document)",
            },
            CheatsheetEntry {
                keys: "c",
                description: "copy value on cursor line",
            },
            CheatsheetEntry {
                keys: "Y",
                description: "yank raw cell text",
            },
            CheatsheetEntry {
                keys: "Esc",
                description: "clear selection, then close",
            },
            CheatsheetEntry {
                keys: "q",
                description: "close",
            },
        ],
    },
    CheatsheetSection {
        title: "Pending changes (Ctrl-P)",
        entries: &[
            CheatsheetEntry {
                keys: "j / k / Ctrl-D / Ctrl-U / g / G",
                description: "scroll",
            },
            CheatsheetEntry {
                keys: "Ctrl-S / Ctrl-X",
                description: "commit / discard",
            },
            CheatsheetEntry {
                keys: "Ctrl-P / Esc / q",
                description: "close",
            },
        ],
    },
    CheatsheetSection {
        title: "Diagram",
        entries: &[
            CheatsheetEntry {
                keys: "Tab / Shift-Tab / j / k",
                description: "cycle selected table",
            },
            CheatsheetEntry {
                keys: "Ctrl-D / Ctrl-U / g / G",
                description: "scroll",
            },
            CheatsheetEntry {
                keys: "Enter",
                description: "recenter on selected table",
            },
            CheatsheetEntry {
                keys: "i",
                description: "toggle focused / full view",
            },
            CheatsheetEntry {
                keys: "y",
                description: "yank as Mermaid",
            },
            CheatsheetEntry {
                keys: "Esc / q",
                description: "close",
            },
        ],
    },
    CheatsheetSection {
        title: "Pickers (goto / history / snippets / settings)",
        entries: &[
            CheatsheetEntry {
                keys: "type",
                description: "filter",
            },
            CheatsheetEntry {
                keys: "j / k / ↑ / ↓ / Ctrl-J / Ctrl-K",
                description: "move selection (goto also Ctrl-N / Ctrl-P)",
            },
            CheatsheetEntry {
                keys: "Ctrl-U",
                description: "clear query (goto)",
            },
            CheatsheetEntry {
                keys: "Enter",
                description: "accept",
            },
            CheatsheetEntry {
                keys: "j / k (settings)",
                description: "next / prev field",
            },
            CheatsheetEntry {
                keys: "Tab / Shift-Tab (settings)",
                description: "next / prev section",
            },
            CheatsheetEntry {
                keys: "Space / Enter (settings)",
                description: "toggle / cycle field",
            },
            CheatsheetEntry {
                keys: "Ctrl-S",
                description: "settings: save",
            },
            CheatsheetEntry {
                keys: "Esc",
                description: "close",
            },
        ],
    },
    CheatsheetSection {
        title: "Confirm / context menu",
        entries: &[
            CheatsheetEntry {
                keys: "type YES + Enter",
                description: "confirm a guarded write (Ctrl-U clears, Esc cancels)",
            },
            CheatsheetEntry {
                keys: "right-click (editor)",
                description: "context menu: j / k, Enter / Space, Esc",
            },
            CheatsheetEntry {
                keys: "mouse",
                description: "click / drag select, double / triple click, middle paste",
            },
        ],
    },
    CheatsheetSection {
        title: "Connections",
        entries: &[
            CheatsheetEntry {
                keys: ":add",
                description: "open the connection wizard (empty form)",
            },
            CheatsheetEntry {
                keys: ":url <dsn>",
                description: "prefill the wizard from a connection URL",
            },
            CheatsheetEntry {
                keys: ":test [name|url]",
                description: "dry-run a connection without opening a session",
            },
            CheatsheetEntry {
                keys: ":edit <name>",
                description: "edit a saved connection in the wizard",
            },
            CheatsheetEntry {
                keys: ":open <name|url>",
                description: "connect to a saved entry or an ad-hoc URL",
            },
            CheatsheetEntry {
                keys: ":remove <name>",
                description: "delete a saved connection (also :rm)",
            },
            CheatsheetEntry {
                keys: "ssh tunnel",
                description: "fill ssh_host + ssh_user in :add (or ?ssh_host=… in :url)",
            },
            CheatsheetEntry {
                keys: "pgpass / env",
                description: "PGPASSWORD / MYSQL_PWD / ~/.pgpass picked up automatically",
            },
            CheatsheetEntry {
                keys: "Tab / ↓ / Shift-Tab / ↑ (wizard)",
                description: "next / prev field",
            },
            CheatsheetEntry {
                keys: "← / → (wizard driver field)",
                description: "cycle driver",
            },
            CheatsheetEntry {
                keys: "Enter / Esc (wizard)",
                description: "save / cancel",
            },
            CheatsheetEntry {
                keys: "Tab on path field",
                description: "filesystem completion in the wizard",
            },
        ],
    },
    CheatsheetSection {
        title: "Snippets",
        entries: &[
            CheatsheetEntry {
                keys: ":save <name>",
                description: "save editor buffer as a named snippet",
            },
            CheatsheetEntry {
                keys: ":load <name>",
                description: "load a snippet into a new tab",
            },
            CheatsheetEntry {
                keys: ":rm-snippet <name>",
                description: "delete a saved snippet",
            },
            CheatsheetEntry {
                keys: ":snippets",
                description: "browse saved snippets",
            },
        ],
    },
];

/// Editor mode hint used to swap which cheatsheet pages get
/// rendered. Only the editor-mode chord set changes; the global
/// shortcuts stay constant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum HelpEditorMode {
    #[default]
    Vim,
    Basic,
    Emacs,
}

/// Cheatsheet entries for basic (modeless) editor mode.
pub const CHEATSHEET_BASIC_EDITOR: &[CheatsheetEntry] = &[
    CheatsheetEntry {
        keys: "Arrow / Home / End",
        description: "move cursor",
    },
    CheatsheetEntry {
        keys: "Ctrl-← / Ctrl-→",
        description: "word jump",
    },
    CheatsheetEntry {
        keys: "Ctrl-Home / Ctrl-End / PgUp / PgDn",
        description: "buffer start / end, page",
    },
    CheatsheetEntry {
        keys: "Shift-Arrow",
        description: "extend selection",
    },
    CheatsheetEntry {
        keys: "Ctrl-A",
        description: "select all",
    },
    CheatsheetEntry {
        keys: "Ctrl-C / Ctrl-X",
        description: "copy / cut selection",
    },
    CheatsheetEntry {
        keys: "Ctrl-V",
        description: "paste clipboard",
    },
    CheatsheetEntry {
        keys: "Ctrl-Z / Ctrl-Y / Ctrl-Shift-Z",
        description: "undo / redo",
    },
    CheatsheetEntry {
        keys: "Ctrl-F / /",
        description: "find in buffer",
    },
    CheatsheetEntry {
        keys: "Tab",
        description: "completion / indent",
    },
    CheatsheetEntry {
        keys: ":",
        description: "open command palette",
    },
    CheatsheetEntry {
        keys: "Esc",
        description: "clear selection / close popups",
    },
];

/// Cheatsheet entries for emacs editor mode.
pub const CHEATSHEET_EMACS_EDITOR: &[CheatsheetEntry] = &[
    CheatsheetEntry {
        keys: "C-f / C-b",
        description: "forward / backward char",
    },
    CheatsheetEntry {
        keys: "C-n / C-p",
        description: "next / previous line",
    },
    CheatsheetEntry {
        keys: "C-a / C-e",
        description: "beginning / end of line",
    },
    CheatsheetEntry {
        keys: "M-f / M-b",
        description: "forward / backward word",
    },
    CheatsheetEntry {
        keys: "M-< / M->",
        description: "beginning / end of buffer",
    },
    CheatsheetEntry {
        keys: "C-Space",
        description: "set mark",
    },
    CheatsheetEntry {
        keys: "C-w / M-w",
        description: "kill / copy region",
    },
    CheatsheetEntry {
        keys: "C-y",
        description: "yank (paste)",
    },
    CheatsheetEntry {
        keys: "C-k",
        description: "kill to end of line",
    },
    CheatsheetEntry {
        keys: "C-d / Del / M-d",
        description: "delete char / word",
    },
    CheatsheetEntry {
        keys: "C-/ / C-_ / C-x u",
        description: "undo",
    },
    CheatsheetEntry {
        keys: "C-s / C-r",
        description: "search forward / backward",
    },
    CheatsheetEntry {
        keys: "C-x C-s",
        description: "submit / run statement",
    },
    CheatsheetEntry {
        keys: "C-g / Esc",
        description: "cancel / clear region",
    },
    CheatsheetEntry {
        keys: "Arrows / Home / End",
        description: "move cursor",
    },
    CheatsheetEntry {
        keys: "Tab",
        description: "completion / indent",
    },
    CheatsheetEntry {
        keys: ":",
        description: "open command palette",
    },
];

/// Render the help modal on top of the current frame.
///
/// The modal occupies a centred rectangle (max 60×24, otherwise 70% of
/// available space) and displays each cheatsheet section as a labelled
/// two-column table (shortcut → description).
///
/// `editor_mode` swaps the editor-section content between vim,
/// basic and emacs without rebuilding the entire cheatsheet.
/// Returns the largest useful scroll offset so the host can clamp.
pub fn render_help_modal(
    frame: &mut Frame<'_>,
    area: Rect,
    theme: &Theme,
    editor_mode: HelpEditorMode,
    scroll: u16,
) -> u16 {
    let (max_width, max_height) = crate::constants::HELP_MODAL_MAX;
    let width = (area.width * 8 / 10).min(max_width);
    let height = (area.height * 9 / 10).min(max_height);
    if width < 30 || height < 8 {
        return 0;
    }
    let popup = centred(area, width, height);
    frame.render_widget(Clear, popup);

    let title = " help · j/k Ctrl-D/U g/G scroll · esc/q closes ";
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(
            title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut lines: Vec<Line<'_>> = Vec::new();
    let key_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.foreground);
    let heading_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);

    for section in CHEATSHEET {
        // Replace the vim editor section with the matching
        // basic / emacs chord set when one of those modes is
        // active; every other section is mode-agnostic.
        let entries: &[CheatsheetEntry] = match (section.title, editor_mode) {
            ("Editor (vim)", HelpEditorMode::Basic) => CHEATSHEET_BASIC_EDITOR,
            ("Editor (vim)", HelpEditorMode::Emacs) => CHEATSHEET_EMACS_EDITOR,
            _ => section.entries,
        };
        let title = match (section.title, editor_mode) {
            ("Editor (vim)", HelpEditorMode::Basic) => "Editor (basic)",
            ("Editor (vim)", HelpEditorMode::Emacs) => "Editor (emacs)",
            (t, _) => t,
        };
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            format!(" {title} "),
            heading_style,
        )));
        for entry in entries {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<36} ", entry.keys), key_style),
                Span::styled(entry.description, desc_style),
            ]));
        }
    }

    let max_scroll = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .saturating_sub(inner.height);
    frame.render_widget(
        Paragraph::new(lines).scroll((scroll.min(max_scroll), 0)),
        inner,
    );
    max_scroll
}

pub(crate) use super::centred_rect as centred;
