//! Template selector for choosing AI provider templates

use crate::selectors::error::{SelectorError, SelectorResult};
use crate::{
    credentials::CredentialStore,
    templates::{TemplateType, get_template_instance},
};

/// Template selector for choosing AI provider templates
pub struct TemplateSelector;

impl TemplateSelector {
    /// Select a template type from available options
    pub fn select_template() -> SelectorResult<TemplateType> {
        let template_types = [
            TemplateType::DeepSeek,
            TemplateType::Zai,
            TemplateType::KatCoder,
            TemplateType::Kimi,
            TemplateType::Longcat,
            TemplateType::Fishtrip,
            TemplateType::MiniMax,
            TemplateType::SeedCode,
            TemplateType::Zenmux,
            TemplateType::Duojie,
            TemplateType::AnyRouter,
        ];

        let options: Vec<String> = template_types.iter().map(|t| t.to_string()).collect();

        let selection = inquire::Select::new("Select AI provider:", options)
            .with_help_message("↑/↓: Navigate, Enter: Select, Esc: Cancel")
            .prompt()
            .map_err(inquire_to_selector_error)?;

        template_types
            .into_iter()
            .find(|t| t.to_string() == selection)
            .ok_or(SelectorError::NotFound)
    }

    /// Get API key for a template type
    pub fn get_api_key_for_template(template_type: TemplateType) -> SelectorResult<Option<String>> {
        crate::selectors::credential::CredentialSelector::select_api_key(template_type)
    }

    /// Get endpoint ID for templates that require it
    pub fn get_endpoint_id_for_template(
        template_type: &TemplateType,
    ) -> SelectorResult<Option<String>> {
        let template_instance = get_template_instance(template_type);

        if !template_instance.requires_additional_config() {
            return Ok(None);
        }

        let endpoint_ids = CredentialStore::new()
            .map(|store| store.get_endpoint_ids(template_type))
            .unwrap_or_default();

        if endpoint_ids.is_empty() {
            return Self::prompt_endpoint_id(template_type);
        }

        let mut options = vec!["Enter new endpoint ID...".to_string()];
        for (display_name, _) in &endpoint_ids {
            options.push(display_name.clone());
        }

        let selection = inquire::Select::new(
            &format!("Select {} endpoint ID:", template_type),
            options,
        )
        .with_help_message("↑/↓: Navigate, Enter: Select, Esc: Cancel")
        .prompt()
        .map_err(inquire_to_selector_error)?;

        if selection == "Enter new endpoint ID..." {
            Self::prompt_endpoint_id(template_type)
        } else {
            endpoint_ids
                .into_iter()
                .find(|(name, _)| name == &selection)
                .map(|(_, id)| Some(id))
                .ok_or(SelectorError::NotFound)
        }
    }

    fn prompt_endpoint_id(template_type: &TemplateType) -> SelectorResult<Option<String>> {
        let template_instance = get_template_instance(template_type);

        if let Some(url) = template_instance.api_key_url() {
            println!("  Get your endpoint ID from: {}", url);
        }

        let endpoint_id = inquire::Text::new(&format!("Enter {} endpoint ID:", template_type))
            .with_placeholder("ep-xxx-xxx")
            .prompt()
            .map_err(inquire_to_selector_error)?;

        if endpoint_id.trim().is_empty() {
            Err(SelectorError::InvalidInput(
                "Endpoint ID cannot be empty".to_string(),
            ))
        } else {
            Ok(Some(endpoint_id))
        }
    }
}

/// Convert inquire errors to SelectorError
fn inquire_to_selector_error(e: inquire::InquireError) -> SelectorError {
    let msg = e.to_string();
    if msg.contains("canceled") || msg.contains("cancelled") {
        SelectorError::Cancelled
    } else {
        SelectorError::Failed(msg)
    }
}

/// Legacy compatibility function
pub fn get_endpoint_id_interactively(template_type: &TemplateType) -> SelectorResult<String> {
    match TemplateSelector::get_endpoint_id_for_template(template_type)? {
        Some(endpoint_id) => Ok(endpoint_id),
        None => Err(SelectorError::NotFound),
    }
}
