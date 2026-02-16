//! Core selector framework with crossterm-based navigation and item operations
//!
//! This module provides a VSCode command palette-like experience with:
//! - Filter/search as first option
//! - Item operations (delete, rename, refresh)
//! - Secondary menu access
//! - Position preservation during refresh

use crate::selectors::error::SelectorError;
use crossterm::{
    ExecutableCommand, QueueableCommand,
    cursor::{Hide, MoveTo, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read},
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write, stdout};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Trait for items that can be selected from a list
pub trait SelectableItem {
    /// Get the display name for the item
    fn display_name(&self) -> String;

    /// Format the item for display in a selection list
    fn format_for_list(&self) -> String;

    /// Get a unique identifier for the item (optional)
    fn id(&self) -> Option<String> {
        None
    }
}

/// Selector result with all possible actions
#[derive(Debug, Clone)]
pub enum SelectionResult<T> {
    /// Item was selected
    Selected(T),
    /// User wants to create a new item
    Create,
    /// User provided custom input
    CustomInput(String),
    /// User wants to go back
    Back,
    /// User wants to exit
    Exit,
    /// User wants to delete an item
    Delete(T),
    /// User wants to rename an item
    Rename(T),
    /// User wants to refresh the list
    Refresh,
    /// User wants to view item details
    ViewDetails(T),
}

/// Cursor display style
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Default,
    Block,
    Line,
}

impl CursorStyle {
    #[allow(clippy::wrong_self_convention)]
    fn to_ansi(&self) -> &'static str {
        match self {
            CursorStyle::Default => "\x1b[0 q",
            CursorStyle::Block => "\x1b[2 q",
            CursorStyle::Line => "\x1b[5 q",
        }
    }
}

/// Configuration for selector
#[derive(Clone, Debug)]
pub struct SelectorConfig {
    pub page_size: usize,
    pub cursor_style: CursorStyle,
    pub allow_create: bool,
    pub allow_custom: bool,
    pub allow_management: bool, // Enable d/n/r shortcuts and ViewDetails on Enter
    pub show_item_count: bool,
    pub preserve_position_on_refresh: bool,
    pub show_filter: bool, // New: show/hide filter input
}

impl Default for SelectorConfig {
    fn default() -> Self {
        Self {
            page_size: 10,
            cursor_style: CursorStyle::Block,
            allow_create: false,
            allow_custom: false,
            allow_management: true, // Default to true for backward compatibility
            show_item_count: true,
            preserve_position_on_refresh: true,
            show_filter: true, // Default to showing filter
        }
    }
}

/// Input state for filter functionality
#[derive(Clone, Debug)]
struct InputState {
    content: String,
    grapheme_count: usize,
    cursor_grapheme_idx: usize,
}

impl InputState {
    fn new() -> Self {
        Self {
            content: String::new(),
            grapheme_count: 0,
            cursor_grapheme_idx: 0,
        }
    }

    fn from_content(content: String) -> Self {
        let grapheme_count = content.graphemes(true).count();
        Self {
            content,
            grapheme_count,
            cursor_grapheme_idx: grapheme_count,
        }
    }

    fn insert_char(&mut self, c: char) {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let mut new_content = String::new();
        for (idx, grapheme) in graphemes.iter().enumerate() {
            if idx == self.cursor_grapheme_idx {
                new_content.push(c);
            }
            new_content.push_str(grapheme);
        }
        if self.cursor_grapheme_idx >= graphemes.len() {
            new_content.push(c);
        }
        self.content = new_content;
        self.grapheme_count = self.content.graphemes(true).count();
        self.cursor_grapheme_idx += 1;
    }

    fn delete_char(&mut self) -> bool {
        if self.cursor_grapheme_idx < self.grapheme_count {
            let graphemes: Vec<&str> = self.content.graphemes(true).collect();
            let mut new_content = String::new();
            for (idx, grapheme) in graphemes.iter().enumerate() {
                if idx != self.cursor_grapheme_idx {
                    new_content.push_str(grapheme);
                }
            }
            self.content = new_content;
            self.grapheme_count = self.content.graphemes(true).count();
            true
        } else {
            false
        }
    }

    fn backspace(&mut self) -> bool {
        if self.cursor_grapheme_idx > 0 {
            let graphemes: Vec<&str> = self.content.graphemes(true).collect();
            let mut new_content = String::new();
            for (idx, grapheme) in graphemes.iter().enumerate() {
                if idx != self.cursor_grapheme_idx - 1 {
                    new_content.push_str(grapheme);
                }
            }
            self.content = new_content;
            self.grapheme_count = self.content.graphemes(true).count();
            self.cursor_grapheme_idx -= 1;
            true
        } else {
            false
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_grapheme_idx > 0 {
            self.cursor_grapheme_idx -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_grapheme_idx < self.grapheme_count {
            self.cursor_grapheme_idx += 1;
        }
    }

    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn content(&self) -> &str {
        &self.content
    }

    fn pre_cursor_width(&self) -> usize {
        let graphemes: Vec<&str> = self.content.graphemes(true).collect();
        let pre_cursor: String =
            graphemes[..self.cursor_grapheme_idx.min(graphemes.len())].concat();
        UnicodeWidthStr::width(pre_cursor.as_str())
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Main selector implementation
pub struct Selector<'a, T: SelectableItem + Clone> {
    message: &'a str,
    items: Vec<T>,
    config: SelectorConfig,
    starting_cursor: usize,
    phantom: std::marker::PhantomData<T>,
}

impl<'a, T: SelectableItem + Clone> Selector<'a, T> {
    pub fn new(message: &'a str, items: Vec<T>) -> Self {
        Self {
            message,
            items,
            config: SelectorConfig::default(),
            starting_cursor: 0,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn with_config(mut self, config: SelectorConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_starting_cursor(mut self, cursor: usize) -> Self {
        self.starting_cursor = cursor;
        self
    }

    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.config.page_size = page_size;
        self
    }

    pub fn with_create(mut self, allow_create: bool) -> Self {
        self.config.allow_create = allow_create;
        self
    }

    pub fn with_custom(mut self, allow_custom: bool) -> Self {
        self.config.allow_custom = allow_custom;
        self
    }

    pub fn prompt(&mut self) -> std::io::Result<SelectionResult<T>> {
        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();

        // Set cursor style
        stdout.execute(Print(self.config.cursor_style.to_ansi()))?;

        // Create state
        let mut state = SelectorState {
            cursor_index: self.starting_cursor,
            scroll_offset: 0,
            filter_text: String::new(),
            filtered_items: self.items.clone(),
            input_state: InputState::new(),
        };

        // Initial render
        self.render(&mut stdout, &state)?;

        // Main event loop
        let result = loop {
            if let Event::Key(key_event) = read()? {
                match self.handle_key_event(key_event, &mut state)? {
                    KeyHandleResult::Continue => {
                        self.render(&mut stdout, &state)?;
                    }
                    KeyHandleResult::Submit(action) => {
                        break Ok(action);
                    }
                    KeyHandleResult::Refresh => {
                        // Refresh the items and update state
                        self.refresh_items(&mut state);
                        self.render(&mut stdout, &state)?;
                    }
                }
            }
        };

        // Cleanup
        self.cleanup(&mut stdout)?;

        result
    }

    fn refresh_items(&self, state: &mut SelectorState<T>) {
        // This would be overridden by specific implementations to refresh their item lists
        // For now, we'll reapply the current filter to refreshed items
        let current_filter_text = state.input_state.content().to_string();

        // Reapply filter to items (override in implementations for actual refresh)
        self.apply_filter_with_text(state, &current_filter_text);

        // Preserve cursor position if enabled
        if !self.config.preserve_position_on_refresh {
            state.cursor_index = 0;
            state.scroll_offset = 0;
        } else {
            // Adjust cursor if list has shrunk
            if state.cursor_index >= state.filtered_items.len() {
                state.cursor_index = state.filtered_items.len().saturating_sub(1);
            }
        }
    }

    fn apply_filter_with_text(&self, state: &mut SelectorState<T>, filter_text: &str) {
        state.filter_text = filter_text.to_string();

        if filter_text.is_empty() {
            state.filtered_items = self.items.clone();
        } else {
            state.filtered_items = self
                .items
                .iter()
                .filter(|item| {
                    item.display_name()
                        .to_lowercase()
                        .contains(&filter_text.to_lowercase())
                })
                .cloned()
                .collect();
        }
    }

    fn handle_key_event(
        &self,
        key: KeyEvent,
        state: &mut SelectorState<T>,
    ) -> io::Result<KeyHandleResult<T>> {
        match key.code {
            KeyCode::Up => {
                if state.cursor_index > 0 {
                    state.cursor_index -= 1;
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Down => {
                let max_index = self.get_total_option_count(state) - 1;
                if state.cursor_index < max_index {
                    state.cursor_index += 1;
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::PageUp => {
                let new_index = state.cursor_index.saturating_sub(self.config.page_size);
                if new_index != state.cursor_index {
                    state.cursor_index = new_index;
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::PageDown => {
                let max_index = self.get_total_option_count(state) - 1;
                let new_index = (state.cursor_index + self.config.page_size).min(max_index);
                if new_index != state.cursor_index {
                    state.cursor_index = new_index;
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Home => {
                state.cursor_index = 0;
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::End => {
                state.cursor_index = self.get_total_option_count(state) - 1;
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                Ok(KeyHandleResult::Submit(SelectionResult::Exit))
            }
            KeyCode::Char(c) => {
                let is_on_filter = self.config.show_filter && state.cursor_index == 0;

                // First check if we're on filter option (index 0) - if so, always treat as input
                if is_on_filter {
                    // Always add to input when on filter option
                    state.input_state.insert_char(c);
                    self.update_filter(state);
                    return Ok(KeyHandleResult::Continue);
                }

                // Only check for shortcuts when not on filter option and management is enabled
                if self.config.allow_management {
                    match c.to_lowercase().collect::<String>().as_str() {
                        "d" => {
                            // Delete operation
                            if let Some(item) = self.get_item_at_cursor(state) {
                                return Ok(KeyHandleResult::Submit(SelectionResult::Delete(item)));
                            }
                        }
                        "n" => {
                            // Rename operation
                            if let Some(item) = self.get_item_at_cursor(state) {
                                return Ok(KeyHandleResult::Submit(SelectionResult::Rename(item)));
                            }
                        }
                        "r" => {
                            // Refresh operation - only when not on filter option
                            return Ok(KeyHandleResult::Refresh);
                        }
                        _ => {
                            // Only add to filter if filter is enabled
                            if self.config.show_filter {
                                state.input_state.insert_char(c);
                                self.update_filter(state);
                            }
                        }
                    }
                } else {
                    // Management disabled: treat all chars as filter input
                    if self.config.show_filter {
                        state.input_state.insert_char(c);
                        self.update_filter(state);
                    }
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Backspace => {
                if self.config.show_filter {
                    state.input_state.backspace();
                    self.update_filter(state);
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Delete => {
                if self.config.show_filter {
                    state.input_state.delete_char();
                    self.update_filter(state);
                }
                Ok(KeyHandleResult::Continue)
            }
            KeyCode::Left => {
                let is_on_filter = self.config.show_filter && state.cursor_index == 0;
                if is_on_filter {
                    // On filter option, move cursor left in input
                    state.input_state.move_cursor_left();
                    Ok(KeyHandleResult::Continue)
                } else {
                    // No filter or not on filter - act as back
                    Ok(KeyHandleResult::Submit(SelectionResult::Back))
                }
            }
            KeyCode::Right => {
                // Right arrow behavior depends on context
                let is_on_filter = self.config.show_filter && state.cursor_index == 0;
                let is_on_create = self.config.allow_create
                    && state.cursor_index
                        == state.filtered_items.len()
                            + if self.config.show_filter { 1 } else { 0 };
                let is_on_custom = self.config.allow_custom
                    && state.cursor_index
                        == state.filtered_items.len()
                            + if self.config.allow_create { 1 } else { 0 }
                            + if self.config.show_filter { 1 } else { 0 };

                if is_on_filter {
                    // On filter option, move cursor right in input
                    state.input_state.move_cursor_right();
                    Ok(KeyHandleResult::Continue)
                } else if is_on_create {
                    Ok(KeyHandleResult::Submit(SelectionResult::Create))
                } else if is_on_custom {
                    Ok(KeyHandleResult::Submit(SelectionResult::CustomInput(
                        state.filter_text.clone(),
                    )))
                } else {
                    // On item, select it directly
                    if let Some(item) = self.get_item_at_cursor(state) {
                        Ok(KeyHandleResult::Submit(SelectionResult::Selected(item)))
                    } else {
                        Ok(KeyHandleResult::Continue)
                    }
                }
            }
            KeyCode::Enter => {
                // Enter on filter option selects first filtered item or treats as custom input
                let is_on_filter = self.config.show_filter && state.cursor_index == 0;
                let is_on_create = self.config.allow_create
                    && state.cursor_index
                        == state.filtered_items.len()
                            + if self.config.show_filter { 1 } else { 0 };
                let is_on_custom = self.config.allow_custom
                    && state.cursor_index
                        == state.filtered_items.len()
                            + if self.config.allow_create { 1 } else { 0 }
                            + if self.config.show_filter { 1 } else { 0 };

                if is_on_filter {
                    if let Some(item) = state.filtered_items.first() {
                        Ok(KeyHandleResult::Submit(SelectionResult::Selected(
                            item.clone(),
                        )))
                    } else if self.config.allow_custom && !state.filter_text.is_empty() {
                        Ok(KeyHandleResult::Submit(SelectionResult::CustomInput(
                            state.filter_text.clone(),
                        )))
                    } else {
                        Ok(KeyHandleResult::Continue)
                    }
                } else if is_on_create {
                    Ok(KeyHandleResult::Submit(SelectionResult::Create))
                } else if is_on_custom {
                    Ok(KeyHandleResult::Submit(SelectionResult::CustomInput(
                        state.filter_text.clone(),
                    )))
                } else if let Some(item) = self.get_item_at_cursor(state) {
                    if self.config.allow_management {
                        // Management mode: Enter shows details/secondary menu
                        Ok(KeyHandleResult::Submit(SelectionResult::ViewDetails(item)))
                    } else {
                        // Picker mode: Enter directly selects the item
                        Ok(KeyHandleResult::Submit(SelectionResult::Selected(item)))
                    }
                } else {
                    Ok(KeyHandleResult::Continue)
                }
            }
            KeyCode::Esc => Ok(KeyHandleResult::Submit(SelectionResult::Back)),
            _ => Ok(KeyHandleResult::Continue),
        }
    }

    fn update_filter(&self, state: &mut SelectorState<T>) {
        let filter_text = state.input_state.content().to_string();
        self.apply_filter_with_text(state, &filter_text);

        // Reset cursor position if needed
        if state.cursor_index >= self.get_total_option_count(state) {
            state.cursor_index = 0;
        }
    }

    fn get_total_option_count(&self, state: &SelectorState<T>) -> usize {
        let mut count = 0;
        if self.config.show_filter {
            count += 1; // Filter option
        }
        count += state.filtered_items.len();
        if self.config.allow_create {
            count += 1;
        }
        if self.config.allow_custom {
            count += 1;
        }
        count
    }

    fn get_item_at_cursor(&self, state: &SelectorState<T>) -> Option<T> {
        let item_index = if self.config.show_filter {
            if state.cursor_index == 0 {
                return None; // Filter option
            }
            state.cursor_index - 1
        } else {
            state.cursor_index
        };
        state.filtered_items.get(item_index).cloned()
    }

    fn render(&self, stdout: &mut io::Stdout, state: &SelectorState<T>) -> io::Result<()> {
        // Clear screen
        stdout.queue(Clear(ClearType::All))?;
        stdout.queue(MoveTo(0, 0))?;

        // Render prompt
        stdout
            .execute(SetForegroundColor(Color::Cyan))?
            .execute(Print("? "))?
            .execute(ResetColor)?
            .execute(Print(self.message))?;

        // Render item count if enabled
        if self.config.show_item_count {
            stdout
                .execute(SetForegroundColor(Color::DarkGrey))?
                .execute(Print(format!(" ({} items)", state.filtered_items.len())))?
                .execute(ResetColor)?;
        }

        // Render options
        self.render_options(stdout, state)?;

        // Render help
        self.render_help(stdout, state)?;

        // IMPORTANT: Reposition cursor after rendering help
        self.reposition_cursor(stdout, state)?;

        stdout.execute(ResetColor)?;
        stdout.flush()?;

        Ok(())
    }

    fn render_options(&self, stdout: &mut io::Stdout, state: &SelectorState<T>) -> io::Result<()> {
        let options = self.get_display_options(state);

        // Calculate pagination
        let page_size = self.config.page_size;
        let half_page = page_size / 2;

        let (scroll_offset, _cursor_in_page) =
            if options.len() <= page_size || state.cursor_index < half_page {
                (0, state.cursor_index)
            } else if state.cursor_index >= options.len() - half_page {
                (
                    options.len().saturating_sub(page_size),
                    state.cursor_index - (options.len().saturating_sub(page_size)),
                )
            } else {
                (state.cursor_index - half_page, half_page)
            };

        // Render options
        for (i, option) in options
            .iter()
            .enumerate()
            .take(options.len().min(scroll_offset + page_size))
            .skip(scroll_offset)
        {
            let is_cursor = i == state.cursor_index;

            // Move to option line
            let visual_line = (i - scroll_offset) + 2; // +2 for prompt line and blank line
            stdout.queue(MoveTo(0, visual_line as u16))?;

            // Render prefix
            if is_cursor {
                stdout
                    .execute(SetForegroundColor(Color::Yellow))?
                    .execute(Print("❯"))?
                    .execute(ResetColor)?
                    .execute(Print(" "))?;
            } else {
                stdout.execute(Print("  "))?;
            }

            // Render option content
            let is_filter_option = self.config.show_filter && i == 0;
            if is_cursor && is_filter_option {
                // Filter option with cursor
                let filter_text = state.input_state.content();
                if filter_text.is_empty() {
                    // Show placeholder in dim color
                    stdout
                        .execute(SetForegroundColor(Color::DarkGrey))?
                        .execute(Print(option))?;
                } else {
                    // Show actual input in normal color
                    stdout
                        .execute(SetForegroundColor(Color::Cyan))?
                        .execute(Print(format!("🔍 {}", filter_text)))?;
                }
                // Note: Cursor positioning is now handled by reposition_cursor()
            } else {
                // Regular option
                if is_cursor {
                    stdout
                        .execute(SetForegroundColor(Color::Yellow))?
                        .execute(Print(option))?;
                } else {
                    stdout.execute(ResetColor)?.execute(Print(option))?;
                }
            }
        }

        // Show/hide cursor
        let is_on_filter = self.config.show_filter && state.cursor_index == 0;
        if is_on_filter {
            // Show cursor when on filter option for text input
            stdout.execute(Show)?;
        } else {
            stdout.execute(Hide)?;
        }

        Ok(())
    }

    fn reposition_cursor(
        &self,
        stdout: &mut io::Stdout,
        state: &SelectorState<T>,
    ) -> io::Result<()> {
        let is_on_filter = self.config.show_filter && state.cursor_index == 0;
        if is_on_filter {
            // We're on the filter option, position cursor according to actual input content
            let filter_line = 2; // 0-indexed: line 0 is prompt, line 1 is blank, line 2 is first option

            // Calculate cursor position: "❯ " + "🔍 " + cursor position in text
            let prefix_width = unicode_width::UnicodeWidthStr::width("❯ 🔍 "); // 6 display columns
            let cursor_col = prefix_width + state.input_state.pre_cursor_width();

            stdout.queue(MoveTo(cursor_col as u16, filter_line as u16))?;
        }
        // If not on filter option, cursor should be hidden (handled in render_options)

        Ok(())
    }

    fn get_display_options(&self, state: &SelectorState<T>) -> Vec<String> {
        let mut options = Vec::new();

        // Filter option - only if enabled
        if self.config.show_filter {
            let filter_display = if state.input_state.is_empty() {
                "🔍 Filter/Custom search...".to_string()
            } else {
                format!("🔍 {}", state.input_state.content())
            };
            options.push(filter_display);
        }

        // Filtered items
        for item in &state.filtered_items {
            options.push(item.format_for_list());
        }

        // Special options
        if self.config.allow_create {
            options.push("➕ Create New...".to_string());
        }
        if self.config.allow_custom {
            options.push("✏️ Enter Custom Value...".to_string());
        }

        options
    }

    fn render_help(&self, stdout: &mut io::Stdout, state: &SelectorState<T>) -> io::Result<()> {
        // Dynamic help based on current selection
        let help_line = self.get_dynamic_help_message(state);

        let help_line_num = self
            .get_display_options(state)
            .len()
            .min(self.config.page_size)
            + 3; // +3 for prompt, blank, and spacing
        stdout.queue(MoveTo(0, help_line_num as u16))?;
        stdout
            .execute(SetForegroundColor(Color::DarkGrey))?
            .execute(Print(help_line))?
            .execute(ResetColor)?;

        Ok(())
    }

    /// Get dynamic help message based on current selection state
    fn get_dynamic_help_message(&self, state: &SelectorState<T>) -> String {
        let is_on_filter = self.config.show_filter && state.cursor_index == 0;
        let is_on_create = self.config.allow_create
            && state.cursor_index
                == state.filtered_items.len() + if self.config.show_filter { 1 } else { 0 };
        let is_on_custom = self.config.allow_custom
            && state.cursor_index
                == state.filtered_items.len()
                    + if self.config.allow_create { 1 } else { 0 }
                    + if self.config.show_filter { 1 } else { 0 };
        let is_on_regular_item = state.cursor_index
            < state.filtered_items.len() + if self.config.show_filter { 1 } else { 0 };

        if is_on_filter {
            "Type to filter, Enter to search, ↑↓ to navigate, Esc to back".to_string()
        } else if is_on_create {
            "Enter to create new item, ↑↓ to navigate, Esc to back".to_string()
        } else if is_on_custom {
            "Enter to input custom value, ↑↓ to navigate, Esc to back".to_string()
        } else if is_on_regular_item {
            let mut help_parts = vec!["↑↓ to navigate".to_string(), "Enter to select".to_string()];

            if self.config.show_filter {
                help_parts.push("←→ to move cursor".to_string());
            }

            if self.config.allow_management {
                help_parts.extend_from_slice(&[
                    "d: Delete".to_string(),
                    "n: Rename".to_string(),
                    "r: Refresh".to_string(),
                ]);
            }

            help_parts.push("Esc: Back".to_string());

            help_parts.join(", ")
        } else {
            "↑↓ to navigate, Enter: Select, Esc: Back".to_string()
        }
    }

    fn cleanup(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Reset cursor style
        stdout.execute(Print(CursorStyle::Default.to_ansi()))?;
        stdout.execute(Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Get current selector state for saving
    pub fn get_state(&self) -> SelectorState<T> {
        // Note: This would need to be called from within the prompt method
        // For now, we'll create a default state
        SelectorState {
            cursor_index: self.starting_cursor,
            scroll_offset: 0,
            filter_text: String::new(),
            filtered_items: self.items.clone(),
            input_state: InputState::new(),
        }
    }

    /// Restore selector state from saved state
    pub fn restore_state(&mut self, state: SelectorState<T>) {
        // Note: This would need to be integrated into the prompt method
        // For now, we'll update the starting cursor position
        self.starting_cursor = state.cursor_index;
    }
}

#[derive(Clone)]
pub struct SelectorState<T> {
    cursor_index: usize,
    scroll_offset: usize,
    filter_text: String,
    filtered_items: Vec<T>,
    input_state: InputState,
}

enum KeyHandleResult<T> {
    Continue,
    Submit(SelectionResult<T>),
    Refresh,
}

// Implement SelectableItem for String to enable simple string selections
impl SelectableItem for String {
    fn display_name(&self) -> String {
        self.clone()
    }

    fn format_for_list(&self) -> String {
        self.clone()
    }

    fn id(&self) -> Option<String> {
        Some(self.clone())
    }
}

/// Base implementation for simple selectors
pub struct BaseSelector<T: SelectableItem> {
    items: Vec<T>,
    title: String,
    allow_create: bool,
    show_filter: bool, // Added to support hiding filter
    saved_state: Option<SelectorState<T>>,
}

impl<T: SelectableItem + Clone> BaseSelector<T> {
    pub fn new(items: Vec<T>, title: &str) -> Self {
        Self {
            items,
            title: title.to_string(),
            allow_create: false,
            show_filter: true, // Default to showing filter for backward compatibility
            saved_state: None,
        }
    }

    pub fn with_show_filter(mut self, show_filter: bool) -> Self {
        self.show_filter = show_filter;
        self
    }

    pub fn with_create(mut self, allow_create: bool) -> Self {
        self.allow_create = allow_create;
        self
    }

    pub fn run(&mut self) -> crate::selectors::error::SelectorResult<Option<T>> {
        let config = SelectorConfig {
            allow_create: self.allow_create,
            show_filter: self.show_filter, // Use the show_filter setting
            ..SelectorConfig::default()
        };

        let mut selector = Selector::new(&self.title, self.items.clone()).with_config(config);

        // Set starting position from saved state
        if let Some(ref saved_state) = self.saved_state {
            selector.restore_state(saved_state.clone());
        }

        match selector.prompt()? {
            SelectionResult::Selected(item) | SelectionResult::ViewDetails(item) => Ok(Some(item)),
            SelectionResult::Create => self.handle_create(),
            SelectionResult::CustomInput(input) => self.handle_custom_input(input),
            SelectionResult::Delete(item) => self.handle_delete(item),
            SelectionResult::Rename(item) => self.handle_rename(item),
            SelectionResult::Refresh => self.handle_refresh(),
            SelectionResult::Back => {
                // Clear saved state when going back from main menu
                self.saved_state = None;
                Ok(None)
            }
            SelectionResult::Exit => {
                println!("🚫 Operation cancelled by user.");
                std::process::exit(0);
            }
        }
    }

    fn handle_create(&self) -> crate::selectors::error::SelectorResult<Option<T>> {
        Err(SelectorError::OperationFailed(
            "Create operation not implemented".to_string(),
        ))
    }

    fn handle_custom_input(
        &self,
        _input: String,
    ) -> crate::selectors::error::SelectorResult<Option<T>> {
        Err(SelectorError::OperationFailed(
            "Custom input not supported".to_string(),
        ))
    }

    fn handle_delete(&self, _item: T) -> crate::selectors::error::SelectorResult<Option<T>> {
        Err(SelectorError::OperationFailed(
            "Delete operation not implemented".to_string(),
        ))
    }

    fn handle_rename(&self, _item: T) -> crate::selectors::error::SelectorResult<Option<T>> {
        Err(SelectorError::OperationFailed(
            "Rename operation not implemented".to_string(),
        ))
    }

    fn handle_refresh(&self) -> crate::selectors::error::SelectorResult<Option<T>> {
        Err(SelectorError::OperationFailed(
            "Refresh operation not implemented".to_string(),
        ))
    }

    /// Show item details in a secondary interface using list-like UI without filter
    pub fn show_item_details(
        item: &T,
        title: &str,
    ) -> crate::selectors::error::SelectorResult<SelectionResult<T>>
    where
        T: SelectableItem + Clone,
    {
        // Create a config without filter
        let config = SelectorConfig {
            show_filter: false,
            allow_create: false,
            allow_custom: false,
            ..SelectorConfig::default()
        };

        // Create a selector with just the item itself
        let mut selector = Selector::new(title, vec![item.clone()]).with_config(config);

        // Run the selector
        let result = selector.prompt()?;

        // Map the result
        match result {
            SelectionResult::Selected(selected_item) => {
                Ok(SelectionResult::Selected(selected_item))
            }
            SelectionResult::Rename(rename_item) => Ok(SelectionResult::Rename(rename_item)),
            SelectionResult::Delete(delete_item) => Ok(SelectionResult::Delete(delete_item)),
            SelectionResult::Back => Ok(SelectionResult::Back),
            _ => Ok(SelectionResult::Back),
        }
    }
}

/// Prompt user for a new name for an item
pub fn prompt_rename(
    current_name: &str,
    item_type: &str,
) -> crate::selectors::error::SelectorResult<String> {
    // Setup terminal for input
    let _ = terminal::enable_raw_mode();
    let mut stdout = stdout();

    // Clear screen and show prompt
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(MoveTo(0, 0))?;

    stdout
        .execute(SetForegroundColor(Color::Cyan))?
        .execute(Print(format!("✏️  Rename {}:", item_type)))?
        .execute(ResetColor)?
        .execute(Print(format!("  Current: {}", current_name)))?;

    let mut input_state = InputState::from_content(current_name.to_string());
    let cursor_pos = "  New name: ".to_string();

    stdout.execute(Print(&cursor_pos))?;

    // Main input loop
    loop {
        if let Event::Key(key_event) = read()? {
            match key_event.code {
                KeyCode::Enter => {
                    let new_name = input_state.content().trim();
                    if new_name.is_empty() {
                        // Show error
                        stdout.queue(MoveTo(0, 3))?;
                        stdout
                            .execute(SetForegroundColor(Color::Red))?
                            .execute(Print("❌ Name cannot be empty"))?
                            .execute(ResetColor)?;
                        continue;
                    }
                    if new_name == current_name {
                        // Show unchanged message
                        stdout.queue(MoveTo(0, 3))?;
                        stdout
                            .execute(SetForegroundColor(Color::Yellow))?
                            .execute(Print("ℹ️  Name unchanged"))?
                            .execute(ResetColor)?;
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                        return Ok(new_name.to_string());
                    }
                    return Ok(new_name.to_string());
                }
                KeyCode::Esc => {
                    stdout.execute(Clear(ClearType::All))?;
                    return Err(SelectorError::Cancelled);
                }
                KeyCode::Char(c) => {
                    input_state.insert_char(c);
                    // Redraw the input
                    stdout.queue(MoveTo(cursor_pos.len() as u16, 1))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(Print(&input_state.content()))?;
                }
                KeyCode::Backspace => {
                    input_state.backspace();
                    // Redraw the input
                    stdout.queue(MoveTo(cursor_pos.len() as u16, 1))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(Print(&input_state.content()))?;
                }
                KeyCode::Delete => {
                    input_state.delete_char();
                    // Redraw the input
                    stdout.queue(MoveTo(cursor_pos.len() as u16, 1))?;
                    stdout.queue(Clear(ClearType::UntilNewLine))?;
                    stdout.queue(Print(&input_state.content()))?;
                }
                KeyCode::Left => {
                    input_state.move_cursor_left();
                    let cursor_col = cursor_pos.len() + input_state.pre_cursor_width();
                    stdout.queue(MoveTo(cursor_col as u16, 1))?;
                }
                KeyCode::Right => {
                    input_state.move_cursor_right();
                    let cursor_col = cursor_pos.len() + input_state.pre_cursor_width();
                    stdout.queue(MoveTo(cursor_col as u16, 1))?;
                }
                _ => {}
            }
        }
        stdout.flush()?;
    }
}
