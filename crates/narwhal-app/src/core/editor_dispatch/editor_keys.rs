//! Editor pane key handling and action interpretation.

use crossterm::event::{KeyCode as CtKey, KeyEvent, KeyModifiers};
use narwhal_core::ColumnHeader;
use narwhal_domain::Motion as DomainMotion;
use narwhal_domain::editor::Selection;
use narwhal_tui::translate_key_event;
use narwhal_vim::{Action, Mode, Motion as VimMotion, Operator};

use crate::completion::{
    CompletionContext, detect_context_with_schemas, gather as gather_completions,
};
use crate::core::{AppCore, CompletionState};
use crate::meta::MetaUpdate;

/// True when the word being completed starts immediately after a `.`
/// (`ast.|` or `ast.i|`), i.e. a column reference rather than a
/// dotted name the cursor has since moved past.
pub(crate) fn right_after_dot(buffer: &str, cursor_offset: usize, prefix: &str) -> bool {
    buffer
        .get(..cursor_offset.saturating_sub(prefix.len()))
        .is_some_and(|before| before.ends_with('.'))
}

/// Convert a `narwhal_vim::Motion` to `narwhal_domain::Motion`.
///
/// The two enums are isomorphic but live in separate crates to avoid
/// a domain-level dependency on the vim crate.
const fn domain_motion(m: VimMotion) -> DomainMotion {
    match m {
        VimMotion::Left => DomainMotion::Left,
        VimMotion::Right => DomainMotion::Right,
        VimMotion::Up => DomainMotion::Up,
        VimMotion::Down => DomainMotion::Down,
        VimMotion::WordForward => DomainMotion::WordForward,
        VimMotion::WordBackward => DomainMotion::WordBackward,
        VimMotion::LineStart => DomainMotion::LineStart,
        VimMotion::LineEnd => DomainMotion::LineEnd,
        VimMotion::FileStart => DomainMotion::FileStart,
        VimMotion::FileEnd => DomainMotion::FileEnd,
        VimMotion::CurrentLine => DomainMotion::CurrentLine,
        // `narwhal_vim::Motion` is #[non_exhaustive]; future variants
        // map to a no-op motion.
        _ => DomainMotion::CurrentLine,
    }
}

impl AppCore {
    /// add a secondary cursor at the next occurrence of the
    /// word under the primary cursor.
    pub(crate) async fn add_multi_cursor_next(&mut self) {
        let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
        if buf.add_secondary_cursor_at_next_word_match() {
            let count = buf.secondary_cursors().len();
            self.ui.status.message = format!("multi-cursor: {count} secondary cursor(s)");
        } else {
            self.ui.status.message = "multi-cursor: no match for word under cursor".into();
        }
    }

    /// add a secondary cursor at every other occurrence of
    /// the word under the primary cursor.
    pub(crate) async fn add_multi_cursor_all(&mut self) {
        let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
        let added = buf.add_secondary_cursors_at_all_word_matches();
        if added > 0 {
            self.ui.status.message = format!("multi-cursor: added {added} cursor(s)");
        } else {
            self.ui.status.message = "multi-cursor: no other matches".into();
        }
    }

    pub(crate) async fn accept_completion_at(&mut self, index: usize) {
        let Some(state) = self.ui.tabs[self.ui.active_tab].completion.as_mut() else {
            return;
        };
        if index >= state.items.len() {
            return;
        }
        state.selected = index;
        let choice = state.items[index].text.clone();
        self.ui.tabs[self.ui.active_tab]
            .editor
            .replace_current_word_with(&choice);
        self.ui.tabs[self.ui.active_tab].completion = None;
        self.ui.status.message = format!("completed: {choice}");
    }

    /// Click on a sidebar table row: navigate the sidebar to that index
    /// and run a preview query. Uses `run_preview` (same as the
    /// keyboard-driven `o` path) so that `pending_source` is set and
    /// cell editing (`e`) works on mouse-previewed tables (M15).
    pub(crate) async fn click_result_tab(&mut self, result_idx: usize) {
        let bundle = &mut self.ui.tabs[self.ui.active_tab].results;
        if result_idx < bundle.len() && bundle.is_multi() {
            bundle.active = result_idx;
            let total = bundle.len();
            self.ui.status.message = format!("result {} of {total}", result_idx + 1);
        }
    }

    pub(crate) async fn handle_editor_key(&mut self, key: KeyEvent) {
        // Context menu owns the keyboard while it's visible —
        // shared by every editor mode (vim / basic / emacs).
        if self.ui.context_menu.is_some() {
            self.handle_context_menu_key(key).await;
            return;
        }
        // The editor search prompt is modal: characters build the needle,
        // Enter accepts, Esc cancels and restores the cursor.
        if self.ui.tabs[self.ui.active_tab].editor_search.prompt_open {
            self.handle_editor_search_key(key).await;
            return;
        }
        // Mode dispatch: vim is the default and historical path;
        // basic and emacs run on dedicated modeless handlers.
        // Multi-cursor / completion interception below stays shared
        // because those popups are mode-agnostic.
        //
        // EXCEPTION: when the vim layer is in Command-line mode
        // (the `:` prompt that basic/emacs open via their cross-mode
        // shortcut), keep routing keystrokes through the vim path
        // so the user can finish typing the command. Without this,
        // a basic-mode user pressing `:settings<Enter>` would see
        // the prompt open on `:`, then the subsequent characters
        // would be inserted into the SQL buffer as plain text and
        // the command would never run.
        if self.ui.vim.mode() != Mode::Command {
            match self.ui.editor_mode {
                narwhal_config::EditorMode::Basic => {
                    self.handle_editor_key_basic(key).await;
                    return;
                }
                narwhal_config::EditorMode::Emacs => {
                    self.handle_editor_key_emacs(key).await;
                    return;
                }
                // Vim falls through into the legacy machinery below.
                narwhal_config::EditorMode::Vim | _ => {}
            }
        }
        // multi-cursor chords intercepted before vim:
        // - Alt-N: add a secondary cursor at the next occurrence of
        // the word under the primary cursor
        // - Alt-A: add a secondary cursor at every other occurrence
        // - Esc when multi-cursor is active: collapse to primary
        // (intercepted only if not in vim insert mode — normal-mode
        // Esc behaviour stays untouched)
        if key.modifiers == KeyModifiers::ALT {
            match key.code {
                CtKey::Char('n' | 'N') => {
                    self.add_multi_cursor_next().await;
                    return;
                }
                CtKey::Char('a' | 'A') => {
                    self.add_multi_cursor_all().await;
                    return;
                }
                _ => {}
            }
        }
        if key.code == CtKey::Esc && self.ui.tabs[self.ui.active_tab].editor.has_multi_cursors() {
            self.ui.tabs[self.ui.active_tab]
                .editor
                .collapse_to_primary();
            self.ui.status.message = "multi-cursor: collapsed".into();
            // Fall through so vim still receives the Esc — leaves
            // insert mode like normal. The buffer's secondary set has
            // already been cleared.
        }
        // The completion popup is modal while it's open: Tab cycles,
        // Enter accepts, Esc closes. Plain character keys fall through
        // so the user can keep typing and the popup refreshes against
        // the new prefix on the way out.
        if self.ui.tabs[self.ui.active_tab].completion.is_some()
            && self.handle_completion_key(key).await
        {
            return;
        }
        // In insert mode, intercept a plain Tab so it triggers completion
        // instead of being forwarded to the vim layer.
        if self.ui.vim.mode() == Mode::Insert && key.code == CtKey::Tab && key.modifiers.is_empty()
        {
            self.trigger_completion().await;
            return;
        }
        if self.ui.vim.mode() == Mode::Normal
            && self.ui.tabs[self.ui.active_tab].editor.has_selection()
            && key.modifiers.difference(KeyModifiers::SHIFT).is_empty()
        {
            match key.code {
                CtKey::Char('y') => return self.apply_visual_operator(Operator::Yank),
                CtKey::Char('d' | 'x') | CtKey::Delete => {
                    return self.apply_visual_operator(Operator::Delete);
                }
                _ => {}
            }
        }
        let Some(logical) = translate_key_event(key) else {
            return;
        };
        let action = self.ui.vim.handle(logical);
        self.apply_action(action).await;

        // After every insert-mode keystroke, refresh the completion
        // popup against the new word prefix. Two thresholds:
        // - prefix.len() >= 2 opens or refreshes the popup;
        // - prefix.len() < 2 closes any open popup so the user can
        // type short words without a flashing list.
        // Silent: no status spam, no '4-space' fallback — manual Tab
        // / Ctrl-Space still handle those cases.
        if self.ui.vim.mode() == Mode::Insert {
            self.maybe_auto_complete().await;
        }
    }

    /// Build a column-name lookup map from the session's schema cache.
    ///
    /// Keys are lowercased table names; values are `(schema_name, columns)`
    /// tuples so each column completion can carry the schema as its detail
    /// string. Returns an empty map when no session is active.
    pub(crate) async fn column_cache(
        &self,
    ) -> std::collections::HashMap<String, (String, Vec<ColumnHeader>)> {
        let Some(session) = self.session.active.as_ref() else {
            return std::collections::HashMap::new();
        };
        let mut map = std::collections::HashMap::new();
        for (schema, tables) in &session.schemas {
            for table in tables {
                let key = table.name.to_ascii_lowercase();
                // Only insert if not already present (first schema wins).
                map.entry(key)
                    .or_insert_with(|| (schema.name.clone(), Vec::new()));
            }
        }
        // Merge any cached column data from the session.
        for (table_lower, (schema_name, cols)) in &session.column_cache {
            map.insert(table_lower.clone(), (schema_name.clone(), cols.clone()));
        }
        map
    }

    /// Describe `context`'s table in the background on first use so
    /// column completion works without opening it from the sidebar. The
    /// reply ([`MetaUpdate::ColumnsFetched`]) fills the cache and
    /// refreshes the popup, so a slow remote describe never blocks typing.
    pub(crate) fn ensure_columns_cached(&mut self, context: &CompletionContext) {
        let CompletionContext::ColumnExpected { table } = context else {
            return;
        };
        let Some(session) = self.session.active.as_mut() else {
            return;
        };
        if session.column_cache.contains_key(table) || session.columns_in_flight.contains(table) {
            return;
        }
        let Some((schema, real_name)) = session.schemas.iter().find_map(|(schema, tables)| {
            tables
                .iter()
                .find(|t| t.name.eq_ignore_ascii_case(table))
                .map(|t| (schema.name.clone(), t.name.clone()))
        }) else {
            return;
        };
        session.columns_in_flight.insert(table.clone());
        let pool = session.pool.clone();
        let session_id = session.config.id;
        let table = table.clone();
        let meta_tx = self.process.meta_tx.clone();
        tokio::spawn(async move {
            let result = async {
                let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
                conn.describe_table(&schema, &real_name)
                    .await
                    .map_err(|e| e.to_string())
            }
            .await
            .map(|ts| {
                ts.columns
                    .into_iter()
                    .map(|c| ColumnHeader {
                        name: c.name,
                        data_type: c.data_type,
                    })
                    .collect()
            });
            let _ = meta_tx
                .send(MetaUpdate::ColumnsFetched {
                    session_id,
                    table,
                    schema,
                    result,
                })
                .await;
        });
    }

    /// Refresh-or-close the completion popup based on the current word
    /// prefix. Called after every insert-mode keystroke. See
    /// [`Self::trigger_completion`] for the manual (Tab / Ctrl-Space)
    /// variant that handles the empty-prefix and no-matches cases
    /// explicitly.
    pub(crate) async fn maybe_auto_complete(&mut self) {
        let prefix = self.ui.tabs[self.ui.active_tab]
            .editor
            .current_word_prefix();
        let schemas = self
            .session
            .active
            .as_ref()
            .map_or(&[][..], |s| s.schemas.as_slice());
        let known_schemas: Vec<String> = schemas.iter().map(|(s, _)| s.name.clone()).collect();
        let buffer_text = self.ui.tabs[self.ui.active_tab].editor.entire_text();
        let offset = self.ui.tabs[self.ui.active_tab].editor.cursor_byte_offset();
        let context = detect_context_with_schemas(&buffer_text, offset, &known_schemas);
        // Columns open right after `ident.`; everything else waits for 2 chars.
        let min_prefix = if right_after_dot(&buffer_text, offset, &prefix) {
            0
        } else {
            2
        };
        if prefix.len() < min_prefix {
            self.ui.tabs[self.ui.active_tab].completion = None;
            return;
        }
        self.ensure_columns_cached(&context);
        let schemas = self
            .session
            .active
            .as_ref()
            .map_or(&[][..], |s| s.schemas.as_slice());
        let columns = self.column_cache().await;
        let items = gather_completions(&prefix, schemas, &context, &columns, 50);
        if items.is_empty() {
            self.ui.tabs[self.ui.active_tab].completion = None;
            return;
        }
        // Preserve the user's current selection across keystrokes when
        // possible — a brand-new popup starts at index 0.
        let selected = self.ui.tabs[self.ui.active_tab]
            .completion
            .as_ref()
            .map_or(0, |c| c.selected.min(items.len() - 1));
        self.ui.tabs[self.ui.active_tab].completion = Some(CompletionState {
            items,
            selected,
            prefix,
        });
    }

    /// Open the editor search prompt (`/` for forward, `?` for backward).
    pub(crate) async fn apply_action(&mut self, action: Action) {
        match action {
            Action::Move { motion, count } => {
                self.ui.tabs[self.ui.active_tab]
                    .editor
                    .apply_motion(domain_motion(motion), count);
                self.sync_visual_selection();
            }
            Action::InsertText(text) => {
                // Note: warn the user when a
                // multi-line paste collapses the secondary-cursor
                // set. The buffer drops them silently (paste-into-
                // multi-cursor is v2.1 scope); using
                // `status.notify()` gives the warning a TTL so it
                // isn't overwritten by the very next keystroke's
                // status update.
                let tab = &mut self.ui.tabs[self.ui.active_tab];
                let had_secondaries = !tab.editor.secondary_cursors().is_empty();
                let multi_line = text.contains('\n');
                tab.editor.insert_str(&text);
                if had_secondaries && multi_line {
                    self.ui.status.notify(
                        "multi-line paste collapsed secondary cursors",
                        std::time::Duration::from_secs(3),
                    );
                }
            }
            Action::DeleteChar => {
                self.ui.tabs[self.ui.active_tab].editor.delete_char();
            }
            Action::DeleteCharForward => {
                let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
                let before = buf.snapshot();
                if buf.has_selection() {
                    buf.delete_selection();
                } else {
                    buf.delete_char_forward();
                }
                buf.commit_undo_snapshot(before);
            }
            Action::EnterMode(mode) => {
                let buf = &self.ui.tabs[self.ui.active_tab].editor;
                if matches!(mode, Mode::Visual | Mode::VisualLine) {
                    self.ui.tabs[self.ui.active_tab].visual_anchor =
                        Some((buf.cursor_row(), buf.cursor_col()));
                    self.sync_visual_selection();
                } else if self.ui.tabs[self.ui.active_tab]
                    .visual_anchor
                    .take()
                    .is_some()
                {
                    self.ui.tabs[self.ui.active_tab].editor.clear_selection();
                }
                self.ui.status.message = match mode {
                    Mode::Insert => "-- INSERT --".into(),
                    Mode::Normal => "ready".into(),
                    Mode::Command => ":".into(),
                    Mode::Visual => "-- VISUAL --".into(),
                    Mode::VisualLine => "-- V-LINE --".into(),
                    Mode::WaitingForSecondG => "g?".into(),
                    Mode::OperatorPending(op) => format!(
                        "-- {} --",
                        match op {
                            Operator::Delete => "OPERATOR DELETE",
                            Operator::Yank => "OPERATOR YANK",
                            Operator::Change => "OPERATOR CHANGE",
                            // Future operators surface as a generic label.
                            _ => "OPERATOR",
                        }
                    ),
                    // Future modes default to a generic status line.
                    _ => "ready".into(),
                };
            }
            Action::SubmitCommand(cmd) => self.execute_command(&cmd).await,
            Action::Pending if self.ui.vim.mode() == Mode::Command => {
                self.ui.status.message = format!(":{}", self.ui.vim.command_buffer());
            }
            Action::Pending => {}
            Action::PromptComplete => self.complete_prompt().await,
            Action::OpenSearch(dir) => self.open_editor_search(dir).await,
            Action::RepeatSearch => self.repeat_editor_search(false).await,
            Action::RepeatSearchReverse => self.repeat_editor_search(true).await,
            Action::Operate { op, .. }
                if self.ui.tabs[self.ui.active_tab]
                    .visual_anchor
                    .take()
                    .is_some() =>
            {
                self.apply_visual_operator(op);
            }
            Action::Operate { op, motion, count } => {
                self.apply_operator(op, motion, count).await;
            }
            // Future Action variants are silently ignored until wired.
            _ => {}
        }
    }

    /// Reset vim to Normal and drop every tab's visual selection.
    pub(crate) fn reset_vim(&mut self) {
        self.ui.vim = narwhal_vim::Vim::new();
        for tab in &mut self.ui.tabs {
            if tab.visual_anchor.take().is_some() {
                tab.editor.clear_selection();
            }
        }
    }

    /// Rebuild the editor selection from the visual anchor to the
    /// cursor. Vim visual ranges are inclusive, so the far end is
    /// pushed past the character under it.
    fn sync_visual_selection(&mut self) {
        let Some(anchor) = self.ui.tabs[self.ui.active_tab].visual_anchor else {
            return;
        };
        let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
        let cursor = (buf.cursor_row(), buf.cursor_col());
        let sel = if self.ui.vim.mode() == Mode::VisualLine {
            Selection::line(anchor, cursor)
        } else {
            let (start, (row, col)) = if anchor <= cursor {
                (anchor, cursor)
            } else {
                (cursor, anchor)
            };
            let line = buf.lines().get(row).map_or("", String::as_str);
            match line.get(col..).and_then(|rest| rest.chars().next()) {
                Some(c) => Selection::character(start, (row, col + c.len_utf8())),
                None if row + 1 < buf.lines().len() => Selection::character(start, (row + 1, 0)),
                None => Selection::character(start, (row, col)),
            }
        };
        buf.set_selection(Some(sel));
    }

    /// Apply a vim operator to the active visual selection.
    fn apply_visual_operator(&mut self, op: Operator) {
        let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
        let text = buf.selected_text();
        if text.is_empty() {
            buf.clear_selection();
            return;
        }
        let chars = text.chars().count();
        if op == Operator::Yank {
            if let Some(sel) = buf.selection() {
                let (start, _) = sel.normalised();
                buf.set_cursor(start.0, start.1);
            }
            buf.clear_selection();
        } else {
            let before = buf.snapshot();
            buf.delete_selection();
            buf.commit_undo_snapshot(before);
        }
        self.ui.status.message = match self.deps.clipboard.set_text(&text) {
            Err(e) => format!("clipboard error: {e}"),
            Ok(()) if op == Operator::Yank => format!("yanked {chars} character(s)"),
            Ok(()) if op == Operator::Delete => format!("deleted {chars} character(s)"),
            Ok(()) => "-- INSERT --".into(),
        };
    }

    /// Apply a vim operator (delete / yank / change) over the range
    /// described by `(motion, count)`. For line-wise motions the range
    /// spans full lines; for character-wise motions it spans from the
    /// cursor to the position after applying the motion.
    ///
    /// Delete and yank both copy text to the clipboard before mutating
    /// the buffer (consistent with vim's unnamed register). Change does
    /// the same and then enters insert mode — the mode transition has
    /// already been set by the state machine.
    async fn apply_operator(&mut self, op: Operator, motion: VimMotion, count: usize) {
        let buf = &mut self.ui.tabs[self.ui.active_tab].editor;
        let dm = domain_motion(motion);
        if buf.has_multi_cursors() && op == Operator::Delete && dm == DomainMotion::Right {
            let _ = self
                .deps
                .clipboard
                .set_text(&buf.operator_range_text(dm, count));
            let before = buf.snapshot();
            for _ in 0..count {
                buf.delete_char_forward();
            }
            buf.commit_undo_snapshot(before);
            return;
        }

        // Compute the text range affected by the operator.
        let yanked = buf.operator_range_text(dm, count);

        // Yank: copy to clipboard, no deletion.
        if op == Operator::Yank {
            if dm == DomainMotion::FileStart {
                let target = count.saturating_sub(1);
                if target < buf.cursor_row() {
                    buf.set_cursor(target, 0);
                }
            }
            if let Err(e) = self.deps.clipboard.set_text(&yanked) {
                self.ui.status.message = format!("clipboard error: {e}");
            } else {
                let lines = yanked.lines().count();
                let chars = yanked.len();
                if lines > 1 {
                    self.ui.status.message = format!("yanked {lines} line(s)");
                } else {
                    self.ui.status.message = format!("yanked {chars} character(s)");
                }
            }
            return;
        }

        // Delete / Change: copy to clipboard first, then delete.
        if !yanked.is_empty() {
            let _ = self.deps.clipboard.set_text(&yanked);
        }
        buf.apply_operator_delete(dm, count);

        if op == Operator::Delete {
            let lines = yanked.lines().count();
            if lines > 1 {
                self.ui.status.message = format!("deleted {lines} line(s)");
            } else {
                self.ui.status.message = "deleted".into();
            }
        }
        // Change: the vim state machine already transitioned to Insert;
        // no additional status needed — the mode indicator handles it.
    }
}
