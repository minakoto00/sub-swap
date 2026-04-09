pub mod widgets;
pub mod wizard;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;

use crate::config::{AppConfig, KeyBackend};
use crate::crypto::keychain::OsKeyStore;
use crate::error::{Result, SubSwapError};
use crate::guard::{CodexGuard, OsGuard};
use crate::paths::Paths;
use crate::profile::store::ProfileStore;
use crate::profile::switch;
use widgets::{Action, AppScreen, AppState};

// ── Public entry point ────────────────────────────────────────────────────────

pub fn run_tui(paths: &Paths) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = run_app(&mut terminal, paths);

    // Always clean up even if run_app returned an error
    let _ = disable_raw_mode();
    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    run_result
}

// ── Main event loop ───────────────────────────────────────────────────────────

fn run_app<B: Backend>(terminal: &mut Terminal<B>, paths: &Paths) -> Result<()> {
    let store = ProfileStore::load(paths)?;
    let mut state = AppState::from_index(&store.index);

    loop {
        // Reload store on each iteration to reflect any changes
        let store = ProfileStore::load(paths)?;

        terminal
            .draw(|f| render(f, &state, &store))
            .map_err(|e| SubSwapError::Io(std::io::Error::other(e.to_string())))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match state.screen {
                AppScreen::Main => handle_main(&mut state, paths, key.code)?,
                AppScreen::ConfirmSwitch => {
                    handle_confirm_switch(&mut state, paths, key.code)?;
                }
                AppScreen::ForceSwitch => {
                    handle_force_switch(&mut state, paths, key.code)?;
                }
                AppScreen::ConfirmDelete => {
                    handle_confirm_delete(&mut state, paths, key.code)?;
                }
                AppScreen::InputName => {
                    handle_input_name(&mut state, paths, key.code)?;
                }
                AppScreen::InputNote => {
                    handle_input_note(&mut state, paths, key.code)?;
                }
                AppScreen::InputPassphrase => {
                    handle_input_passphrase(&mut state, paths, key.code)?;
                }
                AppScreen::ViewDecrypt => {
                    handle_view_decrypt(&mut state, key.code);
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    Ok(())
}

// ── Key handlers ──────────────────────────────────────────────────────────────

fn handle_main(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_up();
            state.message = None;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_down();
            state.message = None;
        }
        KeyCode::Enter => {
            if state.selected_name().is_some() {
                state.pending_action = Some(Action::Switch);
                state.screen = AppScreen::ConfirmSwitch;
                state.message = None;
            }
        }
        KeyCode::Char('a') => {
            state.pending_action = Some(Action::Add);
            state.input_buffer = String::new();
            state.screen = AppScreen::InputName;
            state.message = None;
        }
        KeyCode::Char('r') => {
            if state.selected_name().is_some() {
                state.pending_action = Some(Action::Rename);
                state.input_buffer = String::new();
                state.screen = AppScreen::InputName;
                state.message = None;
            }
        }
        KeyCode::Char('d') => {
            if state.selected_name().is_some() {
                state.pending_action = Some(Action::Delete);
                state.screen = AppScreen::ConfirmDelete;
                state.message = None;
            }
        }
        KeyCode::Char('n') => {
            if state.selected_name().is_some() {
                state.pending_action = Some(Action::Note);
                state.input_buffer = String::new();
                state.screen = AppScreen::InputNote;
                state.message = None;
            }
        }
        KeyCode::Char('v') => {
            if state.selected_name().is_some() {
                let Some(config) = load_runtime_config_or_prompt(paths, state, Action::View, None)?
                else {
                    return Ok(());
                };
                handle_view(state, paths, &config)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn begin_passphrase_prompt(state: &mut AppState, action: Action, staged_input: Option<String>) {
    state.pending_action = Some(action);
    state.passphrase_buffer.clear();
    state.staged_input = staged_input;
    state.screen = AppScreen::InputPassphrase;
}

fn cancel_passphrase_prompt(state: &mut AppState) {
    state.passphrase_buffer.clear();
    state.staged_input = None;
    state.screen = AppScreen::Main;
}

fn load_runtime_config_or_prompt(
    paths: &Paths,
    state: &mut AppState,
    action: Action,
    staged_input: Option<String>,
) -> Result<Option<AppConfig>> {
    let config = AppConfig::load(paths)?;
    if matches!(config.key_backend, Some(KeyBackend::Passphrase))
        && std::env::var("SUB_SWAP_PASSPHRASE").is_err()
    {
        begin_passphrase_prompt(state, action, staged_input);
        return Ok(None);
    }
    Ok(Some(config))
}

fn resolve_runtime_key(config: &AppConfig, state: &AppState) -> Result<[u8; 32]> {
    let passphrase = std::env::var("SUB_SWAP_PASSPHRASE")
        .ok()
        .or_else(|| Some(state.passphrase_buffer.clone()));
    crate::key::resolve_key(config, &OsKeyStore::new(), passphrase.as_deref())
}

fn handle_view(state: &mut AppState, paths: &Paths, config: &AppConfig) -> Result<()> {
    let Some(n) = state.selected_name() else {
        return Ok(());
    };
    let name = n.to_string();

    let key = resolve_runtime_key(config, state)?;

    match switch::decrypt_profile_to_stdout(paths, &name, &key) {
        Ok((auth_str, config_str)) => {
            let output =
                format!("=== auth.json ===\n{auth_str}\n=== config.toml ===\n{config_str}");
            state.decrypt_output = Some(output);
            state.scroll_offset = 0;
            state.screen = AppScreen::ViewDecrypt;
            state.pending_action = None;
        }
        Err(e) => {
            state.message = Some(format!("Error: {e}"));
            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
    }

    Ok(())
}

fn handle_view_decrypt(state: &mut AppState, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll_offset = state.scroll_offset.saturating_add(1);
        }
        KeyCode::PageUp => {
            state.scroll_offset = state.scroll_offset.saturating_sub(20);
        }
        KeyCode::PageDown => {
            state.scroll_offset = state.scroll_offset.saturating_add(20);
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.scroll_offset = 0;
        }
        _ => {}
    }
}

fn handle_confirm_switch(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('y') | KeyCode::Enter => {
            let Some(n) = state.selected_name() else {
                state.screen = AppScreen::Main;
                return Ok(());
            };
            let name = n.to_string();

            // Check process guard
            let guard = OsGuard::new();
            match guard.check() {
                Err(SubSwapError::CodexRunning(_)) => {
                    // Show ForceSwitch confirmation instead
                    state.screen = AppScreen::ForceSwitch;
                    state.message = Some(
                        "Codex is running! Press y to force switch anyway, n to cancel."
                            .to_string(),
                    );
                    return Ok(());
                }
                Err(e) => return Err(e),
                Ok(()) => {}
            }

            let Some(config) = load_runtime_config_or_prompt(paths, state, Action::Switch, None)?
            else {
                return Ok(());
            };
            do_switch(state, paths, &name, &config)?;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_force_switch(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('y') | KeyCode::Enter => {
            let Some(n) = state.selected_name() else {
                state.screen = AppScreen::Main;
                return Ok(());
            };
            let name = n.to_string();
            let Some(config) = load_runtime_config_or_prompt(paths, state, Action::Switch, None)?
            else {
                return Ok(());
            };
            do_switch(state, paths, &name, &config)?;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.pending_action = None;
            state.message = Some("Switch cancelled.".to_string());
        }
        _ => {}
    }
    Ok(())
}

fn do_switch(state: &mut AppState, paths: &Paths, name: &str, config: &AppConfig) -> Result<()> {
    let key = resolve_runtime_key(config, state)?;

    match switch::switch_profile(paths, name, &key, config.encryption_enabled) {
        Ok(()) => {
            // Reload profile names from store
            let store = ProfileStore::load(paths)?;
            state.profile_names = store
                .index
                .names()
                .into_iter()
                .map(ToString::to_string)
                .collect();
            state.active_profile.clone_from(&store.index.active_profile);
            state.message = Some(format!("Switched to '{name}'."));
        }
        Err(e) => {
            state.message = Some(format!("Error: {e}"));
        }
    }

    state.screen = AppScreen::Main;
    state.pending_action = None;
    Ok(())
}

fn handle_confirm_delete(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('y') => {
            let Some(n) = state.selected_name() else {
                state.screen = AppScreen::Main;
                return Ok(());
            };
            let name = n.to_string();

            let mut store = ProfileStore::load(paths)?;
            match store.index.remove(&name) {
                Ok(_) => {
                    let _ = ProfileStore::delete_profile_dir(paths, &name);
                    store.save(paths)?;

                    // Reload state
                    let store = ProfileStore::load(paths)?;
                    state.profile_names = store
                        .index
                        .names()
                        .into_iter()
                        .map(ToString::to_string)
                        .collect();
                    state.active_profile.clone_from(&store.index.active_profile);

                    // Clamp selection
                    if state.selected >= state.profile_names.len()
                        && !state.profile_names.is_empty()
                    {
                        state.selected = state.profile_names.len() - 1;
                    }
                    state.message = Some(format!("Deleted '{name}'."));
                }
                Err(e) => {
                    state.message = Some(format!("Error: {e}"));
                }
            }

            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_name(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Enter => {
            let name = state.input_buffer.trim().to_string();
            if name.is_empty() {
                state.screen = AppScreen::Main;
                state.pending_action = None;
                return Ok(());
            }

            match state.pending_action {
                Some(Action::Add) => {
                    let Some(config) = load_runtime_config_or_prompt(
                        paths,
                        state,
                        Action::Add,
                        Some(name.clone()),
                    )?
                    else {
                        return Ok(());
                    };
                    do_add_profile(state, paths, &name, &config)?;
                }
                Some(Action::Rename) => {
                    if let Err(e) = crate::error::validate_profile_name(&name) {
                        state.message = Some(format!("Error: {e}"));
                        state.screen = AppScreen::Main;
                        state.pending_action = None;
                        return Ok(());
                    }
                    let Some(n) = state.selected_name() else {
                        state.screen = AppScreen::Main;
                        state.pending_action = None;
                        return Ok(());
                    };
                    let old_name = n.to_string();

                    let mut store = ProfileStore::load(paths)?;
                    match store.index.rename(&old_name, &name) {
                        Ok(()) => {
                            let _ = ProfileStore::rename_profile_dir(paths, &old_name, &name);
                            store.save(paths)?;

                            let store = ProfileStore::load(paths)?;
                            state.profile_names = store
                                .index
                                .names()
                                .into_iter()
                                .map(ToString::to_string)
                                .collect();
                            state.active_profile.clone_from(&store.index.active_profile);
                            state.message = Some(format!("Renamed '{old_name}' to '{name}'."));
                        }
                        Err(e) => {
                            state.message = Some(format!("Error: {e}"));
                        }
                    }
                }
                _ => {}
            }

            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
        KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.pending_action = None;
            state.input_buffer = String::new();
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_note(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Enter => {
            let note_text = state.input_buffer.trim().to_string();
            let Some(n) = state.selected_name() else {
                state.screen = AppScreen::Main;
                state.pending_action = None;
                return Ok(());
            };
            let name = n.to_string();

            let mut store = ProfileStore::load(paths)?;
            let note = if note_text.is_empty() {
                None
            } else {
                Some(note_text)
            };

            match store.index.set_note(&name, note) {
                Ok(()) => {
                    store.save(paths)?;
                    state.message = Some(format!("Note updated for '{name}'."));
                }
                Err(e) => {
                    state.message = Some(format!("Error: {e}"));
                }
            }

            state.screen = AppScreen::Main;
            state.pending_action = None;
        }
        KeyCode::Esc => {
            state.screen = AppScreen::Main;
            state.pending_action = None;
            state.input_buffer = String::new();
        }
        KeyCode::Backspace => {
            state.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            state.input_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn handle_input_passphrase(state: &mut AppState, paths: &Paths, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Enter => {
            let action = state.pending_action;
            let staged_input = state.staged_input.clone();
            let result = (|| -> Result<()> {
                let config = AppConfig::load(paths)?;
                match action {
                    Some(Action::Switch) => {
                        let Some(n) = state.selected_name() else {
                            state.screen = AppScreen::Main;
                            state.pending_action = None;
                            return Ok(());
                        };
                        let name = n.to_string();
                        do_switch(state, paths, &name, &config)
                    }
                    Some(Action::Add) => {
                        let Some(name) = staged_input.as_deref() else {
                            state.screen = AppScreen::Main;
                            state.pending_action = None;
                            return Ok(());
                        };
                        do_add_profile(state, paths, name, &config)
                    }
                    Some(Action::View) => handle_view(state, paths, &config),
                    _ => {
                        state.screen = AppScreen::Main;
                        state.pending_action = None;
                        Ok(())
                    }
                }
            })();

            state.passphrase_buffer.clear();
            state.staged_input = None;

            if let Err(e) = result {
                state.screen = AppScreen::Main;
                state.pending_action = None;
                state.message = Some(format!("Error: {e}"));
            }
        }
        KeyCode::Esc => {
            cancel_passphrase_prompt(state);
            state.pending_action = None;
        }
        KeyCode::Backspace => {
            state.passphrase_buffer.pop();
        }
        KeyCode::Char(c) => {
            state.passphrase_buffer.push(c);
        }
        _ => {}
    }
    Ok(())
}

fn do_add_profile(
    state: &mut AppState,
    paths: &Paths,
    name: &str,
    config: &AppConfig,
) -> Result<()> {
    let key = resolve_runtime_key(config, state)?;
    let mut store = ProfileStore::load_or_init(paths)?;

    match switch::add_profile_from_codex(
        paths,
        &mut store,
        name,
        None,
        &key,
        config.encryption_enabled,
    ) {
        Ok(()) => {
            let store = ProfileStore::load(paths)?;
            state.profile_names = store
                .index
                .names()
                .into_iter()
                .map(ToString::to_string)
                .collect();
            state.active_profile.clone_from(&store.index.active_profile);
            state.message = Some(format!("Added '{name}'."));
        }
        Err(e) => {
            state.message = Some(format!("Error: {e}"));
        }
    }

    state.screen = AppScreen::Main;
    state.pending_action = None;
    Ok(())
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render(f: &mut Frame, state: &AppState, store: &ProfileStore) {
    let area = f.area();

    match state.screen {
        AppScreen::ViewDecrypt => {
            render_decrypt_view(f, state, area);
        }
        AppScreen::InputName => {
            render_main_layout(f, state, store, area);
            render_input_overlay(f, state, area, "Enter profile name: ");
        }
        AppScreen::InputNote => {
            render_main_layout(f, state, store, area);
            render_input_overlay(
                f,
                state,
                area,
                "Enter note (Enter to save, Esc to cancel): ",
            );
        }
        AppScreen::InputPassphrase => {
            render_main_layout(f, state, store, area);
            render_passphrase_overlay(f, state, area);
        }
        AppScreen::ConfirmSwitch => {
            render_main_layout(f, state, store, area);
            render_confirm_overlay(f, state, area, "Switch to this profile? [y/n]: ");
        }
        AppScreen::ForceSwitch => {
            render_main_layout(f, state, store, area);
            let msg = state
                .message
                .as_deref()
                .unwrap_or("Codex is running. Switch anyway? [y/n]: ");
            render_confirm_overlay(f, state, area, msg);
        }
        AppScreen::ConfirmDelete => {
            render_main_layout(f, state, store, area);
            render_confirm_overlay(f, state, area, "Delete this profile? [y/n]: ");
        }
        AppScreen::Main => {
            render_main_layout(f, state, store, area);
        }
    }
}

fn render_decrypt_view(f: &mut Frame, state: &AppState, area: Rect) {
    let content = state
        .decrypt_output
        .as_deref()
        .unwrap_or("No decrypted content available. Press any key to return.");

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title("Decrypted Profile (↑↓ scroll, q/Esc back)")
                .borders(Borders::ALL),
        )
        .wrap(ratatui::widgets::Wrap { trim: false })
        .scroll((state.scroll_offset, 0));

    f.render_widget(paragraph, area);
}

fn render_main_layout(f: &mut Frame, state: &AppState, store: &ProfileStore, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    render_profile_list(f, state, store, chunks[0]);
    render_key_hints(f, chunks[1]);
    render_status_bar(f, state, chunks[2]);
}

fn render_profile_list(f: &mut Frame, state: &AppState, store: &ProfileStore, area: Rect) {
    let items: Vec<ListItem> = state
        .profile_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == state.selected;
            let is_active = state.active_profile.as_deref() == Some(name.as_str());

            let cursor = if is_selected { ">" } else { " " };
            let marker = if is_active { "*" } else { " " };
            let note_suffix = store
                .index
                .get(name)
                .and_then(|p| p.notes.as_deref())
                .map_or(String::new(), |n| format!(" — {n}"));
            let label = format!("{cursor} {marker} {name}{note_suffix}");

            let style = if is_selected && is_active {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Yellow)
            } else if is_active {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("sub-swap — Profiles")
            .borders(Borders::ALL),
    );

    f.render_widget(list, area);
}

fn render_key_hints(f: &mut Frame, area: Rect) {
    let hints = "Enter: switch  a: add  r: rename  d: delete  n: note  v: view  q: quit";
    let paragraph =
        Paragraph::new(hints).block(Block::default().title("Keys").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, state: &AppState, area: Rect) {
    let msg = state.message.as_deref().unwrap_or("");
    let paragraph =
        Paragraph::new(msg).block(Block::default().title("Status").borders(Borders::ALL));
    f.render_widget(paragraph, area);
}

fn render_input_overlay(f: &mut Frame, state: &AppState, area: Rect, prompt: &str) {
    render_value_overlay(f, area, prompt, &state.input_buffer, Color::White);
}

fn render_passphrase_overlay(f: &mut Frame, state: &AppState, area: Rect) {
    let masked = "*".repeat(state.passphrase_buffer.chars().count());
    render_value_overlay(
        f,
        area,
        "Enter passphrase (Enter to submit, Esc to cancel): ",
        &masked,
        Color::White,
    );
}

fn render_value_overlay(f: &mut Frame, area: Rect, prompt: &str, value: &str, color: Color) {
    let input_area = centered_rect(60, 5, area);
    let content = format!("{prompt}{value}");
    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(color));
    f.render_widget(paragraph, input_area);
}

fn render_confirm_overlay(f: &mut Frame, _state: &AppState, area: Rect, prompt: &str) {
    let confirm_area = centered_rect(50, 5, area);
    let paragraph = Paragraph::new(prompt)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(paragraph, confirm_area);
}

/// Create a centered rectangle with given percentage width and fixed height.
fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical_chunks[1]);

    horizontal_chunks[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{Profile, ProfileIndex};
    use widgets::AppState;

    fn state_in_decrypt_view() -> AppState {
        let mut index = ProfileIndex::default();
        index.add(Profile::new("test", None));
        let mut state = AppState::from_index(&index);
        state.screen = AppScreen::ViewDecrypt;
        state.decrypt_output = Some("line1\nline2\nline3".to_string());
        state
    }

    #[test]
    fn test_scroll_down_increments_offset() {
        let mut state = state_in_decrypt_view();
        handle_view_decrypt(&mut state, KeyCode::Down);
        assert_eq!(state.scroll_offset, 1);
        assert_eq!(state.screen, AppScreen::ViewDecrypt);
    }

    #[test]
    fn test_scroll_up_decrements_offset() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 5;
        handle_view_decrypt(&mut state, KeyCode::Up);
        assert_eq!(state.scroll_offset, 4);
    }

    #[test]
    fn test_scroll_up_saturates_at_zero() {
        let mut state = state_in_decrypt_view();
        assert_eq!(state.scroll_offset, 0);
        handle_view_decrypt(&mut state, KeyCode::Up);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_page_down_scrolls_20_lines() {
        let mut state = state_in_decrypt_view();
        handle_view_decrypt(&mut state, KeyCode::PageDown);
        assert_eq!(state.scroll_offset, 20);
    }

    #[test]
    fn test_page_up_scrolls_20_lines() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 25;
        handle_view_decrypt(&mut state, KeyCode::PageUp);
        assert_eq!(state.scroll_offset, 5);
    }

    #[test]
    fn test_page_up_saturates_at_zero() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 10;
        handle_view_decrypt(&mut state, KeyCode::PageUp);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_vim_keys_scroll() {
        let mut state = state_in_decrypt_view();
        handle_view_decrypt(&mut state, KeyCode::Char('j'));
        assert_eq!(state.scroll_offset, 1);
        handle_view_decrypt(&mut state, KeyCode::Char('k'));
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_q_exits_and_resets_scroll() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 10;
        handle_view_decrypt(&mut state, KeyCode::Char('q'));
        assert_eq!(state.screen, AppScreen::Main);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_esc_exits_and_resets_scroll() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 5;
        handle_view_decrypt(&mut state, KeyCode::Esc);
        assert_eq!(state.screen, AppScreen::Main);
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_unhandled_key_does_not_change_state() {
        let mut state = state_in_decrypt_view();
        state.scroll_offset = 3;
        handle_view_decrypt(&mut state, KeyCode::Char('x'));
        assert_eq!(state.scroll_offset, 3);
        assert_eq!(state.screen, AppScreen::ViewDecrypt);
    }

    #[test]
    fn test_begin_passphrase_prompt_switches_screen() {
        let mut index = ProfileIndex::default();
        index.add(Profile::new("test", None));
        let mut state = AppState::from_index(&index);

        begin_passphrase_prompt(&mut state, Action::View, None);

        assert_eq!(state.screen, AppScreen::InputPassphrase);
        assert_eq!(state.pending_action, Some(Action::View));
    }

    #[test]
    fn test_cancel_passphrase_prompt_clears_buffer() {
        let mut index = ProfileIndex::default();
        index.add(Profile::new("test", None));
        let mut state = AppState::from_index(&index);

        begin_passphrase_prompt(&mut state, Action::View, None);
        state.passphrase_buffer = "secret".to_string();
        cancel_passphrase_prompt(&mut state);

        assert_eq!(state.screen, AppScreen::Main);
        assert!(state.passphrase_buffer.is_empty());
    }
}
