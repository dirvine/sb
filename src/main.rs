use anyhow::Result;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ratatui::crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use tui_textarea::TextArea;

mod app;
use app::*;
mod preview;
use preview::*;
mod fs;
mod git;

// Ensures terminal is restored even if the app panics or exits abruptly
struct TermGuard;
impl Drop for TermGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

fn main() -> Result<()> {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?);
    let mut app = App::new(root)?;
    run(&mut app)
}

fn run(app: &mut App) -> Result<()> {
    // Create a guard to always restore terminal state on exit/panic
    let _tg = TermGuard;
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main loop
    loop {
        terminal.draw(|f| ui(f, app))?;
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(k) => {
                    if app.creating_file {
                        match (k.code, k.modifiers) {
                            (KeyCode::Enter, _) => {
                                let _ = app.confirm_create_file();
                            }
                            (KeyCode::Esc, _) => app.cancel_create_file(),
                            _ => {
                                let _ = app.filename_input.input(k);
                            }
                        }
                        continue;
                    }
                    if app.picking_file {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => app.picker_cancel(),
                            (KeyCode::Enter, _) => {
                                let _ = app.picker_activate();
                            }
                            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.picker_up(),
                            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.picker_down(),
                            // New Git-aware file picker commands
                            (KeyCode::Char('d') | KeyCode::Char('D'), _) => {
                                let _ = app.picker_delete_with_git_check();
                            }
                            (KeyCode::Char('m') | KeyCode::Char('M'), _) => {
                                let _ = app.picker_start_move();
                            }
                            (KeyCode::Char('p') | KeyCode::Char('P'), _) => {
                                let _ = app.picker_parent_dir();
                            }
                            (KeyCode::Char('s') | KeyCode::Char('S'), _) => {
                                app.picker_show_git_status();
                            }
                            (KeyCode::Char('o') | KeyCode::Char('O'), _) => {
                                // Open selected file in external editor
                                if app.picker_index < app.picker_items.len() {
                                    let path = &app.picker_items[app.picker_index];
                                    let _ = opener::open(path);
                                    app.status = format!("Opened {} externally", path.display());
                                }
                            }
                            (KeyCode::Char(c), _) => {
                                // Debug: show what key was pressed
                                app.status = format!("File picker: Unknown key '{}'", c);
                            }
                            _ => {}
                        }
                        continue;
                    }
                    // Handle move destination picker
                    if app.showing_move_dest {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => app.cancel_move(),
                            (KeyCode::Enter, _) => {
                                let _ = app.confirm_move();
                            }
                            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_dest_up(),
                            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_dest_down(),
                            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                                let _ = app.move_dest_enter();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    // Handle Git status display
                    if app.showing_git_status {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) | (KeyCode::Enter, _) | (KeyCode::Char('s'), _) => {
                                app.close_git_status();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    if !matches!(app.op_mode, app::OpMode::None) {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => app.cancel_op(),
                            (KeyCode::Enter, _) => {
                                let _ = app.confirm_op();
                            }
                            _ => {
                                let _ = app.op_input.input(k);
                            }
                        }
                        continue;
                    }
                    // Inline line edit in Preview
                    if app.editing_line {
                        match (k.code, k.modifiers) {
                            (KeyCode::Enter, _) => {
                                app.confirm_line_edit();
                            }
                            (KeyCode::Esc, _) => app.cancel_line_edit(),
                            _ => {
                                let _ = app.line_input.input(k);
                            }
                        }
                        continue;
                    }
                    // Editor command mode
                    if app.editor_cmd_mode {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => app.cancel_editor_cmd(),
                            (KeyCode::Enter, _) => {
                                let _ = app.confirm_editor_cmd();
                            }
                            _ => {
                                let _ = app.editor_cmd_input.input(k);
                            }
                        }
                        continue;
                    }
                    // Raw edit mode in Preview: route keys to the text editor; Esc exits back to preview
                    if app.show_raw_editor {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _) => {
                                app.show_raw_editor = false;
                                app.prefer_raw_editor = false;
                            }
                            (KeyCode::Tab, _) => {
                                // Temporarily exit raw editor but remember preference
                                app.show_raw_editor = false;
                                app.prefer_raw_editor = true;
                                // Don't continue - let Tab be handled by the normal focus switching below
                            }
                            _ => {
                                if !app.editor_cmd_mode {
                                    app.editor.input(k);
                                }
                                continue;
                            }
                        }
                        // Only continue if we didn't handle Tab
                        if k.code != KeyCode::Tab {
                            continue;
                        }
                    }
                    if app.show_help {
                        match (k.code, k.modifiers) {
                            (KeyCode::Esc, _)
                            | (KeyCode::Char('?'), _)
                            | (KeyCode::Char('h'), _) => app.toggle_help(),
                            _ => {}
                        }
                        continue;
                    }
                    if app.confirming_delete {
                        match k.code {
                            KeyCode::Enter | KeyCode::Char('d') => {
                                let _ = app.confirm_delete_with_git();
                            }
                            KeyCode::Esc => app.cancel_delete(),
                            _ => {}
                        }
                        continue;
                    }
                    // Debug already at top, don't duplicate

                    match (k.code, k.modifiers) {
                        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                            break
                        }
                        (KeyCode::Esc, _) => {
                            if matches!(app.focus, Focus::Preview) {
                                break;
                            }
                        }
                        (KeyCode::Tab, mods) => {
                            // Check if this is Ctrl+I (which sends Tab with CONTROL modifier)
                            if mods.contains(KeyModifiers::CONTROL) {
                                // This is actually Ctrl+I for file picker
                                let _ = app.begin_file_picker();
                            } else if app.show_left_pane {
                                // Tab between left pane and right pane (in whatever mode it's in)
                                app.focus = match app.focus {
                                    Focus::Left => {
                                        // Moving to right pane - decide which mode based on preference
                                        if app.prefer_raw_editor && app.opened.is_some() {
                                            Focus::Editor
                                        } else {
                                            Focus::Preview
                                        }
                                    }
                                    Focus::Preview | Focus::Editor => Focus::Left,
                                };
                                // Restore raw editor mode if user was in it and we're now on Editor focus
                                if matches!(app.focus, Focus::Editor) && app.prefer_raw_editor {
                                    app.show_raw_editor = true;
                                }
                            }
                            // If left pane is hidden, Tab does nothing (only one pane visible)
                        }
                        (KeyCode::BackTab, _) => {
                            if app.show_left_pane {
                                // BackTab between left pane and right pane (same as Tab since only 2 panes)
                                app.focus = match app.focus {
                                    Focus::Left => {
                                        // Moving to right pane - decide which mode based on preference
                                        if app.prefer_raw_editor && app.opened.is_some() {
                                            Focus::Editor
                                        } else {
                                            Focus::Preview
                                        }
                                    }
                                    Focus::Preview | Focus::Editor => Focus::Left,
                                };
                                // Restore raw editor mode if user was in it and we're now on Editor focus
                                if matches!(app.focus, Focus::Editor) && app.prefer_raw_editor {
                                    app.show_raw_editor = true;
                                }
                            }
                            // If left pane is hidden, BackTab does nothing (only one pane visible)
                        }
                        // 'p' previously toggled preview; now preview is always on, so ignore or repurpose later
                        (KeyCode::Char('?'), _) => app.toggle_help(),
                        // Add explicit handler for 'i' with Control modifier as fallback
                        (KeyCode::Char('i'), mods) if mods.contains(KeyModifiers::CONTROL) => {
                            let _ = app.begin_file_picker();
                        }
                        (KeyCode::Char('n'), _) if matches!(app.focus, Focus::Left) => {
                            app.begin_create_file()
                        }
                        (KeyCode::Char('d'), _) if matches!(app.focus, Focus::Left) => {
                            app.begin_delete()
                        }
                        // Note: Ctrl+I is handled above in the Tab handler because terminals
                        // send Tab for Ctrl+I (historical terminal convention)
                        (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                            let _ = app.save();
                        }
                        (KeyCode::Char('o'), _)
                            if matches!(app.focus, Focus::Left | Focus::Editor) =>
                        {
                            let _ = app.open_externally();
                        }
                        // Editor command mode (minimal)
                        (KeyCode::Char(':'), _) => {
                            if matches!(app.focus, Focus::Editor) {
                                app.begin_editor_cmd();
                            }
                        }
                        // Video controls
                        (KeyCode::Char(' '), _) => {
                            app.toggle_pause_video();
                        }
                        (KeyCode::Char('s'), _) => {
                            app.stop_video();
                        }
                        (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                            app.autoplay_video = !app.autoplay_video;
                            app.status = format!(
                                "Autoplay video {}",
                                if app.autoplay_video { "on" } else { "off" }
                            );
                        }
                        // Toggle left pane
                        (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                            app.toggle_left_pane();
                        }
                        (KeyCode::F(9), _) => {
                            app.toggle_left_pane();
                        }
                        // F-keys like MC
                        (KeyCode::F(2), _) => {
                            // F2 as alternative to Ctrl+I for file picker
                            let _ = app.begin_file_picker();
                        }
                        (KeyCode::F(3), _) => { /* Quick view handled by preview always-on */ }
                        (KeyCode::F(4), _) => {
                            app.focus = Focus::Editor;
                            // Restore raw editor mode if user prefers it
                            if app.prefer_raw_editor {
                                app.show_raw_editor = true;
                            }
                        }
                        (KeyCode::F(5), _) => {
                            app.begin_copy();
                        }
                        (KeyCode::F(6), _) => {
                            app.begin_move();
                        }
                        (KeyCode::F(7), _) => {
                            app.begin_mkdir();
                        }
                        (KeyCode::F(8), _) => {
                            app.begin_delete();
                        }
                        (KeyCode::F(10), _) => break,
                        (KeyCode::Enter, _) => {
                            match app.focus {
                                Focus::Left => {
                                    let _ = app.activate_on_tree();
                                }
                                Focus::Editor => { /* enter does nothing special here */ }
                                Focus::Preview => {}
                            }
                        }
                        // Toggle files pane with 'h' when in Preview
                        (KeyCode::Char('h'), _) => {
                            if matches!(app.focus, Focus::Preview) {
                                app.toggle_left_pane();
                            }
                        }
                        (KeyCode::Char('e'), _) => {
                            if matches!(app.focus, Focus::Preview) {
                                app.show_raw_editor = true;
                                app.prefer_raw_editor = true;
                            }
                        }
                        (KeyCode::Up, _)
                        | (KeyCode::Down, _)
                        | (KeyCode::Left, _)
                        | (KeyCode::Right, _)
                        | (KeyCode::Char('j'), _)
                        | (KeyCode::Char('k'), _) => match app.focus {
                            Focus::Left => match k.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    let _ = app.left_state.key_up();
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let _ = app.left_state.key_down();
                                }
                                KeyCode::Left => {
                                    // Try to collapse tree node
                                    let _ = app.left_state.key_left();
                                }
                                KeyCode::Right => {
                                    // Check if current selection is a file (can't be expanded)
                                    let is_file = app
                                        .left_state
                                        .selected()
                                        .first()
                                        .and_then(|s| Path::new(s).to_str())
                                        .map(|s| Path::new(s).is_file())
                                        .unwrap_or(false);

                                    if is_file {
                                        // If it's a file, switch to preview/editor pane
                                        app.focus = if app.prefer_raw_editor && app.opened.is_some()
                                        {
                                            Focus::Editor
                                        } else {
                                            Focus::Preview
                                        };
                                        // Restore raw editor mode if needed
                                        if matches!(app.focus, Focus::Editor)
                                            && app.prefer_raw_editor
                                        {
                                            app.show_raw_editor = true;
                                        }
                                    } else {
                                        // If it's a directory, try to expand it
                                        let _ = app.left_state.key_right();
                                    }
                                }
                                _ => {}
                            },
                            Focus::Editor => {
                                if !app.editor_cmd_mode {
                                    app.editor.input(k);
                                }
                            }
                            Focus::Preview => match k.code {
                                KeyCode::Up | KeyCode::Char('k') => app.move_cursor_up(),
                                KeyCode::Down | KeyCode::Char('j') => app.move_cursor_down(),
                                KeyCode::Left => {
                                    // In preview mode, left arrow switches to left pane if visible
                                    if app.show_left_pane {
                                        app.focus = Focus::Left;
                                    } else {
                                        app.move_col_left();
                                    }
                                }
                                KeyCode::Right => {
                                    // In preview mode, right arrow just scrolls horizontally
                                    app.move_col_right();
                                }
                                _ => {}
                            },
                        },
                        _ => {}
                    }
                }
                Event::Mouse(me) => match me.kind {
                    MouseEventKind::ScrollDown => {
                        if app.show_raw_editor {
                            for _ in 0..3 {
                                let _ = app
                                    .editor
                                    .input(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
                            }
                        } else if matches!(app.focus, Focus::Preview) {
                            for _ in 0..3 {
                                app.move_cursor_down();
                            }
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        if app.show_raw_editor {
                            for _ in 0..3 {
                                let _ = app
                                    .editor
                                    .input(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
                            }
                        } else if matches!(app.focus, Focus::Preview) {
                            for _ in 0..3 {
                                app.move_cursor_up();
                            }
                        }
                    }
                    MouseEventKind::Down(_) => {
                        // Mouse click to change focus based on click position
                        let _chunks = Layout::default()
                            .direction(Direction::Horizontal)
                            .constraints(if app.show_left_pane {
                                [Constraint::Length(30), Constraint::Min(40)]
                            } else {
                                [Constraint::Length(0), Constraint::Min(40)]
                            })
                            .split(Rect {
                                x: 0,
                                y: 0,
                                width: 100, // Terminal dimensions will be different, but proportions matter
                                height: 50,
                            });

                        if app.show_left_pane && me.column < 30 {
                            // Click in left pane
                            app.focus = Focus::Left;
                        } else if me.column >= (if app.show_left_pane { 30 } else { 0 }) {
                            // Click in right pane - determine if in editor mode
                            if app.prefer_raw_editor {
                                app.focus = Focus::Editor;
                                app.show_raw_editor = true;
                            } else {
                                app.focus = Focus::Preview;
                            }
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {}
                Event::FocusGained | Event::FocusLost | Event::Paste(_) => {}
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    // First split vertically to reserve space for status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Main content area
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Then split the main area horizontally for file tree and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.show_left_pane {
            [Constraint::Length(30), Constraint::Min(40)]
        } else {
            [Constraint::Length(0), Constraint::Min(40)]
        })
        .split(main_chunks[0]);

    // --- Left pane
    if app.show_left_pane {
        let left_border = if matches!(app.focus, Focus::Left) {
            Color::Cyan
        } else {
            Color::Blue
        };
        let left_block = Block::default()
            .title("Files")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(left_border));
        let left_tree = tui_tree_widget::Tree::new(&app.left_tree)
            .expect("unique ids")
            .block(left_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        f.render_stateful_widget(left_tree, chunks[0], &mut app.left_state);
    }

    // (Right tree and standalone editor hidden in 2-pane UX)

    // --- Unified preview/editor (right)
    // Prepare current editor buffer and metadata
    let text = app.editor.lines().join("\n");
    // Provide current path/text and preview cursor to preview for code highlighting/diff and raw-line overlay
    if let Some(path) = app.opened.as_ref() {
        std::env::set_var("SB_CURRENT_FILE", path);
        std::env::set_var("SB_CURRENT_TEXT", &text);
        std::env::set_var("SB_PREVIEW_CURSOR", app.preview_cursor.to_string());
        std::env::set_var("SB_PREVIEW_COL", app.preview_col.to_string());
        std::env::set_var("SB_PREVIEW_SCROLL", app.preview_scroll.to_string());
    }
    let preview = if let Some(path) = app.opened.clone() {
        // Check if we should show diff instead of regular preview
        if app.should_show_diff(&path) {
            if let Some(diff_content) = app.get_file_diff(&path) {
                Preview {
                    text: Text::raw(diff_content),
                    images: vec![],
                    videos: vec![],
                }
            } else {
                Preview::from_markdown(&path, &text).unwrap_or_else(|_| Preview {
                    text: Text::raw("(preview error)"),
                    images: vec![],
                    videos: vec![],
                })
            }
        } else {
            Preview::from_markdown(&path, &text).unwrap_or_else(|_| Preview {
                text: Text::raw("(preview error)"),
                images: vec![],
                videos: vec![],
            })
        }
    } else {
        Preview {
            text: Text::raw("(no file)"),
            images: vec![],
            videos: vec![],
        }
    };
    // Auto start/stop video based on first detected link
    if app.autoplay_video {
        if let Some(first) = preview.videos.first() {
            if app.video_path.as_ref() != Some(first) {
                app.start_video(first.clone());
            }
        } else if app.video_player.is_some() {
            app.stop_video();
        }
    }
    // Set preview viewport height (usable rows for text block)
    let preview_text_rows = chunks[1].height.saturating_sub(2) as usize;
    app.preview_viewport = preview_text_rows;
    // Clamp scroll to valid range against file length
    let total_lines = app.editor.lines().len();
    if app.preview_scroll + app.preview_viewport > total_lines.saturating_sub(1) {
        app.preview_scroll = total_lines
            .saturating_sub(app.preview_viewport)
            .saturating_sub(0);
    }
    // Right pane: preview or full raw editor
    if app.show_raw_editor {
        let block = Block::default()
            .title("Edit (raw)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        let area = chunks[1];
        f.render_widget(block.clone(), area);
        let inner = block.inner(area);
        f.render_widget(&app.editor, inner);
    } else {
        preview::render_preview(f, chunks[1], &preview);
    }
    // Editor command mode prompt overlays at bottom when active
    if app.editor_cmd_mode {
        let h = 1;
        let area = Rect {
            x: chunks[1].x + 1,
            y: chunks[1].bottom().saturating_sub(h),
            width: chunks[1].width.saturating_sub(2),
            height: h,
        };
        f.render_widget(Clear, area);
        f.render_widget(&app.editor_cmd_input, area);
    }
    if !app.show_raw_editor && matches!(app.focus, Focus::Preview) && app.editing_line {
        // Draw an inline single-line editor at the bottom of preview as a simple approach
        let h = 3;
        let area = Rect {
            x: chunks[1].x + 1,
            y: chunks[1].bottom().saturating_sub(h),
            width: chunks[1].width.saturating_sub(2),
            height: h,
        };
        let block = Block::default()
            .title(format!(
                "Edit line {} (Enter=save, Esc=cancel)",
                app.preview_cursor + 1
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green));
        f.render_widget(Clear, area);
        f.render_widget(block.clone(), area);
        let inner = block.inner(area);
        f.render_widget(&app.line_input, inner);
    }

    // If a video is playing, overlay the last frame below the text area similar to images
    if !app.show_raw_editor && matches!(app.focus, Focus::Preview) {
        if let Some(vp) = &app.video_player {
            if let Some(img) = vp.last_frame() {
                let picker = ratatui_image::picker::Picker::from_query_stdio()
                    .unwrap_or_else(|_| ratatui_image::picker::Picker::from_fontsize((8, 12)));
                let mut state = picker.new_resize_protocol(img);
                let widget =
                    ratatui_image::StatefulImage::new().resize(ratatui_image::Resize::Fit(None));
                // carve a small area at bottom of preview
                let h = 12;
                let area = Rect {
                    x: chunks[1].x + 1,
                    y: chunks[1].bottom().saturating_sub(h),
                    width: chunks[1].width.saturating_sub(2),
                    height: h,
                };
                f.render_stateful_widget(widget, area, &mut state);
            }
        }
    }

    // Badge: indicate Files pane hidden
    if matches!(app.focus, Focus::Preview) {
        let badge = if app.show_left_pane {
            String::new()
        } else {
            "Files hidden  ·  Ctrl+B/F9".to_string()
        };
        let tw = badge.len() as u16;
        let w = tw.min(chunks[1].width);
        let x = chunks[1].x + chunks[1].width.saturating_sub(w);
        let area = Rect {
            x,
            y: chunks[1].y,
            width: w,
            height: 1,
        };
        f.render_widget(Clear, area);
        let p = Paragraph::new(badge)
            .style(Style::default().fg(Color::DarkGray).bg(Color::Black))
            .alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    // "Q to quit" indicator in top-left corner
    let quit_hint = " Q: Quit ";
    let quit_area = Rect {
        x: 0,
        y: 0,
        width: quit_hint.len() as u16,
        height: 1,
    };
    f.render_widget(Clear, quit_area);
    let quit_widget =
        Paragraph::new(quit_hint).style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(quit_widget, quit_area);

    // Footer hint in Preview to restore Files pane (one row above global status bar)
    if matches!(app.focus, Focus::Preview)
        && !app.show_raw_editor
        && !app.show_left_pane
        && !app.editor_cmd_mode
        && !app.editing_line
    {
        let hint = "Press F9 to show Files pane";
        let y = chunks[1].bottom().saturating_sub(2);
        let x = chunks[1].x + 1;
        let w = chunks[1].width.saturating_sub(2);
        let area = Rect {
            x,
            y,
            width: w,
            height: 1,
        };
        f.render_widget(Clear, area);
        let p = Paragraph::new(hint)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    // --- Status bar (context-sensitive)
    let status_text = match (&app.focus, app.show_raw_editor, app.picking_file) {
        // File picker mode
        (_, _, true) => {
            // File picker has its own status bar, skip main status
            "".to_string()
        }
        // Editor mode
        (Focus::Editor, true, false) | (_, true, false) if app.prefer_raw_editor => {
            format!(
                "EDITOR MODE  │  Ctrl+S save  │  ESC return to preview  │  {}",
                app.status
            )
        }
        // Preview mode with focus
        (Focus::Preview, false, false) => {
            if app.show_left_pane {
                format!(
                    "PREVIEW MODE  │  ↑↓ scroll  ← files  e edit  Ctrl+S save  F2 picker  ? help  │  {}",
                    app.status
                )
            } else {
                format!(
                    "PREVIEW MODE  │  ↑↓ scroll  e edit  Ctrl+S save  F2 picker  ? help  │  {}",
                    app.status
                )
            }
        }
        // File tree focus
        (Focus::Left, _, false) => {
            format!("FILES  │  ↑↓ navigate  → preview  Enter open  D delete  N new  F5 copy  F6 move  F7 mkdir  │  {}", app.status)
        }
        // Default
        _ => {
            format!(
                "Tab focus  │  Ctrl+B toggle files  │  F2 file picker  │  ? help  │  {}",
                app.status
            )
        }
    };

    if !status_text.is_empty() {
        let status = Paragraph::new(status_text)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(ratatui::layout::Alignment::Center);

        // Use the reserved status bar area from main_chunks[1]
        f.render_widget(status, main_chunks[1]);
    }

    // --- Help overlay
    if app.show_help {
        draw_centered_help(f, f.area());
    }

    // --- New file overlay
    if app.creating_file {
        draw_new_file_prompt(f, f.area(), &app.filename_input);
    }

    // --- Delete confirm overlay
    if app.confirming_delete {
        draw_delete_confirm(f, f.area(), app.delete_target.as_deref());
    }

    // --- File picker overlay
    if app.picking_file {
        // Removed debug output that was being called every frame
        draw_file_picker(f, f.area(), app);
    }

    // --- Operation input overlay (Copy/Move/Mkdir)
    if !matches!(app.op_mode, app::OpMode::None) {
        draw_op_input(f, f.area(), app);
    }

    // --- Move destination picker overlay
    if app.showing_move_dest {
        draw_move_destination_picker(f, f.area(), app);
    }

    // --- Git status display overlay
    if app.showing_git_status {
        draw_git_status(f, f.area(), app);
    }
}

fn draw_centered_help(f: &mut Frame, area: Rect) {
    let help = [
        "sb — Markdown TUI",
        "",
        "Focus: Tab / Shift+Tab",
        "Tree: ↑↓←→ or j/k, Enter toggles/open",
        "Editor: type freely (Enter = newline)",
        "New file: N",
        "Delete: d (confirm)",
        "Insert link (picker): F2 or Ctrl+I",
        "Save: Ctrl+S",
        "Open externally: o",
        "Help: ? (toggle)",
        "Quit: Q / Esc",
    ]
    .join("\n");
    let paragraph = Paragraph::new(help)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    let w = area.width.min(60);
    let h = area.height.min(14);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    f.render_widget(Clear, popup);
    f.render_widget(paragraph, popup);
}

fn draw_new_file_prompt(f: &mut Frame, area: Rect, input: &TextArea) {
    let w = area.width.min(60);
    let h = 5;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    let block = Block::default()
        .title("New file name (.md)")
        .borders(Borders::ALL);
    f.render_widget(Clear, popup);
    f.render_widget(block.clone(), popup);
    let inner = block.inner(popup);
    f.render_widget(input, inner);
}

fn draw_delete_confirm(f: &mut Frame, area: Rect, target: Option<&std::path::Path>) {
    // Create a semi-transparent background overlay
    let overlay_block = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(overlay_block, area);

    let w = area.width.min(60);
    let h = 8;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    // Azure-style blue border with white background
    let block = Block::default()
        .title(" ⚠️  Confirm Delete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    f.render_widget(Clear, popup);
    f.render_widget(block.clone(), popup);

    let inner = block.inner(popup);

    // Create content with better spacing
    let file_name = match target {
        Some(p) => p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        None => "selected file".to_string(),
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Are you sure you want to delete "),
            Span::styled(
                format!("'{file_name}'"),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from("This action cannot be undone.").style(Style::default().fg(Color::Red)),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                " Enter ",
                Style::default().fg(Color::Black).bg(Color::Green),
            ),
            Span::raw(" Confirm  "),
            Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Red)),
            Span::raw(" Cancel"),
        ]),
    ];

    let body = Paragraph::new(content).alignment(Alignment::Center);
    f.render_widget(body, inner);
}

fn draw_file_picker(f: &mut Frame, area: Rect, app: &App) {
    // Create centered popup
    let w = 70.min(area.width - 4);
    let h = 25.min(area.height - 4);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    // Clear the area and draw border
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(" File Picker ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    f.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    // Split inner area into list and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // File list
            Constraint::Length(1), // Status bar
        ])
        .split(inner);

    // Create file list
    let items: Vec<ListItem> = app
        .picker_items
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let is_selected = i == app.picker_index;
            let display_name = path.file_name().unwrap_or_default().to_string_lossy();

            let prefix = if path.is_dir() { "📁 " } else { "📄 " };

            // Add Git status if available
            let status_indicator = if let Some(ref repo) = app.git_repo {
                if let Ok(statuses) = repo.status() {
                    if let Some(status) = statuses.get(path) {
                        match status {
                            crate::git::FileStatus::Modified => " [M]",
                            crate::git::FileStatus::Added => " [A]",
                            crate::git::FileStatus::Deleted => " [D]",
                            crate::git::FileStatus::Untracked => " [?]",
                            crate::git::FileStatus::Conflicted => " [C]",
                            _ => "",
                        }
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            } else {
                ""
            };

            let text = format!("{}{}{}", prefix, display_name, status_indicator);

            let style = if is_selected {
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default());

    f.render_widget(list, chunks[0]);

    // Draw status bar at bottom with commands
    let status_text =
        "↑↓:navigate  Enter:select  O:open-external  D:delete  P:parent  S:git-status  ESC:cancel";
    let status = Paragraph::new(status_text)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(status, chunks[1]);
}

fn draw_op_input(f: &mut Frame, area: Rect, app: &App) {
    use app::OpMode;
    let w = area.width.min(70);
    let h = 5;
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };
    let title = match app.op_mode {
        OpMode::Copy => "Copy to (name or path)",
        OpMode::Move => "Move to (name or path)",
        OpMode::Mkdir => "Create directory name",
        OpMode::None => "",
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(Clear, popup);
    f.render_widget(block.clone(), popup);
    let inner = block.inner(popup);
    f.render_widget(&app.op_input, inner);
}

fn draw_move_destination_picker(f: &mut Frame, area: Rect, app: &App) {
    let w = area.width.min(60);
    let h = area.height.min(18);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    let source_name = app
        .move_source
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let title = format!(
        "Move '{}' to — {}",
        source_name,
        app.move_dest_dir.display()
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    f.render_widget(Clear, popup);
    f.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    // Split for list and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let list_area = chunks[0];
    let status_area = chunks[1];

    let items: Vec<ListItem> = app
        .move_dest_items
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let name = if p == &app.move_dest_dir {
                "..".to_string()
            } else {
                format!("{}/", p.file_name().unwrap_or_default().to_string_lossy())
            };

            let style = if i == app.move_dest_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_widget(list, list_area);

    // Status bar
    let status_text = "ENTER:move-here ↑↓:navigate →:enter ESC:cancel";
    let status_bar =
        Paragraph::new(status_text).style(Style::default().fg(Color::White).bg(Color::Green));
    f.render_widget(status_bar, status_area);
}

fn draw_git_status(f: &mut Frame, area: Rect, app: &App) {
    let w = area.width.min(80);
    let h = area.height.min(20);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let popup = Rect {
        x,
        y,
        width: w,
        height: h,
    };

    let git_root = app
        .git_repo
        .as_ref()
        .map(|repo| repo.root().display().to_string())
        .unwrap_or_else(|| "Not a Git repository".to_string());

    let title = format!("Git Status — {}", git_root);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    f.render_widget(Clear, popup);
    f.render_widget(block.clone(), popup);
    let inner = block.inner(popup);

    // Split for content and status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let content_area = chunks[0];
    let status_area = chunks[1];

    // Git status content
    let content = if app.git_status_text.is_empty() {
        "Working tree clean".to_string()
    } else {
        app.git_status_text.clone()
    };

    let status_paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(status_paragraph, content_area);

    // Status bar
    let status_text = "ESC:close ENTER:close S:refresh";
    let status_bar =
        Paragraph::new(status_text).style(Style::default().fg(Color::White).bg(Color::Magenta));
    f.render_widget(status_bar, status_area);
}
