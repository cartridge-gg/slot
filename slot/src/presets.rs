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
    pub origin: Vec<String>,
    pub theme: Option<serde_json::Value>,
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

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to fetch preset {}: HTTP Status {}",
            preset_name,
            response.status()
        ));
    }

    let config = response.json::<ControllerConfig>().await.context(format!(
        "Failed to parse preset configuration: {}",
        preset_name
    ))?;

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
                        selector: String::new(), // Leave empty as it will be calculated by the backend
                    });
                }
            }
        }
    }

    policies
}

#[derive(Debug, Clone)]
pub struct PaymasterPolicyInput {
    pub contract_address: String,
    pub entry_point: String,
    pub selector: String,
}
