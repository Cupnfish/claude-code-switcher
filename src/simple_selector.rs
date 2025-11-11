use crate::{
    credentials::{CredentialStore, SavedCredential},
    templates::{TemplateType, get_template_instance},
};
use anyhow::{Result, anyhow};
use inquire::{Select, Text};

/// Simple credential selector using inquire for consistency with CLI experience
pub struct SimpleCredentialSelector {
    credentials: Vec<SavedCredential>,
    template_type: TemplateType,
}

impl SimpleCredentialSelector {
    pub fn new(credentials: Vec<SavedCredential>, template_type: TemplateType) -> Self {
        Self {
            credentials,
            template_type,
        }
    }

    pub fn run(&mut self) -> Result<Option<String>> {
        if self.credentials.is_empty() {
            // No saved credentials, show API key acquisition URL if available
            let template_instance = get_template_instance(&self.template_type);
            if let Some(url) = template_instance.api_key_url() {
                println!("  💡 Get your API key from: {}", url);
            }

            // Prompt for new API key directly
            let prompt_text = format!("Enter your {} API key:", self.template_type);
            let api_key = Text::new(&prompt_text)
                .with_placeholder("sk-...")
                .prompt()
                .map_err(|e| anyhow!("Failed to get API key: {}", e))?;

            if !api_key.trim().is_empty() {
                return Ok(Some(api_key));
            }
            return Ok(None);
        }

        // Create selection options
        let mut options = Vec::new();

        // Add saved credentials
        for credential in &self.credentials {
            let masked_key = mask_api_key(credential.api_key());
            options.push(format!("{} ({})", credential.name(), masked_key));
        }

        // Add "Enter new API key" option
        options.push("Enter new API key...".to_string());

        // Show selection prompt
        let prompt_text = format!("Select {} API key:", self.template_type);
        match Select::new(&prompt_text, options).prompt() {
            Ok(choice) => {
                if choice == "Enter new API key..." {
                    // Show API key acquisition URL if available
                    let template_instance = get_template_instance(&self.template_type);
                    if let Some(url) = template_instance.api_key_url() {
                        println!("  💡 Get your API key from: {}", url);
                    }

                    // Prompt for new API key
                    let prompt_text = format!("Enter your {} API key:", self.template_type);
                    let api_key = Text::new(&prompt_text)
                        .with_placeholder("sk-...")
                        .prompt()
                        .map_err(|e| anyhow!("Failed to get API key: {}", e))?;

                    if !api_key.trim().is_empty() {
                        return Ok(Some(api_key));
                    }
                    Err(anyhow!("API key cannot be empty"))
                } else {
                    // Extract API key from selected credential
                    let index = self.find_credential_index(&choice)?;
                    Ok(Some(self.credentials[index].api_key().to_string()))
                }
            }
            Err(_) => Ok(None), // User cancelled
        }
    }

    fn find_credential_index(&self, choice: &str) -> Result<usize> {
        for (i, credential) in self.credentials.iter().enumerate() {
            let masked_key = mask_api_key(credential.api_key());
            let option_text = format!("{} ({})", credential.name(), masked_key);
            if option_text == choice {
                return Ok(i);
            }
        }
        Err(anyhow!("Credential not found"))
    }
}

/// Simple endpoint ID selector using inquire
pub fn get_endpoint_id_interactively(template_type: &TemplateType) -> Result<String> {
    // Get saved endpoint IDs
    let endpoint_ids = if let Ok(credential_store) = CredentialStore::new() {
        credential_store.get_endpoint_ids(template_type)
    } else {
        Vec::new()
    };

    if endpoint_ids.is_empty() {
        // No saved endpoint IDs, show API key acquisition URL if available
        let template_instance = get_template_instance(template_type);
        if let Some(url) = template_instance.api_key_url() {
            println!("  💡 Get your API key from: {}", url);
        }

        // Prompt for new one directly
        let prompt_text = format!("Enter {} endpoint ID:", template_type);
        let endpoint_id = Text::new(&prompt_text)
            .with_placeholder("ep-xxx-xxx")
            .prompt()
            .map_err(|e| anyhow!("Failed to get endpoint ID: {}", e))?;

        if !endpoint_id.trim().is_empty() {
            return Ok(endpoint_id);
        }
        return Err(anyhow!("Endpoint ID cannot be empty"));
    }

    // Create selection options
    let mut options = Vec::new();

    // Add saved endpoint IDs
    for (display_name, _endpoint_id) in &endpoint_ids {
        options.push(display_name.clone());
    }

    // Add "Enter new endpoint ID" option
    options.push("Enter new endpoint ID...".to_string());

    // Show selection prompt
    let prompt_text = format!("Select {} endpoint ID:", template_type);
    match Select::new(&prompt_text, options).prompt() {
        Ok(choice) => {
            if choice == "Enter new endpoint ID..." {
                // Show API key acquisition URL if available
                let template_instance = get_template_instance(template_type);
                if let Some(url) = template_instance.api_key_url() {
                    println!("  💡 Get your API key from: {}", url);
                }

                // Prompt for new endpoint ID
                let prompt_text = format!("Enter {} endpoint ID:", template_type);
                let endpoint_id = Text::new(&prompt_text)
                    .with_placeholder("ep-xxx-xxx")
                    .prompt()
                    .map_err(|e| anyhow!("Failed to get endpoint ID: {}", e))?;

                if !endpoint_id.trim().is_empty() {
                    return Ok(endpoint_id);
                }
                Err(anyhow!("Endpoint ID cannot be empty"))
            } else {
                // Find and return the selected endpoint ID
                for (display_name, endpoint_id) in &endpoint_ids {
                    if display_name == &choice {
                        return Ok(endpoint_id.clone());
                    }
                }
                Err(anyhow!("Endpoint ID not found"))
            }
        }
        Err(_) => Err(anyhow!("No endpoint ID selected")), // User cancelled
    }
}

fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        "••••••••".to_string()
    } else {
        format!(
            "{}{}{}",
            &api_key[..4],
            "•".repeat(api_key.len() - 8),
            &api_key[api_key.len() - 4..]
        )
    }
}
