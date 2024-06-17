#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::{collections::HashMap, str::FromStr};

use account_sdk::abigen::cartridge_account::Signer;
use account_sdk::signers::webauthn::{DeviceSigner, WebauthnAccountSigner};
use account_sdk::signers::SignerTrait;
use anyhow::{Context, Result};
use katana_primitives::contract::ContractAddress;
use katana_primitives::genesis::json::{ClassNameOrHash, GenesisClassJson};
use katana_primitives::genesis::json::{GenesisContractJson, GenesisJson};
use katana_primitives::FieldElement;
use serde_json::Value;
use starknet::core::utils::get_storage_var_address;

mod webauthn;

const CONTROLLER_CLASS_NAME: &str = "controller";

const WEBAUTHN_RP_ID: &str = "cartridge.gg";
const WEBAUTHN_ORIGIN: &str = "https://x.cartridge.gg";

// TODO(kariy): should accept the whole account struct instead of individual fields
// build the genesis json file
pub fn add_controller_account(
    genesis: &mut GenesisJson,
    address: &str,
    credential_id: &str,
    pub_key: &str,
) -> Result<()> {
    add_controller_class(genesis)?;

    let credential_id = webauthn::credential::from_base64(credential_id)?;
    let pub_key = webauthn::cose_key::from_base64(pub_key)?;
    let signer = DeviceSigner::new(
        WEBAUTHN_RP_ID.to_string(),
        WEBAUTHN_ORIGIN.to_string(),
        credential_id,
        pub_key,
    );

    let signer = Signer::Webauthn(signer.signer_pub_data());
    // webauthn signer type as seen in the cairo contract <https://github.com/cartridge-gg/controller-internal/blob/394b60b1df92d7b173b3215051d67b85c342dbea/crates/webauthn/auth/src/signer.cairo#L181>
    let r#type = FieldElement::from(4u8);
    let guid = signer.guid();

    let (address, contract) = {
        let address = FieldElement::from_str(address)?;

        // the storage variable name for webauthn signer
        const NON_STARK_OWNER_VAR_NAME: &str = "_owner_non_stark";
        let storage = get_storage_var_address(NON_STARK_OWNER_VAR_NAME, &[r#type]).unwrap();
        let storages = HashMap::from([(storage, guid)]);

        let account = GenesisContractJson {
            nonce: None,
            balance: None,
            storage: Some(storages),
            class: Some(ClassNameOrHash::Name(CONTROLLER_CLASS_NAME.to_string())),
        };

        (ContractAddress::from(address), account)
    };

    genesis.contracts.insert(address, contract);

    Ok(())
}

fn add_controller_class(genesis: &mut GenesisJson) -> Result<()> {
    // parse the controller class json file
    let json = include_str!("../build/controller_CartridgeAccount.contract_class.json");
    let json = serde_json::from_str::<Value>(json).context("Failed to parse class artifact")?;

    let class = GenesisClassJson {
        class_hash: None,
        class: json.into(),
        name: Some(CONTROLLER_CLASS_NAME.to_string()),
    };

    genesis.classes.push(class);

    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_add_controller_account() {}
}
