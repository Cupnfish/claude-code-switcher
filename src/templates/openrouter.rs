//! OpenRouter AI provider template implementation
//!
//! OpenRouter provides access to multiple AI models through a unified API.
//! This template supports interactive model selection with free models prioritized.

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use serde::Deserialize;
use std::collections::HashMap;

/// OpenRouter model information
#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub pricing: Option<ModelPricing>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub context_length: Option<i64>,
}

/// Model pricing information
#[derive(Debug, Clone, Deserialize)]
pub struct ModelPricing {
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub completion: Option<String>,
}

impl OpenRouterModel {
    /// Check if this model is free (both prompt and completion prices are "0" or "0.0")
    pub fn is_free(&self) -> bool {
        if let Some(pricing) = &self.pricing {
            let prompt_free = pricing
                .prompt
                .as_ref()
                .map(|p| p == "0" || p == "0.0" || p == "0.00")
                .unwrap_or(false);
            let completion_free = pricing
                .completion
                .as_ref()
                .map(|c| c == "0" || c == "0.0" || c == "0.00")
                .unwrap_or(false);
            prompt_free && completion_free
        } else {
            false
        }
    }

    /// Get display string for the model
    pub fn display_string(&self) -> String {
        let free_tag = if self.is_free() { " [FREE]" } else { "" };
        let context = self
            .context_length
            .map(|c| format!(" ({}K)", c / 1000))
            .unwrap_or_default();
        format!("{}{}{}{}", self.name, free_tag, context, self.id)
    }
}

/// OpenRouter API response
#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

/// Fetch models from OpenRouter API
fn fetch_openrouter_models() -> Result<Vec<OpenRouterModel>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get("https://openrouter.ai/api/v1/models")
        .header("User-Agent", "claude-code-switcher")
        .send()
        .map_err(|e| anyhow!("Failed to fetch models from OpenRouter: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "OpenRouter API returned status: {}",
            response.status()
        ));
    }

    let models_response: OpenRouterModelsResponse = response
        .json()
        .map_err(|e| anyhow!("Failed to parse OpenRouter models response: {}", e))?;

    Ok(models_response.data)
}

/// Sort and filter models: free models first, then by creation time (newest first)
fn sort_models(mut models: Vec<OpenRouterModel>) -> Vec<OpenRouterModel> {
    models.sort_by(|a, b| {
        // Free models first
        let a_free = a.is_free();
        let b_free = b.is_free();

        if a_free && !b_free {
            return std::cmp::Ordering::Less;
        }
        if !a_free && b_free {
            return std::cmp::Ordering::Greater;
        }

        // Then by creation time (newest first)
        let a_created = a.created.unwrap_or(0);
        let b_created = b.created.unwrap_or(0);
        b_created.cmp(&a_created)
    });

    models
}

/// OpenRouter AI provider template
#[derive(Debug, Clone)]
pub struct OpenRouterTemplate {
    model_id: String,
}

impl OpenRouterTemplate {
    pub fn new(model_id: String) -> Self {
        Self { model_id }
    }

    /// Create template with interactive model selection
    pub fn create_with_model_selection() -> Result<Self> {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "OpenRouter requires interactive mode to select model. Use 'openrouter' with a specific model ID or run in interactive mode."
            ));
        }

        println!("🔄 Fetching available models from OpenRouter...");

        let models =
            fetch_openrouter_models().map_err(|e| anyhow!("Failed to fetch models: {}", e))?;

        if models.is_empty() {
            return Err(anyhow!("No models available from OpenRouter"));
        }

        let sorted_models = sort_models(models);

        let options: Vec<String> = sorted_models.iter().map(|m| m.display_string()).collect();

        let choice = Select::new("Select OpenRouter model:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get model selection: {}", e))?;

        // Find the selected model
        let selected_model = sorted_models
            .iter()
            .find(|m| m.display_string() == choice)
            .ok_or_else(|| anyhow!("Selected model not found"))?;

        Ok(Self::new(selected_model.id.clone()))
    }

    /// Create template with a specific model ID (non-interactive)
    pub fn with_model(model_id: &str) -> Self {
        Self::new(model_id.to_string())
    }
}

impl Template for OpenRouterTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::OpenRouter
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["OPENROUTER_API_KEY", "ANTHROPIC_AUTH_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "OpenRouter"
    }

    fn description(&self) -> &'static str {
        "OpenRouter - Access multiple AI models through a unified API"
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://openrouter.ai/keys")
    }

    fn api_host(&self) -> Option<&'static str> {
        Some("openrouter.ai")
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        // Return a single default variant - actual model selection happens interactively
        Ok(vec![Self::with_model("anthropic/claude-3.5-sonnet")])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        Self::create_with_model_selection()
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some(self.model_id.clone());

            settings.permissions = Some(Permissions {
                allow: Some(vec![
                    "Bash".to_string(),
                    "Read".to_string(),
                    "Write".to_string(),
                    "Edit".to_string(),
                    "MultiEdit".to_string(),
                    "Glob".to_string(),
                    "Grep".to_string(),
                    "WebFetch".to_string(),
                ]),
                ask: None,
                deny: Some(vec!["WebSearch".to_string()]),
                additional_directories: None,
                default_mode: None,
                disable_bypass_permissions_mode: None,
            });
        }

        if matches!(
            scope,
            SnapshotScope::Env | SnapshotScope::Common | SnapshotScope::All
        ) {
            let mut env = HashMap::new();
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://openrouter.ai/api".to_string(),
            );
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert("ANTHROPIC_API_KEY".to_string(), "".to_string());
            env.insert("ANTHROPIC_MODEL".to_string(), self.model_id.clone());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                self.model_id.clone(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create OpenRouter template settings (legacy compatibility function)
pub fn create_openrouter_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    // Default to a common model for legacy usage
    let template = OpenRouterTemplate::with_model("anthropic/claude-3.5-sonnet");
    template.create_settings(api_key, scope)
}
