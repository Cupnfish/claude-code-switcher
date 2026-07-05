//! Apply-TUI application state and event handling.
//!
//! The TUI gathers the user's intent (which key, effort, scope, co-author,
//! variant, auto-compact) for a given target and returns an [`ApplySelection`]; it
//! does not touch settings.json itself.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

use crate::CredentialManager;
use crate::credentials::{
    ApiKeyChoice, ApiKeySource, CredentialStore, collect_api_key_sources, mask_api_key,
};
use crate::prefs::{KeyRef, Prefs};
use crate::snapshots::SnapshotScope;
use crate::templates::{
    AutoCompactWindow, TemplateType, get_template_instance_with_input, is_generic_target,
    supports_auto_compact_option, variant_options,
};
use crate::usage::{UsageReport, UsageRequest, UsageStatus, query_usage, supports_usage};

use super::input::TextInput;

/// The user's confirmed choices from the TUI.
#[derive(Debug, Clone)]
pub struct ApplySelection {
    pub key: ApiKeyChoice,
    pub effort: Option<String>,
    pub scope: SnapshotScope,
    /// `true` = co-author disabled.
    pub co_author_off: bool,
    pub variant: Option<String>,
    pub auto_compact_window: Option<AutoCompactWindow>,
}

/// What an event produced.
pub enum Outcome {
    /// Keep running.
    Continue,
    /// User confirmed — apply these choices.
    Apply(ApplySelection),
    /// User cancelled (Esc / q).
    Quit,
}

/// Current interaction mode.
pub enum Mode {
    Normal,
    InputNewKey(TextInput),
    InputRename { idx: usize, input: TextInput },
    ConfirmDelete { idx: usize },
    Help,
    Message(String),
}

const EFFORTS: &[&str] = &["max", "xhigh", "high", "medium", "low"];
const SCOPES: &[SnapshotScope] = &[
    SnapshotScope::Common,
    SnapshotScope::Env,
    SnapshotScope::All,
];
const USAGE_CACHE_TTL: Duration = Duration::from_secs(60);
const SPINNER_FRAMES: [char; 4] = ['-', '\\', '|', '/'];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UsageCacheKey {
    template_type: TemplateType,
    api_key: String,
    base_url: Option<String>,
}

struct UsageCacheEntry {
    report: UsageReport,
    fetched_at: Instant,
}

struct UsageInFlight {
    key: UsageCacheKey,
    receiver: Receiver<Result<UsageReport, String>>,
}

fn supported_auto_compact_windows_for(
    template_type: &TemplateType,
    alias: &str,
) -> Vec<AutoCompactWindow> {
    let template = get_template_instance_with_input(template_type, alias);
    if supports_auto_compact_option(template.as_ref()) {
        template.supported_auto_compact_windows().to_vec()
    } else {
        Vec::new()
    }
}

/// Which row the cursor is on.
#[derive(Clone, Copy)]
enum Row {
    Key(usize),
    NewKey,
    Effort,
    Scope,
    CoAuthor,
    Variant,
    AutoCompact,
    UsageRefresh,
    Apply,
}

pub struct App {
    pub template_type: TemplateType,
    pub target: String,
    pub display_name: String,
    pub current_label: String,

    sources: Vec<ApiKeySource>,
    selected_key: Option<usize>,
    cursor: usize,

    effort_idx: usize,
    scope_idx: usize,
    co_author: bool, // true = enabled
    variant_aliases: Vec<(&'static str, &'static str)>,
    variant_idx: usize,
    has_variant_row: bool,
    auto_compact_windows: Vec<AutoCompactWindow>,
    auto_compact_idx: usize,

    usage_cache: HashMap<UsageCacheKey, UsageCacheEntry>,
    usage_in_flight: Option<UsageInFlight>,
    usage_status: UsageStatus,
    usage_status_key: Option<UsageCacheKey>,
    spinner_idx: usize,

    pub mode: Mode,
}

impl App {
    /// Build the app state, pre-filling from prefs (remembered last selection,
    /// falling back to global defaults).
    pub fn new(
        template_type: TemplateType,
        target: String,
        display_name: String,
        current_label: String,
        prefs: &Prefs,
    ) -> anyhow::Result<Self> {
        let sources = collect_api_key_sources(&template_type)?;

        let variant_aliases = variant_options(&template_type);
        let has_variant_row = !variant_aliases.is_empty() && is_generic_target(&target);
        let tpref = prefs.template_pref(&template_type);

        // variant
        let variant_idx = if !has_variant_row {
            0
        } else {
            let remembered = tpref.and_then(|p| p.variant.as_deref());
            remembered
                .and_then(|v| variant_aliases.iter().position(|(a, _)| *a == v))
                .unwrap_or(0)
        };
        let initial_alias = if has_variant_row {
            variant_aliases[variant_idx].0
        } else {
            target.as_str()
        };
        let auto_compact_windows =
            supported_auto_compact_windows_for(&template_type, initial_alias);

        // auto-compact: last → provider default
        let auto_compact_idx = if auto_compact_windows.is_empty() {
            0
        } else {
            tpref
                .and_then(|p| {
                    p.last_auto_compact_window
                        .as_deref()
                        .or(p.last_context_window.as_deref())
                })
                .and_then(|value| value.parse::<AutoCompactWindow>().ok())
                .and_then(|value| auto_compact_windows.iter().position(|x| *x == value))
                .unwrap_or(0)
        };

        // selected key: remembered & still present, else first
        let selected_key = (|| {
            let kr = tpref.and_then(|p| p.last_key.as_ref())?;
            sources.iter().position(|s| match (s, kr) {
                (ApiKeySource::EnvVar { env_var_name, .. }, KeyRef::EnvVar(n)) => env_var_name == n,
                (ApiKeySource::Saved { credential }, KeyRef::Credential(id)) => {
                    credential.id() == id
                }
                _ => false,
            })
        })()
        .or(if sources.is_empty() { None } else { Some(0) });

        // effort: last → global default → "max"
        let effort_idx = tpref
            .and_then(|p| p.last_effort.as_deref())
            .or(prefs.default_effort.as_deref())
            .and_then(|e| EFFORTS.iter().position(|x| *x == e))
            .unwrap_or(0);

        // scope: last → global default → common (0)
        let scope_idx = tpref
            .and_then(|p| p.last_scope.as_ref())
            .or(Some(&prefs.default_scope))
            .and_then(|s| SCOPES.iter().position(|x| x == s))
            .unwrap_or(0);

        // co-author: last → global default (false = off)
        let co_author = tpref
            .and_then(|p| p.last_co_author)
            .unwrap_or(prefs.default_co_author);

        // initial cursor: on the selected key if any, else NewKey, else Apply
        let cursor = selected_key.unwrap_or({
            // sources empty → NewKey is row index 0 (== sources.len())
            0
        });

        let mut app = Self {
            template_type,
            target,
            display_name,
            current_label,
            sources,
            selected_key,
            cursor,
            effort_idx,
            scope_idx,
            co_author,
            variant_aliases,
            variant_idx,
            has_variant_row,
            auto_compact_windows,
            auto_compact_idx,
            usage_cache: HashMap::new(),
            usage_in_flight: None,
            usage_status: UsageStatus::Unsupported,
            usage_status_key: None,
            spinner_idx: 0,
            mode: Mode::Normal,
        };
        app.refresh_usage(false);
        Ok(app)
    }

    // ── row model ────────────────────────────────────────────────────────────

    fn n_keys(&self) -> usize {
        self.sources.len()
    }
    fn n_options(&self) -> usize {
        3 + if self.has_variant_row { 1 } else { 0 }
            + if self.has_auto_compact_row() { 1 } else { 0 }
            + if self.has_usage_row() { 1 } else { 0 }
    }
    fn total_rows(&self) -> usize {
        self.n_keys() + 1 + self.n_options() + 1
    }
    fn apply_index(&self) -> usize {
        self.total_rows() - 1
    }

    fn row_at(&self, cursor: usize) -> Row {
        let nk = self.n_keys();
        if cursor < nk {
            return Row::Key(cursor);
        }
        if cursor == nk {
            return Row::NewKey;
        }
        if cursor == self.apply_index() {
            return Row::Apply;
        }

        // option rows start after NewKey
        let mut o = cursor - (nk + 1);
        if o == 0 {
            return Row::Effort;
        }
        o -= 1;
        if o == 0 {
            return Row::Scope;
        }
        o -= 1;
        if o == 0 {
            return Row::CoAuthor;
        }
        o -= 1;
        if self.has_variant_row {
            if o == 0 {
                return Row::Variant;
            }
            o -= 1;
        }
        if self.has_auto_compact_row() {
            if o == 0 {
                return Row::AutoCompact;
            }
            o -= 1;
        }
        if self.has_usage_row() && o == 0 {
            return Row::UsageRefresh;
        }
        Row::Apply
    }

    // ── accessors for rendering ──────────────────────────────────────────────

    pub fn sources(&self) -> &[ApiKeySource] {
        &self.sources
    }
    pub fn selected_key(&self) -> Option<usize> {
        self.selected_key
    }
    pub fn cursor(&self) -> usize {
        self.cursor
    }
    pub fn effort(&self) -> &'static str {
        EFFORTS[self.effort_idx]
    }
    pub fn scope(&self) -> SnapshotScope {
        SCOPES[self.scope_idx].clone()
    }
    pub fn co_author_enabled(&self) -> bool {
        self.co_author
    }
    pub fn has_variant_row(&self) -> bool {
        self.has_variant_row
    }
    pub fn variant_label(&self) -> Option<&'static str> {
        if !self.has_variant_row {
            return None;
        }
        Some(self.variant_aliases[self.variant_idx].1)
    }
    pub fn has_auto_compact_row(&self) -> bool {
        !self.auto_compact_windows.is_empty()
    }
    pub fn auto_compact_label(&self) -> Option<&'static str> {
        if !self.has_auto_compact_row() {
            return None;
        }
        Some(self.auto_compact_windows[self.auto_compact_idx].label())
    }
    pub fn auto_compact_window(&self) -> Option<AutoCompactWindow> {
        if !self.has_auto_compact_row() {
            return None;
        }
        Some(self.auto_compact_windows[self.auto_compact_idx])
    }
    pub fn has_usage_row(&self) -> bool {
        supports_usage(&self.template_type)
    }
    pub fn usage_status(&self) -> &UsageStatus {
        &self.usage_status
    }

    /// Build a template instance reflecting the current variant choice (for
    /// previewing model / base URL).
    pub fn preview_template_instance(&self) -> Box<dyn crate::templates::Template> {
        let alias = if self.has_variant_row {
            self.variant_aliases[self.variant_idx].0
        } else {
            self.target.as_str()
        };
        get_template_instance_with_input(&self.template_type, alias)
    }

    /// Compute the (model, base URL) the current selection would produce, for
    /// the Preview pane.
    pub fn preview_model_and_base(&self) -> (String, String) {
        let inst = self.preview_template_instance();
        let key = self
            .selected_key
            .and_then(|i| self.sources.get(i))
            .map(|s| s.api_key().to_string())
            .unwrap_or_else(|| "sk-preview".to_string());
        let settings = inst
            .create_settings_with_auto_compact(
                &key,
                &SnapshotScope::Common,
                self.auto_compact_window(),
            )
            .unwrap_or_else(|_| inst.create_settings(&key, &SnapshotScope::Common));
        let model = settings.model.unwrap_or_else(|| "(default)".to_string());
        let base = settings
            .env
            .as_ref()
            .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
            .cloned()
            .unwrap_or_else(|| "(none)".to_string());
        (model, base)
    }

    pub fn masked_selected_key(&self) -> String {
        match self.selected_key.and_then(|i| self.sources.get(i)) {
            Some(s) => mask_api_key(s.api_key()),
            None => "(none)".to_string(),
        }
    }

    fn selected_usage_key(&self) -> Option<UsageCacheKey> {
        if !supports_usage(&self.template_type) {
            return None;
        }
        let api_key = self
            .selected_key
            .and_then(|i| self.sources.get(i))
            .map(|s| s.api_key().to_string())?;
        let (_, base_url) = self.preview_model_and_base();
        Some(UsageCacheKey {
            template_type: self.template_type.clone(),
            api_key,
            base_url: if base_url == "(none)" {
                None
            } else {
                Some(base_url)
            },
        })
    }

    pub fn tick(&mut self) {
        self.spinner_idx = (self.spinner_idx + 1) % SPINNER_FRAMES.len();
        self.drain_usage_result();
        if self
            .selected_usage_key()
            .as_ref()
            .is_some_and(|key| self.usage_status_key.as_ref() != Some(key))
        {
            self.refresh_usage(false);
            return;
        }
        if let Some(key) = self.selected_usage_key() {
            let expired = self
                .usage_cache
                .get(&key)
                .is_some_and(|entry| entry.fetched_at.elapsed() >= USAGE_CACHE_TTL);
            let already_loading = self.usage_in_flight.as_ref().is_some_and(|p| p.key == key);
            if expired && !already_loading {
                self.refresh_usage(false);
            } else if matches!(self.usage_status, UsageStatus::Loading { .. }) {
                self.update_loading_spinner();
            }
        }
    }

    fn drain_usage_result(&mut self) {
        let Some(in_flight) = self.usage_in_flight.as_ref() else {
            return;
        };
        match in_flight.receiver.try_recv() {
            Ok(Ok(report)) => {
                let key = in_flight.key.clone();
                self.usage_cache.insert(
                    key.clone(),
                    UsageCacheEntry {
                        report: report.clone(),
                        fetched_at: Instant::now(),
                    },
                );
                self.usage_in_flight = None;
                if self.selected_usage_key().as_ref() == Some(&key) {
                    self.usage_status_key = Some(key);
                    self.usage_status = UsageStatus::Ready(report);
                }
            }
            Ok(Err(message)) => {
                let key = in_flight.key.clone();
                let stale = self.usage_cache.get(&key).map(|entry| entry.report.clone());
                self.usage_in_flight = None;
                if self.selected_usage_key().as_ref() == Some(&key) {
                    self.usage_status_key = Some(key);
                    self.usage_status = UsageStatus::Error { message, stale };
                }
            }
            Err(TryRecvError::Empty) => self.update_loading_spinner(),
            Err(TryRecvError::Disconnected) => {
                let key = in_flight.key.clone();
                let stale = self.usage_cache.get(&key).map(|entry| entry.report.clone());
                self.usage_in_flight = None;
                if self.selected_usage_key().as_ref() == Some(&key) {
                    self.usage_status_key = Some(key);
                    self.usage_status = UsageStatus::Error {
                        message: "usage query worker stopped".to_string(),
                        stale,
                    };
                }
            }
        }
    }

    fn refresh_usage(&mut self, force: bool) {
        self.drain_usage_result();
        if !supports_usage(&self.template_type) {
            self.usage_status_key = None;
            self.usage_status = UsageStatus::Unsupported;
            return;
        }
        let Some(key) = self.selected_usage_key() else {
            self.usage_status_key = None;
            self.usage_status = UsageStatus::NoKey;
            self.usage_in_flight = None;
            return;
        };

        if !force
            && let Some(entry) = self.usage_cache.get(&key)
            && entry.fetched_at.elapsed() < USAGE_CACHE_TTL
        {
            self.usage_status_key = Some(key);
            self.usage_status = UsageStatus::Ready(entry.report.clone());
            return;
        }

        if self.usage_in_flight.as_ref().is_some_and(|p| p.key == key) {
            self.update_loading_spinner();
            return;
        }

        let stale = self.usage_cache.get(&key).map(|entry| entry.report.clone());
        self.start_usage_query(key, stale);
    }

    fn start_usage_query(&mut self, key: UsageCacheKey, stale: Option<UsageReport>) {
        let request = UsageRequest {
            template_type: key.template_type.clone(),
            api_key: key.api_key.clone(),
            anthropic_base_url: key.base_url.clone(),
        };
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let result = query_usage(&request).map_err(|e| e.to_string());
            let _ = sender.send(result);
        });
        self.usage_status_key = Some(key.clone());
        self.usage_status = UsageStatus::Loading {
            stale,
            spinner: SPINNER_FRAMES[self.spinner_idx],
        };
        self.usage_in_flight = Some(UsageInFlight { key, receiver });
    }

    fn update_loading_spinner(&mut self) {
        if let UsageStatus::Loading { stale, spinner } = &mut self.usage_status {
            *spinner = SPINNER_FRAMES[self.spinner_idx];
            if stale.is_none()
                && let Some(key) = &self.usage_status_key
            {
                *stale = self.usage_cache.get(key).map(|entry| entry.report.clone());
            }
        }
    }

    // ── event handling ───────────────────────────────────────────────────────

    pub fn handle_event(&mut self, key: KeyEvent) -> Outcome {
        // Modes that capture keys: take the mode out by value so input handlers
        // can borrow `self` freely (the TextInput would otherwise alias self).
        let taken = std::mem::replace(&mut self.mode, Mode::Normal);
        match taken {
            Mode::InputNewKey(input) => return self.handle_input_newkey(key, input),
            Mode::InputRename { idx, input } => {
                return self.handle_input_rename(key, idx, input);
            }
            Mode::ConfirmDelete { idx } => return self.handle_confirm_delete(key, idx),
            Mode::Help | Mode::Message(_) => return Outcome::Continue, // self.mode already Normal
            Mode::Normal => {}
        }

        // Ctrl+C always quits.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Outcome::Quit;
        }

        match key.code {
            KeyCode::Up => self.move_cursor(-1),
            KeyCode::Down => self.move_cursor(1),
            KeyCode::Left | KeyCode::Right => {
                let dir = if key.code == KeyCode::Left { -1 } else { 1 };
                self.cycle_option(dir);
            }
            KeyCode::Enter => match self.row_at(self.cursor) {
                Row::Key(idx) => {
                    self.selected_key = Some(idx);
                    self.refresh_usage(false);
                    return self.build_apply();
                }
                Row::NewKey => self.mode = Mode::InputNewKey(TextInput::empty()),
                Row::Effort => self.cycle_effort(1),
                Row::Scope => self.cycle_scope(1),
                Row::CoAuthor => self.co_author = !self.co_author,
                Row::Variant => self.cycle_variant(1),
                Row::AutoCompact => self.cycle_auto_compact(1),
                Row::UsageRefresh => self.refresh_usage(true),
                Row::Apply => return self.build_apply(),
            },
            KeyCode::Char('a') => return self.build_apply(),
            KeyCode::Char('n') => self.mode = Mode::InputNewKey(TextInput::empty()),
            KeyCode::Char('d') => self.try_delete(),
            KeyCode::Char('r') => self.try_rename(),
            KeyCode::Char('u') => self.refresh_usage(true),
            KeyCode::Char('?') => self.mode = Mode::Help,
            KeyCode::Esc | KeyCode::Char('q') => return Outcome::Quit,
            _ => {}
        }

        Outcome::Continue
    }

    fn move_cursor(&mut self, delta: i32) {
        let total = self.total_rows() as i32;
        if total == 0 {
            return;
        }
        let mut c = self.cursor as i32 + delta;
        if c < 0 {
            c = 0;
        }
        if c >= total {
            c = total - 1;
        }
        self.cursor = c as usize;
        // live-select the key under the cursor
        if let Row::Key(idx) = self.row_at(self.cursor) {
            self.selected_key = Some(idx);
            self.refresh_usage(false);
        }
    }

    fn cycle_option(&mut self, dir: i32) {
        match self.row_at(self.cursor) {
            Row::Effort => self.cycle_effort(dir),
            Row::Scope => self.cycle_scope(dir),
            Row::CoAuthor => self.co_author = !self.co_author,
            Row::Variant => self.cycle_variant(dir),
            Row::AutoCompact => self.cycle_auto_compact(dir),
            Row::UsageRefresh => self.refresh_usage(true),
            _ => {}
        }
    }

    fn cycle_effort(&mut self, dir: i32) {
        let n = EFFORTS.len() as i32;
        self.effort_idx = ((self.effort_idx as i32 + dir).rem_euclid(n)) as usize;
    }
    fn cycle_scope(&mut self, dir: i32) {
        let n = SCOPES.len() as i32;
        self.scope_idx = ((self.scope_idx as i32 + dir).rem_euclid(n)) as usize;
    }
    fn cycle_variant(&mut self, dir: i32) {
        let n = self.variant_aliases.len() as i32;
        if n == 0 {
            return;
        }
        self.variant_idx = ((self.variant_idx as i32 + dir).rem_euclid(n)) as usize;
        self.refresh_auto_compact_windows();
        self.refresh_usage(false);
    }
    fn refresh_auto_compact_windows(&mut self) {
        let current = self.auto_compact_window();
        let alias = if self.has_variant_row {
            self.variant_aliases[self.variant_idx].0
        } else {
            self.target.as_str()
        };
        self.auto_compact_windows = supported_auto_compact_windows_for(&self.template_type, alias);
        self.auto_compact_idx = current
            .and_then(|value| self.auto_compact_windows.iter().position(|x| *x == value))
            .unwrap_or(0);
        let total = self.total_rows();
        if total > 0 && self.cursor >= total {
            self.cursor = total - 1;
        }
    }
    fn cycle_auto_compact(&mut self, dir: i32) {
        let n = self.auto_compact_windows.len() as i32;
        if n == 0 {
            return;
        }
        self.auto_compact_idx = ((self.auto_compact_idx as i32 + dir).rem_euclid(n)) as usize;
    }

    fn build_apply(&mut self) -> Outcome {
        let Some(idx) = self.selected_key else {
            self.mode = Mode::Message("No key selected — add one first (n or ➕).".into());
            return Outcome::Continue;
        };
        let Some(src) = self.sources.get(idx).cloned() else {
            self.mode = Mode::Message("Selected key no longer available.".into());
            return Outcome::Continue;
        };
        // touch last-used for saved credentials
        if let ApiKeySource::Saved { credential } = &src
            && let Ok(store) = CredentialStore::new()
        {
            let _ = store.touch_last_used(credential.id());
        }
        Outcome::Apply(ApplySelection {
            key: ApiKeyChoice {
                key: src.api_key().to_string(),
                source: Some(src.to_key_ref()),
            },
            effort: Some(self.effort().to_string()),
            scope: self.scope(),
            co_author_off: !self.co_author,
            variant: if self.has_variant_row {
                Some(self.variant_aliases[self.variant_idx].0.to_string())
            } else {
                None
            },
            auto_compact_window: self.auto_compact_window(),
        })
    }

    fn try_delete(&mut self) {
        let idx = match self.row_at(self.cursor) {
            Row::Key(i) => i,
            _ => return,
        };
        match self.sources.get(idx) {
            Some(ApiKeySource::Saved { .. }) => self.mode = Mode::ConfirmDelete { idx },
            Some(ApiKeySource::EnvVar { .. }) => {
                self.mode = Mode::Message("Can't delete an env-var key from here.".into());
            }
            None => {}
        }
    }

    fn try_rename(&mut self) {
        let idx = match self.row_at(self.cursor) {
            Row::Key(i) => i,
            _ => return,
        };
        match self.sources.get(idx) {
            Some(ApiKeySource::Saved { credential }) => {
                self.mode = Mode::InputRename {
                    idx,
                    input: TextInput::new(credential.name()),
                };
            }
            Some(ApiKeySource::EnvVar { .. }) => {
                self.mode = Mode::Message("Can't rename an env-var key.".into());
            }
            None => {}
        }
    }

    fn reload_sources(&mut self) {
        if let Ok(src) = collect_api_key_sources(&self.template_type) {
            self.sources = src;
        }
        // keep cursor / selection valid
        let total = self.total_rows();
        if self.cursor >= total && total > 0 {
            self.cursor = total - 1;
        }
        if let Some(s) = self.selected_key
            && s >= self.sources.len()
        {
            self.selected_key = if self.sources.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    fn handle_input_newkey(&mut self, key: KeyEvent, mut input: TextInput) -> Outcome {
        match key.code {
            // Esc / Enter terminate input mode. They must return early so the
            // "keep editing" restore below doesn't re-enter the input mode —
            // that was the bug where the create-key page could never be left.
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                return Outcome::Continue;
            }
            KeyCode::Enter => {
                let value = input.value().trim().to_string();
                if value.is_empty() {
                    self.mode = Mode::Message("API key cannot be empty.".into());
                    return Outcome::Continue;
                }
                let tt = self.template_type.clone();
                match CredentialStore::new() {
                    Ok(store) => match store.create_credential_smart(&value, tt, None) {
                        Ok(cred) => {
                            self.mode = Mode::Normal;
                            self.reload_sources();
                            // select the freshly added key
                            if let Some(i) = self
                                .sources
                                .iter()
                                .position(|s| s.api_key() == cred.api_key())
                            {
                                self.selected_key = Some(i);
                                self.cursor = i;
                            }
                        }
                        Err(e) => self.mode = Mode::Message(format!("Failed to save: {e}")),
                    },
                    Err(e) => self.mode = Mode::Message(format!("Credential store error: {e}")),
                }
                return Outcome::Continue;
            }
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Left => input.move_left(),
            KeyCode::Right => input.move_right(),
            KeyCode::Home => input.move_start(),
            KeyCode::End => input.move_end(),
            KeyCode::Char(c) => input.insert(c),
            _ => {}
        }
        // Editing keys: stay in input mode with the (possibly modified) input.
        self.mode = Mode::InputNewKey(input);
        Outcome::Continue
    }

    fn handle_input_rename(&mut self, key: KeyEvent, idx: usize, mut input: TextInput) -> Outcome {
        match key.code {
            // Esc / Enter terminate input mode (see handle_input_newkey).
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                return Outcome::Continue;
            }
            KeyCode::Enter => {
                let new_name = input.value().trim().to_string();
                if let Some(ApiKeySource::Saved { credential }) = self.sources.get(idx).cloned() {
                    if new_name.is_empty() {
                        self.mode = Mode::Message("Name cannot be empty.".into());
                        return Outcome::Continue;
                    }
                    if new_name != credential.name() {
                        match CredentialStore::new() {
                            Ok(store) => {
                                if let Err(e) = store.update_name(credential.id(), new_name) {
                                    self.mode = Mode::Message(format!("Rename failed: {e}"));
                                    return Outcome::Continue;
                                }
                            }
                            Err(e) => {
                                self.mode = Mode::Message(format!("Credential store error: {e}"));
                                return Outcome::Continue;
                            }
                        }
                    }
                }
                self.mode = Mode::Normal;
                self.reload_sources();
                return Outcome::Continue;
            }
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Left => input.move_left(),
            KeyCode::Right => input.move_right(),
            KeyCode::Home => input.move_start(),
            KeyCode::End => input.move_end(),
            KeyCode::Char(c) => input.insert(c),
            _ => {}
        }
        // Editing keys: stay in input mode with the (possibly modified) input.
        self.mode = Mode::InputRename { idx, input };
        Outcome::Continue
    }

    fn handle_confirm_delete(&mut self, key: KeyEvent, idx: usize) -> Outcome {
        match key.code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(ApiKeySource::Saved { credential }) = self.sources.get(idx).cloned() {
                    match CredentialStore::new() {
                        Ok(store) => {
                            if let Err(e) = store.delete_credential(credential.id()) {
                                self.mode = Mode::Message(format!("Delete failed: {e}"));
                                return Outcome::Continue;
                            }
                        }
                        Err(e) => {
                            self.mode = Mode::Message(format!("Credential store error: {e}"));
                            return Outcome::Continue;
                        }
                    }
                }
                self.mode = Mode::Normal;
                self.reload_sources();
            }
            _ => self.mode = Mode::Normal, // Esc / n / anything else cancels
        }
        Outcome::Continue
    }

    /// Borrow the current mode for rendering (input fields, popups).
    pub fn mode_ref(&self) -> &Mode {
        &self.mode
    }
}

#[cfg(test)]
mod snapshot_tests {
    //! Render the TUI to an in-memory buffer (ratatui `TestBackend`) and dump
    //! the text, so rendering can be inspected without a real terminal.
    //! Run: `cargo test snapshot_states -- --nocapture`.

    use super::*;
    use crate::credentials::CredentialData;
    use crate::templates::TemplateType;
    use crate::usage::{UsageReport, UsageStatus};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::{Terminal, backend::TestBackend, layout::Position};

    fn cred(name: &str, key: &str, last_used: Option<&str>) -> SavedCredentialStub {
        // SavedCredential == CredentialData
        let mut c = CredentialData::new(name.to_string(), key.to_string(), TemplateType::Zai);
        c.last_used_at = last_used.map(|s| s.to_string());
        c
    }
    // alias for clarity (SavedCredential is CredentialData)
    type SavedCredentialStub = CredentialData;

    fn base_app() -> App {
        App {
            template_type: TemplateType::Zai,
            target: "zai".into(),
            display_name: "ZAI China (智谱AI)".into(),
            current_label: "deepseek".into(),
            sources: vec![
                ApiKeySource::Saved {
                    credential: cred(
                        "work",
                        "sk-workkey-abcdef-1234-5678",
                        Some("2026-06-12 10:00:00 UTC"),
                    ),
                },
                ApiKeySource::Saved {
                    credential: cred("personal", "sk-personal-0987-6543-2100", None),
                },
                ApiKeySource::EnvVar {
                    env_var_name: "Z_AI_API_KEY".to_string(),
                    api_key: "sk-envvar-zzzz-yyyy-xxxx".to_string(),
                },
            ],
            selected_key: Some(0),
            cursor: 0,
            effort_idx: 0,
            scope_idx: 0,
            co_author: false,
            variant_aliases: variant_options(&TemplateType::Zai),
            variant_idx: 0,
            has_variant_row: true,
            auto_compact_windows: vec![
                AutoCompactWindow::K896,
                AutoCompactWindow::K768,
                AutoCompactWindow::K512,
                AutoCompactWindow::K256,
            ],
            auto_compact_idx: 0,
            usage_cache: HashMap::new(),
            usage_in_flight: None,
            usage_status: UsageStatus::Loading {
                stale: None,
                spinner: '-',
            },
            usage_status_key: None,
            spinner_idx: 0,
            mode: Mode::Normal,
        }
    }

    fn with_usage(mut app: App, status: UsageStatus) -> App {
        app.usage_status = status;
        app
    }

    fn usage_report(provider: &str, summary: &str) -> UsageReport {
        UsageReport {
            provider: provider.to_string(),
            summary: summary.to_string(),
        }
    }

    fn render(app: &App, w: u16, h: u16) -> String {
        let backend = TestBackend::new(w, h);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| crate::tui::view::render(f, app)).unwrap();
        let buf = terminal.backend().buffer();
        let area = buf.area();
        let mut out = String::new();
        for y in 0..area.height {
            let mut row = String::new();
            for x in 0..area.width {
                let sym = buf
                    .cell(Position { x, y })
                    .map(|c| c.symbol())
                    .unwrap_or(" ");
                row.push_str(sym);
            }
            out.push_str(row.trim_end());
            out.push('\n');
        }
        out
    }

    fn key(c: KeyCode) -> KeyEvent {
        KeyEvent::new(c, KeyModifiers::NONE)
    }

    fn banner(title: &str, body: &str) {
        println!("\n================= {title} =================\n{body}");
    }

    #[test]
    fn snapshot_states() {
        // 1. initial
        let app = base_app();
        banner(
            "1. INITIAL (cursor on work key, zai generic → variant row)",
            &render(&app, 76, 22),
        );

        // 2. cursor into options
        let mut app = base_app();
        for _ in 0..6 {
            app.handle_event(key(KeyCode::Down));
        }
        banner("2. AFTER Down x6", &render(&app, 76, 22));

        // 3. change effort
        let mut app = base_app();
        app.cursor = 4; // Effort row (3 keys + newkey + effort)
        app.handle_event(key(KeyCode::Left));
        banner("3. EFFORT changed via Left (→ low)", &render(&app, 76, 22));

        // 4. new-key input
        let mut app = base_app();
        app.handle_event(key(KeyCode::Char('n')));
        banner("4. NEW-KEY INPUT open", &render(&app, 76, 22));

        // 5. help
        let mut app = base_app();
        app.handle_event(key(KeyCode::Char('?')));
        banner("5. HELP overlay", &render(&app, 76, 22));

        // 6. confirm delete
        let mut app = base_app();
        app.cursor = 0;
        app.handle_event(key(KeyCode::Char('d')));
        banner("6. CONFIRM DELETE popup", &render(&app, 76, 22));

        // 7. no keys
        let mut app = base_app();
        app.sources.clear();
        app.selected_key = None;
        app.cursor = 0;
        app.refresh_usage(false);
        banner("7. NO KEYS (cursor on ➕)", &render(&app, 76, 22));

        // 8. narrow terminal
        let app = base_app();
        banner("8. NARROW 52x18", &render(&app, 52, 18));

        // 9. usage loading with stale cache
        let app = with_usage(
            base_app(),
            UsageStatus::Loading {
                stale: Some(usage_report(
                    "ZHIPU",
                    "5h 42%@18:00 | wk 150/1000 15%@06-22 00:00 | MCP 2/30 7%",
                )),
                spinner: '|',
            },
        );
        banner("9. USAGE LOADING", &render(&app, 76, 22));

        // 10. usage ready
        let app = with_usage(
            base_app(),
            UsageStatus::Ready(usage_report(
                "ZHIPU",
                "5h 42%@18:00 | wk 150/1000 15%@06-22 00:00 | MCP 2/30 7%",
            )),
        );
        banner("10. USAGE READY", &render(&app, 76, 22));

        // 11. usage error with stale data
        let app = with_usage(
            base_app(),
            UsageStatus::Error {
                message: "request failed".to_string(),
                stale: Some(usage_report(
                    "DeepSeek",
                    "available | CNY 12.34 (grant 2.00, top-up 10.34)",
                )),
            },
        );
        banner("11. USAGE ERROR", &render(&app, 76, 22));
    }

    #[test]
    fn newkey_input_can_be_exited_and_normal_keys_resume() {
        let mut app = base_app();

        // 'n' opens the create-key input.
        app.handle_event(key(KeyCode::Char('n')));
        assert!(
            matches!(app.mode, Mode::InputNewKey(_)),
            "'n' should open the new-key input"
        );

        // Typing is captured into the field.
        for c in "sk-test".chars() {
            app.handle_event(key(KeyCode::Char(c)));
        }
        match &app.mode {
            Mode::InputNewKey(input) => assert_eq!(input.value(), "sk-test"),
            _ => panic!("expected to still be in new-key input while typing"),
        }

        // Esc must leave the input page. Before the fix the page stayed open
        // (the "keep editing" guard re-entered InputNewKey) and every key was
        // swallowed as text — the user could neither exit nor navigate.
        app.handle_event(key(KeyCode::Esc));
        assert!(
            matches!(app.mode, Mode::Normal),
            "Esc must exit the new-key input back to Normal"
        );

        // In Normal mode, '?' opens Help instead of being inserted as text.
        app.handle_event(key(KeyCode::Char('?')));
        assert!(
            matches!(app.mode, Mode::Help),
            "post-Esc keys must act on Normal mode, not be typed"
        );
    }

    #[test]
    fn newkey_input_render_shows_typed_text() {
        let mut app = base_app();
        app.handle_event(key(KeyCode::Char('n')));
        for c in "sk-abcd-1234".chars() {
            app.handle_event(key(KeyCode::Char(c)));
        }
        let screen = render(&app, 70, 16);
        banner("NEW-KEY INPUT (typed sk-abcd-1234)", &screen);
        assert!(screen.contains("Create new API key"));
        assert!(screen.contains("sk-abcd-1234"));
        assert!(screen.contains("Esc"));
    }

    #[test]
    fn rename_input_esc_exits_to_normal() {
        let mut app = base_app();
        app.cursor = 0; // first row is a saved credential
        app.handle_event(key(KeyCode::Char('r')));
        assert!(
            matches!(app.mode, Mode::InputRename { .. }),
            "'r' on a saved key should open rename input"
        );
        // Same guard bug used to trap rename too — Esc must leave it.
        app.handle_event(key(KeyCode::Esc));
        assert!(
            matches!(app.mode, Mode::Normal),
            "Esc must exit rename input back to Normal"
        );
    }

    #[test]
    fn auto_compact_row_changes_preview_and_selection() {
        let mut app = base_app();
        app.cursor = 8; // 3 keys + new-key + effort + scope + co-author + variant

        app.handle_event(key(KeyCode::Right));

        assert_eq!(app.auto_compact_window(), Some(AutoCompactWindow::K768));
        assert_eq!(
            app.preview_model_and_base().0,
            "glm-5.2[1m]",
            "auto compact must not remove the [1m] model suffix"
        );

        match app.handle_event(key(KeyCode::Char('a'))) {
            Outcome::Apply(selection) => {
                assert_eq!(selection.auto_compact_window, Some(AutoCompactWindow::K768));
            }
            _ => panic!("expected apply outcome"),
        }
    }

    #[test]
    fn auto_compact_row_requires_supported_1m_template() {
        let mut app = base_app();
        app.template_type = TemplateType::DeepSeek;
        app.target = "deepseek".into();
        app.variant_aliases.clear();
        app.has_variant_row = false;
        app.refresh_auto_compact_windows();

        assert_eq!(app.preview_model_and_base().0, "deepseek-v4-pro[1m]");
        assert!(!app.has_auto_compact_row());
        assert_eq!(app.auto_compact_window(), None);
    }

    #[test]
    fn usage_snapshot_states_cover_loading_ready_and_stale_error() {
        let loading = render(
            &with_usage(
                base_app(),
                UsageStatus::Loading {
                    stale: Some(usage_report(
                        "ZHIPU",
                        "5h 42%@18:00 | wk 150/1000 15%@06-22 00:00 | MCP 2/30 7%",
                    )),
                    spinner: '/',
                },
            ),
            76,
            22,
        );
        assert!(loading.contains("usage: / loading"));
        assert!(loading.contains("cached: ZHIPU 5h 42%"));
        assert!(!loading.contains("24h"));

        let ready = render(
            &with_usage(
                base_app(),
                UsageStatus::Ready(usage_report("DeepSeek", "available | CNY 12.34")),
            ),
            76,
            22,
        );
        assert!(ready.contains("usage: DeepSeek available | CNY 12.34"));

        let error = render(
            &with_usage(
                base_app(),
                UsageStatus::Error {
                    message: "request failed".to_string(),
                    stale: Some(usage_report("DeepSeek", "available | CNY 12.34")),
                },
            ),
            76,
            22,
        );
        assert!(error.contains("usage: error: request failed"));
        assert!(error.contains("cached: DeepSeek available | CNY 12.34"));
    }

    #[test]
    fn usage_loading_state_keeps_stale_cache_visible() {
        let mut app = base_app();
        let key = app.selected_usage_key().unwrap();
        app.usage_cache.insert(
            key.clone(),
            UsageCacheEntry {
                report: UsageReport {
                    provider: "ZHIPU".to_string(),
                    summary: "5h 42%@18:00".to_string(),
                },
                fetched_at: Instant::now() - USAGE_CACHE_TTL - Duration::from_secs(1),
            },
        );
        app.usage_status_key = Some(key.clone());
        app.usage_status = UsageStatus::Loading {
            stale: None,
            spinner: '-',
        };
        let (_sender, receiver) = mpsc::channel();
        app.usage_in_flight = Some(UsageInFlight {
            key: key.clone(),
            receiver,
        });

        app.tick();

        match app.usage_status() {
            UsageStatus::Loading { stale, .. } => {
                assert_eq!(
                    stale.as_ref().map(|report| report.summary.as_str()),
                    Some("5h 42%@18:00")
                );
            }
            other => panic!("expected loading usage status, got {other:?}"),
        }
    }
}
