//! Template selector using the unified selector framework

use crate::selectors::{
    base::SelectableItem,
    error::{SelectorError, SelectorResult},
    navigation::NavigationManager,
};
use crate::{
    credentials::CredentialStore,
    templates::{TemplateType, get_template_instance},
};

/// Template selector for choosing AI provider templates
pub struct TemplateSelector;

impl TemplateSelector {
    /// Select a template type
    pub fn select_template() -> SelectorResult<TemplateType> {
        let template_types = vec![
            TemplateType::DeepSeek,
            TemplateType::Zai,
            TemplateType::KatCoder,
            TemplateType::Kimi,
            TemplateType::Longcat,
            TemplateType::MiniMax,
            TemplateType::SeedCode,
            TemplateType::Zenmux,
        ];

        let items: Vec<TemplateItem> = template_types.into_iter().map(TemplateItem::new).collect();

        match NavigationManager::select_from_list(
            &items,
            "Select AI provider:",
            false,
            Some("↑/↓: Navigate, →: Select, ←/Esc: Back"),
        )? {
            crate::selectors::navigation::NavigationResult::Selected(item) => {
                Ok(item.template_type)
            }
            crate::selectors::navigation::NavigationResult::Back
            | crate::selectors::navigation::NavigationResult::Exit
            | crate::selectors::navigation::NavigationResult::CreateNew => {
                Err(SelectorError::Cancelled)
            }
        }
    }

    /// Get API key for a template type
    pub fn get_api_key_for_template(template_type: TemplateType) -> SelectorResult<Option<String>> {
        crate::selectors::credential::CredentialSelector::select_api_key(template_type)
    }

    /// Get endpoint ID for template types that require it
    pub fn get_endpoint_id_for_template(
        template_type: &TemplateType,
    ) -> SelectorResult<Option<String>> {
        let template_instance = get_template_instance(template_type);

        // Only some templates require additional config
        if !template_instance.requires_additional_config() {
            return Ok(None);
        }

        // Get saved endpoint IDs
        let endpoint_ids = if let Ok(credential_store) = CredentialStore::new() {
            credential_store.get_endpoint_ids(template_type)
        } else {
            Vec::new()
        };

        if endpoint_ids.is_empty() {
            // No saved endpoint IDs, prompt for new one
            if let Some(url) = template_instance.api_key_url() {
                println!("  💡 Get your endpoint ID from: {}", url);
            }

            let prompt_text = format!("Enter {} endpoint ID:", template_type);
            let endpoint_id =
                NavigationManager::get_text_input(&prompt_text, Some("ep-xxx-xxx"), None)?;

            if !endpoint_id.trim().is_empty() {
                return Ok(Some(endpoint_id));
            }
            return Err(SelectorError::InvalidInput(
                "Endpoint ID cannot be empty".to_string(),
            ));
        }

        // Create selection options
        let items: Vec<EndpointItem> = endpoint_ids
            .into_iter()
            .map(|(display_name, endpoint_id)| EndpointItem {
                display_name,
                endpoint_id,
            })
            .collect();

        let mut options = vec!["Enter new endpoint ID...".to_string()];

        for item in &items {
            options.push(item.display_name.clone());
        }

        let selection = NavigationManager::select_option(
            &format!("Select {} endpoint ID:", template_type),
            &options.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            Some("↑/↓: Navigate, →: Select, ←/Esc: Back"),
        )?;

        if selection == "Enter new endpoint ID..." {
            if let Some(url) = template_instance.api_key_url() {
                println!("  💡 Get your endpoint ID from: {}", url);
            }

            let prompt_text = format!("Enter {} endpoint ID:", template_type);
            let endpoint_id =
                NavigationManager::get_text_input(&prompt_text, Some("ep-xxx-xxx"), None)?;

            if !endpoint_id.trim().is_empty() {
                Ok(Some(endpoint_id))
            } else {
                Err(SelectorError::InvalidInput(
                    "Endpoint ID cannot be empty".to_string(),
                ))
            }
        } else {
            // Find and return the selected endpoint ID
            for item in items {
                if item.display_name == selection {
                    return Ok(Some(item.endpoint_id));
                }
            }
            Err(SelectorError::NotFound)
        }
    }
}

/// Template item for selection
#[derive(Debug, Clone)]
struct TemplateItem {
    template_type: TemplateType,
}

impl TemplateItem {
    fn new(template_type: TemplateType) -> Self {
        Self { template_type }
    }
}

impl SelectableItem for TemplateItem {
    fn display_name(&self) -> String {
        format!("{}", self.template_type)
    }

    fn format_for_list(&self) -> String {
        let template_instance = get_template_instance(&self.template_type);
        let env_vars = template_instance.env_var_names();
        let env_indicator = if env_vars.len() > 1 {
            format!(" (+{})", env_vars.len())
        } else {
            String::new()
        };

        format!("{} ({:?}){}", self.template_type, env_vars, env_indicator)
    }

    fn id(&self) -> Option<String> {
        Some(format!("{:?}", self.template_type))
    }
}

/// Endpoint ID item for selection
#[derive(Debug, Clone)]
struct EndpointItem {
    display_name: String,
    endpoint_id: String,
}

impl SelectableItem for EndpointItem {
    fn display_name(&self) -> String {
        self.display_name.clone()
    }

    fn format_for_list(&self) -> String {
        self.display_name.clone()
    }

    fn id(&self) -> Option<String> {
        Some(self.endpoint_id.clone())
    }
}

/// Simple endpoint ID selector for legacy compatibility
pub fn get_endpoint_id_interactively(template_type: &TemplateType) -> SelectorResult<String> {
    match TemplateSelector::get_endpoint_id_for_template(template_type)? {
        Some(endpoint_id) => Ok(endpoint_id),
        None => Err(SelectorError::NotFound),
    }
}
