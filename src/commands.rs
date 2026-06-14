use crate::{
    Configurable, CredentialManager, cli,
    credentials::{CredentialStore, mask_api_key, resolve_api_key},
    prefs::{KeyRef, Prefs},
    settings::{Attribution, ClaudeSettings},
    snapshots::{self, SnapshotScope, SnapshotStore},
    templates::{
        AutoCompactWindow, TemplateType, get_all_templates, get_template_instance,
        get_template_instance_with_input, get_template_type, is_generic_target,
        supports_auto_compact_option, variant_options,
    },
    utils::{
        backup_settings, confirm_action, get_credentials_dir, get_settings_path, get_snapshots_dir,
    },
};
use anyhow::{Result, anyhow};
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;

/// Common environment variables that should be added to all templates
fn get_common_env_vars() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("ENABLE_TOOL_SEARCH".to_string(), "true".to_string());
    env
}

/// Inject common environment variables into settings.
/// Does not overwrite keys that are already set by the template.
fn inject_common_env_vars(settings: &mut ClaudeSettings) {
    if let Some(ref mut env) = settings.env {
        for (key, value) in get_common_env_vars() {
            env.entry(key).or_insert(value);
        }
        #[cfg(target_os = "windows")]
        {
            env.entry("CLAUDE_CODE_USE_POWERSHELL_TOOL".to_string())
                .or_insert_with(|| "1".to_string());
        }
    } else {
        let mut env = get_common_env_vars();
        #[cfg(target_os = "windows")]
        {
            env.insert(
                "CLAUDE_CODE_USE_POWERSHELL_TOOL".to_string(),
                "1".to_string(),
            );
        }
        settings.env = Some(env);
    }
}

/// Get Git Bash path for CLAUDE_CODE_GIT_BASH_PATH (Windows only).
///
/// Checks template settings and system env first. In non-interactive mode,
/// auto-detects. In interactive mode, prompts the user. Returns None if already
/// set or skipped.
#[cfg(target_os = "windows")]
fn get_git_bash_path(settings: &ClaudeSettings, cli_mode: bool) -> Result<Option<String>> {
    if settings
        .env
        .as_ref()
        .is_some_and(|e| e.contains_key("CLAUDE_CODE_GIT_BASH_PATH"))
    {
        return Ok(None);
    }

    if let Ok(value) = std::env::var("CLAUDE_CODE_GIT_BASH_PATH") {
        return Ok(Some(value));
    }

    let detected = crate::utils::detect_git_bash_paths();

    if cli_mode {
        return Ok(detected.into_iter().next().map(|p| p.display().to_string()));
    }

    let mut options: Vec<String> = detected.iter().map(|p| p.display().to_string()).collect();
    options.push("Enter custom path...".to_string());
    options.push("Skip".to_string());

    let selection = match inquire::Select::new(
        "Select Git Bash path for CLAUDE_CODE_GIT_BASH_PATH:",
        options,
    )
    .prompt()
    {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    if selection == "Skip" {
        return Ok(None);
    }

    if selection == "Enter custom path..." {
        let custom_path = match inquire::Text::new("Enter Git Bash path:")
            .with_validator(|input: &str| {
                let path = PathBuf::from(input.trim());
                if path.exists() {
                    Ok(inquire::validator::Validation::Valid)
                } else {
                    Ok(inquire::validator::Validation::Invalid(
                        "Path does not exist".into(),
                    ))
                }
            })
            .prompt()
        {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        return Ok(Some(custom_path));
    }

    Ok(Some(selection))
}

#[cfg(not(target_os = "windows"))]
fn get_git_bash_path(_settings: &ClaudeSettings, _cli_mode: bool) -> Result<Option<String>> {
    Ok(None)
}

/// Run a command based on CLI arguments
pub fn run_command(args: &crate::Cli) -> Result<()> {
    match &args.command {
        cli::Commands::List => list_command()?,
        cli::Commands::Apply {
            target,
            scope,
            model,
            settings_path,
            backup,
            yes,
            cli,
            effort,
            auto_compact,
            api_key,
            no_co_author,
            switch_key,
            dry_run,
            variant,
        } => apply_command(
            target,
            scope,
            model,
            settings_path,
            *backup,
            *yes,
            *cli,
            effort,
            auto_compact,
            api_key,
            *no_co_author,
            *switch_key,
            *dry_run,
            variant,
        )?,
        cli::Commands::Credentials { command } => match command {
            cli::CredentialCommands::List => credentials_list_command()?,
            cli::CredentialCommands::Clear { yes } => credentials_clear_command(*yes)?,
        },
        cli::Commands::Config(cfg) => config_command(cfg)?,
        cli::Commands::Current => current_command()?,
    }
    Ok(())
}

/// List available snapshots
pub fn list_command() -> Result<()> {
    println!("📸 Snapshot Browser");
    println!();

    let mut selector = crate::selectors::snapshot::SnapshotSelector::new()?;

    match selector.run_management() {
        Ok(()) => println!("\n👋 Goodbye!"),
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("User cancelled selection") {
                println!("\n👋 Cancelled. See you next time!");
            } else {
                println!("\n❌ Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Create a snapshot
pub fn snap_command(
    name: &str,
    scope: &SnapshotScope,
    settings_path: &Option<PathBuf>,
    description: &Option<String>,
    overwrite: bool,
) -> Result<()> {
    let settings_path = get_settings_path(settings_path.clone());
    let settings = ClaudeSettings::from_file(&settings_path)?;

    let mut snapshot_settings = settings;
    if matches!(scope, SnapshotScope::All | SnapshotScope::Env) {
        snapshot_settings.env = Some(ClaudeSettings::capture_environment());
    }

    let snapshots_dir = get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    if store.exists_by_name(name)
        && !overwrite
        && !confirm_action(
            &format!("Snapshot '{}' already exists. Overwrite?", name),
            false,
        )?
    {
        return Ok(());
    }

    let snapshot = snapshots::Snapshot::new(
        name.to_string(),
        snapshot_settings,
        scope.clone(),
        description.clone(),
    );

    store.save(&snapshot)?;
    println!(
        "{} Snapshot '{}' created successfully!",
        style("✓").green().bold(),
        name
    );

    Ok(())
}

// ── apply ────────────────────────────────────────────────────────────────────

/// Apply a snapshot or template
#[allow(clippy::too_many_arguments)]
pub fn apply_command(
    target: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &Option<PathBuf>,
    backup: bool,
    yes: bool,
    cli: bool,
    effort: &Option<String>,
    auto_compact: &Option<String>,
    api_key: &Option<String>,
    no_co_author: bool,
    switch_key: bool,
    dry_run: bool,
    variant: &Option<String>,
) -> Result<()> {
    let settings_path = get_settings_path(settings_path.clone());

    // Try to parse as a template first
    if let Ok(template_type) = get_template_type(target) {
        return apply_template_command(
            &template_type,
            target,
            scope,
            model,
            &settings_path,
            backup,
            yes,
            cli,
            effort,
            auto_compact,
            api_key,
            no_co_author,
            switch_key,
            dry_run,
            variant,
        );
    }

    // Otherwise treat as a snapshot name
    apply_snapshot_command(target, scope, model, &settings_path, backup, yes)
}

/// One-time first-run onboarding for global defaults.
fn onboard_prefs(prefs: &mut Prefs) -> Result<()> {
    println!(
        "{} First run — setting your defaults (saved to {})",
        style("👋").cyan(),
        Prefs::path().display()
    );

    let effort_options = vec!["max", "xhigh", "high", "medium", "low"];
    let effort = inquire::Select::new("Default reasoning effort:", effort_options)
        .with_help_message("Controls Claude Code thinking depth")
        .prompt()
        .map_err(|e| anyhow!("Failed to read effort: {}", e))?;
    prefs.default_effort = Some(effort.to_string());

    let co_author = inquire::Confirm::new("Enable co-authored-by in git commits/PRs?")
        .with_default(false)
        .prompt()
        .unwrap_or(false);
    prefs.default_co_author = co_author;

    Ok(())
}

/// Resolve effort: flag → prefs default → (non-interactive fallback) → None.
fn resolve_effort(flag: Option<&str>, prefs: &Prefs, non_interactive: bool) -> Option<String> {
    if let Some(e) = flag.map(str::trim).filter(|e| !e.is_empty()) {
        return Some(e.to_string());
    }
    if let Some(e) = &prefs.default_effort {
        return Some(e.clone());
    }
    if non_interactive {
        return Some("xhigh".to_string());
    }
    None
}

/// Resolve whether co-author should be OFF: --no-co-author → off; else honor
/// prefs (default_co_author false means off).
fn resolve_co_author_off(no_co_author: bool, prefs: &Prefs) -> bool {
    no_co_author || !prefs.default_co_author
}

/// Resolve the auto-compaction threshold for providers that expose it.
/// Returns `None` for providers that do not support it.
fn resolve_auto_compact_window(
    template_type: &TemplateType,
    template: &dyn crate::templates::Template,
    flag: Option<&str>,
    prefs: &Prefs,
) -> Result<Option<AutoCompactWindow>> {
    if !supports_auto_compact_option(template) {
        if let Some(value) = flag.map(str::trim).filter(|v| !v.is_empty()) {
            return Err(anyhow!(
                "{} does not support auto-compact thresholds (got '{}')",
                template.display_name(),
                value
            ));
        }
        return Ok(None);
    }

    let supported = template.supported_auto_compact_windows();

    let parsed = if let Some(value) = flag.map(str::trim).filter(|v| !v.is_empty()) {
        Some(value.parse::<AutoCompactWindow>()?)
    } else {
        prefs
            .template_pref(template_type)
            .and_then(|p| {
                p.last_auto_compact_window
                    .as_deref()
                    .or(p.last_context_window.as_deref())
            })
            .map(str::parse::<AutoCompactWindow>)
            .transpose()?
    };

    let auto_compact_window = parsed
        .or_else(|| template.default_auto_compact_window())
        .expect("supported auto-compact windows should have a default");

    if !supported.contains(&auto_compact_window) {
        let choices = supported
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(anyhow!(
            "{} does not support auto-compact threshold '{}'. Use one of: {}",
            template.display_name(),
            auto_compact_window,
            choices
        ));
    }

    Ok(Some(auto_compact_window))
}

/// Resolve the variant alias for a generic target (remembering / prompting).
/// Returns `None` for specific aliases or templates with no variants (use the
/// target as-is).
fn resolve_variant_alias(
    template_type: &TemplateType,
    target: &str,
    variant_flag: Option<&str>,
    prefs: &mut Prefs,
    non_interactive: bool,
) -> Result<Option<String>> {
    let options = variant_options(template_type);
    if options.is_empty() || !is_generic_target(target) {
        return Ok(None);
    }

    // Explicit --variant alias
    if let Some(v) = variant_flag.map(str::trim).filter(|v| !v.is_empty()) {
        prefs.set_variant(template_type, Some(v.to_string()));
        return Ok(Some(v.to_string()));
    }

    // Remembered variant alias
    if let Some(v) = prefs
        .template_pref(template_type)
        .and_then(|p| p.variant.clone())
    {
        return Ok(Some(v));
    }

    // Prompt (only on the fast/non-interactive-skipped path)
    if non_interactive {
        let hint = options
            .iter()
            .map(|(a, _)| *a)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(anyhow!(
            "Non-interactive mode requires a variant for '{}'. Use one of: {} (or --variant)",
            target,
            hint
        ));
    }

    let labels: Vec<String> = options
        .iter()
        .map(|(alias, label)| format!("{}  ({})", label, alias))
        .collect();
    let choice = inquire::Select::new(
        &format!("Select {} variant:", template_type),
        labels.clone(),
    )
    .with_help_message("↑/↓ navigate, Enter select, Esc cancel")
    .prompt()
    .map_err(|e| anyhow!("Variant selection failed: {}", e))?;
    let idx = labels.iter().position(|l| l == &choice).unwrap();
    let alias = options[idx].0.to_string();
    prefs.set_variant(template_type, Some(alias.clone()));
    Ok(Some(alias))
}

/// Detect which known provider is currently active in settings.json.
fn detect_current_provider() -> Option<TemplateType> {
    let settings_path = get_settings_path(None);
    let settings = ClaudeSettings::from_file(&settings_path).ok()?;
    let base_url = settings
        .env
        .as_ref()
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .cloned();
    get_all_templates().into_iter().find(|tt| {
        get_template_instance(tt)
            .api_host()
            .is_some_and(|host| base_url.as_deref().is_some_and(|u| u.contains(host)))
    })
}

fn detect_current_provider_label() -> String {
    detect_current_provider()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "none".to_string())
}

/// Print a concise summary of what is being applied.
fn print_apply_summary(
    template_type: &TemplateType,
    settings: &ClaudeSettings,
    key: &str,
    auto_compact_window: Option<AutoCompactWindow>,
) {
    println!();
    println!("{} applying '{}'", style("•").cyan(), template_type);
    if let Some(m) = &settings.model {
        println!("  model:  {}", m);
    }
    println!("  key:    {}", mask_api_key(key));
    if let Some(e) = &settings.effort_level {
        println!("  effort: {}", e);
    }
    if let Some(auto_compact_window) = auto_compact_window {
        println!("  compact: {}", auto_compact_window);
    }
    if let Some(base) = settings
        .env
        .as_ref()
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
    {
        println!("  base:   {}", base);
    }
}

/// Apply a template
#[allow(clippy::too_many_arguments)]
fn apply_template_command(
    template_type: &TemplateType,
    target: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &PathBuf,
    backup: bool,
    yes: bool,
    cli: bool,
    effort: &Option<String>,
    auto_compact: &Option<String>,
    api_key: &Option<String>,
    no_co_author: bool,
    switch_key: bool,
    dry_run: bool,
    variant: &Option<String>,
) -> Result<()> {
    let non_interactive = cli || !atty::is(atty::Stream::Stdin);
    // Interactive TUI when on a TTY, not forced via flags, and not --yes.
    let use_tui = !non_interactive && !yes;
    let mut prefs = Prefs::load_or_default();

    // Gather intent: (variant alias, key, effort, compact, scope, co-author-off).
    let (variant_alias, key_choice, effort, auto_compact_window, scope, co_author_off) = if use_tui
    {
        let display = get_template_instance(template_type)
            .display_name()
            .to_string();
        let current_label = detect_current_provider_label();
        match crate::tui::run_apply_tui(
            template_type.clone(),
            target.to_string(),
            display,
            current_label,
            &prefs,
        )? {
            Some(sel) => (
                sel.variant,
                sel.key,
                sel.effort,
                sel.auto_compact_window,
                sel.scope,
                sel.co_author_off,
            ),
            None => {
                println!("Cancelled.");
                return Ok(());
            }
        }
    } else {
        // Fast / scripted path.
        if !Prefs::exists() && !non_interactive {
            onboard_prefs(&mut prefs)?;
        }
        let va = resolve_variant_alias(
            template_type,
            target,
            variant.as_deref(),
            &mut prefs,
            non_interactive,
        )?;
        let remembered_key: Option<KeyRef> = prefs
            .template_pref(template_type)
            .and_then(|p| p.last_key.clone());
        let kc = resolve_api_key(
            template_type,
            api_key.as_deref(),
            remembered_key.as_ref(),
            switch_key,
            non_interactive,
        )?
        .ok_or_else(|| anyhow!("Cancelled"))?;
        prefs.set_last_key(template_type, kc.source.clone());
        let eff = resolve_effort(effort.as_deref(), &prefs, non_interactive);
        let preview_template =
            get_template_instance_with_input(template_type, va.as_deref().unwrap_or(target));
        let compact = resolve_auto_compact_window(
            template_type,
            preview_template.as_ref(),
            auto_compact.as_deref(),
            &prefs,
        )?;
        let cao = resolve_co_author_off(no_co_author, &prefs);
        (va, kc, eff, compact, scope.clone(), cao)
    };

    // Build template settings from the resolved alias + key + scope.
    let template_instance =
        get_template_instance_with_input(template_type, variant_alias.as_deref().unwrap_or(target));
    let mut settings = template_instance.create_settings_with_auto_compact(
        &key_choice.key,
        &scope,
        auto_compact_window,
    )?;
    inject_common_env_vars(&mut settings);

    // Windows: CLAUDE_CODE_GIT_BASH_PATH — always auto-detect (never prompt;
    // interactive selection now happens in the TUI / flags, not here).
    if let Some(git_bash_path) = get_git_bash_path(&settings, true)? {
        settings
            .env
            .get_or_insert_with(HashMap::new)
            .insert("CLAUDE_CODE_GIT_BASH_PATH".to_string(), git_bash_path);
    }

    // --model override
    if let Some(model_name) = model {
        settings.model = Some(model_name.clone());
    }

    // effort + co-author from the resolved selection
    settings.effort_level = effort.clone();
    settings.attribution = if co_author_off {
        Some(Attribution {
            commit: Some(String::new()),
            pr: Some(String::new()),
        })
    } else {
        None
    };

    // Merge by scope (preserves unrelated keys/fields).
    let existing = ClaudeSettings::from_file(settings_path)?;
    let merged = ClaudeSettings::merge_by_scope(existing, settings, &scope);

    if backup {
        backup_settings(settings_path)?;
    }

    print_apply_summary(template_type, &merged, &key_choice.key, auto_compact_window);

    if dry_run {
        println!("{} (dry-run — no changes written)", style("•").yellow());
        prefs.save()?;
        return Ok(());
    }

    merged.to_file(settings_path)?;
    // Remember this apply for next time.
    prefs.record_apply(
        template_type,
        variant_alias.clone(),
        key_choice.source.clone(),
        scope.clone(),
        effort.clone(),
        !co_author_off,
        auto_compact_window,
    );
    prefs.save()?;

    println!(
        "{} Applied '{}' — wrote {}",
        style("✓").green().bold(),
        template_type,
        settings_path.display()
    );
    Ok(())
}

/// Apply a snapshot (replace-within-scope; snapshots are deliberate restore points)
fn apply_snapshot_command(
    snapshot_name: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &PathBuf,
    backup: bool,
    yes: bool,
) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    let mut snapshot = store.load_by_name(snapshot_name)?;

    snapshot.settings = snapshot.settings.filter_by_scope(scope);

    if let Some(model_name) = model {
        snapshot.settings.model = Some(model_name.clone());
    }

    let existing_settings = ClaudeSettings::from_file(settings_path)?;

    if backup {
        backup_settings(settings_path)?;
    }

    if !yes {
        let existing_masked = existing_settings.clone().mask_sensitive_data();
        let snapshot_masked = snapshot.settings.clone().mask_sensitive_data();

        println!("Current settings:");
        println!(
            "{}",
            crate::settings::format_settings_for_display(&existing_masked, false)
        );
        println!("\nSnapshot settings:");
        println!(
            "{}",
            crate::settings::format_settings_for_display(&snapshot_masked, false)
        );

        let options = vec!["Apply", "Cancel"];
        let selection = inquire::Select::new("Confirm:", options)
            .prompt()
            .map_err(|_| anyhow!("Cancelled"))?;
        if selection == "Cancel" {
            return Ok(());
        }
    }

    snapshot.settings.to_file(settings_path)?;

    println!(
        "{} Applied snapshot '{}' successfully!",
        style("✓").green().bold(),
        snapshot_name
    );

    Ok(())
}

// ── credentials ──────────────────────────────────────────────────────────────

/// List saved credentials interactively
pub fn credentials_list_command() -> Result<()> {
    println!("🔐 Credential Browser");
    println!();

    let mut selector = crate::selectors::credential::CredentialSelector::new_all()?;

    match selector.run_management() {
        Ok(()) => println!("\n👋 Goodbye!"),
        Err(e) => {
            let error_str = e.to_string();
            if error_str.contains("User cancelled selection") {
                println!("\n👋 Cancelled. See you next time!");
            } else {
                println!("\n❌ Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Clear all credentials
pub fn credentials_clear_command(yes: bool) -> Result<()> {
    if !yes && !confirm_action("Clear all saved credentials?", false)? {
        return Ok(());
    }

    let _credentials_dir = get_credentials_dir();
    let credential_store = CredentialStore::new()?;

    credential_store.clear_credentials()?;

    println!("{} Cleared all credentials!", style("✓").green().bold());

    Ok(())
}

// ── config ───────────────────────────────────────────────────────────────────

/// View / edit persistent preferences.
pub fn config_command(cfg: &cli::ConfigArgs) -> Result<()> {
    let mut prefs = Prefs::load_or_default();

    if cfg.reset {
        prefs = Prefs::default();
        prefs.save()?;
        println!(
            "{} Reset all preferences to defaults.",
            style("✓").green().bold()
        );
        return Ok(());
    }

    let mut changed = false;
    if let Some(e) = cfg.effort.as_deref() {
        prefs.default_effort = Some(e.to_string());
        changed = true;
    }
    if let Some(co) = cfg.co_author {
        prefs.default_co_author = co;
        changed = true;
    }
    if let Some(scope) = cfg.scope.as_ref() {
        prefs.default_scope = scope.clone();
        changed = true;
    }

    if !changed && atty::is(atty::Stream::Stdin) {
        // No flags + interactive terminal → edit defaults via a menu.
        config_interactive(&mut prefs)?;
    }

    prefs.save()?;
    print_config(&prefs);
    Ok(())
}

fn config_interactive(prefs: &mut Prefs) -> Result<()> {
    let options = vec![
        "Edit default effort",
        "Edit co-author",
        "Edit default scope",
        "Done",
    ];
    loop {
        let choice = match inquire::Select::new("Preferences:", options.clone())
            .with_help_message("↑/↓ navigate, Enter select, Esc done")
            .prompt()
        {
            Ok(c) => c,
            Err(_) => break,
        };
        match choice {
            "Edit default effort" => {
                let efforts = vec!["max", "xhigh", "high", "medium", "low"];
                if let Ok(e) = inquire::Select::new("Default effort:", efforts).prompt() {
                    prefs.default_effort = Some(e.to_string());
                    prefs.save()?;
                    println!("{} default effort = {}", style("✓").green(), e);
                }
            }
            "Edit co-author" => {
                let co = inquire::Confirm::new("Enable co-authored-by?")
                    .with_default(prefs.default_co_author)
                    .prompt()
                    .unwrap_or(prefs.default_co_author);
                prefs.default_co_author = co;
                prefs.save()?;
                println!("{} co-author = {}", style("✓").green(), co);
            }
            "Edit default scope" => {
                let scopes = vec!["common", "env", "all"];
                if let Ok(s) = inquire::Select::new("Default scope:", scopes).prompt()
                    && let Ok(scope) = s.parse::<SnapshotScope>()
                {
                    prefs.default_scope = scope;
                    prefs.save()?;
                    println!("{} default scope = {}", style("✓").green(), s);
                }
            }
            _ => break,
        }
    }
    Ok(())
}

fn print_config(prefs: &Prefs) {
    println!();
    println!(
        "{} Preferences ({})",
        style("•").cyan(),
        Prefs::path().display()
    );
    println!(
        "  default effort:   {}",
        prefs.default_effort.as_deref().unwrap_or("(unset)")
    );
    println!(
        "  co-author:        {}",
        if prefs.default_co_author {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!("  default scope:    {}", prefs.default_scope);
    println!("  remembered templates: {}", prefs.templates.len());
}

// ── current ──────────────────────────────────────────────────────────────────

/// Show the currently-active provider detected from settings.json.
pub fn current_command() -> Result<()> {
    let settings_path = get_settings_path(None);
    let settings = ClaudeSettings::from_file(&settings_path)?;

    println!("📍 {}", settings_path.display());

    let base_url = settings
        .env
        .as_ref()
        .and_then(|e| e.get("ANTHROPIC_BASE_URL"))
        .cloned();

    match detect_current_provider() {
        Some(tt) => println!("Provider: {}", tt),
        None => println!("Provider: {}", style("(unknown / custom)").yellow()),
    }

    if let Some(m) = &settings.model {
        println!("Model:    {}", m);
    }
    if let Some(env) = &settings.env
        && let Some(k) = env
            .get("ANTHROPIC_AUTH_TOKEN")
            .or_else(|| env.get("ANTHROPIC_API_KEY"))
    {
        println!("Key:      {}", mask_api_key(k));
    }
    if let Some(e) = &settings.effort_level {
        println!("Effort:   {}", e);
    }
    if let Some(base) = &base_url {
        println!("Base URL: {}", base);
    }

    Ok(())
}
