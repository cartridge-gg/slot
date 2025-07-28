use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const CONFIG_BASE_URL: &str = "https://static.cartridge.gg/presets";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Method {
    pub name: Option<String>,
    pub description: Option<String>,
    pub entrypoint: String,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub is_paymastered: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractPolicy {
    pub name: Option<String>,
    pub description: Option<String>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SessionPolicies {
    pub contracts: HashMap<String, ContractPolicy>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChainPolicies {
    pub policies: SessionPolicies,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ControllerConfig {
    #[serde(default)]
    pub chains: HashMap<String, ChainPolicies>,
}

pub async fn load_preset(preset_name: &str) -> Result<ControllerConfig> {
    let client = Client::new();
    let url = format!("{}/{}/config.json", CONFIG_BASE_URL, preset_name);

    let response = client
        .get(&url)
        .send()
        .await
        .context(format!("Failed to fetch preset: {}", preset_name))?;

    if response.status() == 404 {
        return Err(anyhow::anyhow!(
            "Preset '{}' not found. Check available presets at https://github.com/cartridge-gg/presets/tree/main/configs",
            preset_name
        ));
    }

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch preset {}: HTTP Status {}",
            preset_name,
            response.status()
        ));
    }

    let response_text = response.text().await.context(format!(
        "Failed to get response body as text for preset: {}",
        preset_name
    ))?;

    // Try to parse the text into the expected structure
    let config = serde_json::from_str::<ControllerConfig>(&response_text).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse preset configuration: {}\nError details: {}\nResponse body: {}",
            preset_name,
            e,
            response_text
        )
    })?;

    Ok(config)
}

pub fn extract_paymaster_policies(
    config: &ControllerConfig,
    chain_id: &str,
) -> Vec<PaymasterPolicyInput> {
    let mut policies = Vec::new();

    if let Some(chain_policies) = config.chains.get(chain_id) {
        for (contract_address, contract_policy) in &chain_policies.policies.contracts {
            for method in &contract_policy.methods {
                if method.is_paymastered {
                    policies.push(PaymasterPolicyInput {
                        contract_address: contract_address.clone(),
                        entry_point: method.entrypoint.clone(),
                    });
                }
            }
        }
    }

    policies
}

#[derive(Deserialize, Debug, Clone)]
pub struct PaymasterPolicyInput {
    #[serde(rename = "contractAddress")]
    pub contract_address: String,

    #[serde(rename = "entryPoint")]
    pub entry_point: String,
}
