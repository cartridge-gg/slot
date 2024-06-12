mod webauthn;

use std::{collections::HashMap, str::FromStr};

use account_sdk::{
    abigen::cartridge_account::Signer,
    signers::{
        webauthn::{DeviceSigner, WebauthnAccountSigner},
        SignerTrait,
    },
};
use anyhow::Result;
use katana_primitives::{
    contract::ContractAddress,
    genesis::json::{ClassNameOrHash, GenesisClassJson, GenesisContractJson, GenesisJson},
    FieldElement,
};
use lazy_static::lazy_static;
use serde_json::Value;
use starknet::core::utils::get_storage_var_address;

const WEBAUTHN_RP_ID: &str = "cartridge.gg";
const WEBAUTHN_ORIGIN: &str = "https://x.cartridge.gg";

lazy_static! {
    static ref CARTRIDGE_CONTROLLER_CLASS: Value = serde_json::from_str(include_str!(
        "../artifacts/cartridge_account_CartridgeAccount.contract_class.json"
    ))
    .unwrap();
}

fn add_controller_class(genesis: &mut GenesisJson) -> Result<()> {
    let class = GenesisClassJson {
        class_hash: None,
        name: Some("controller".to_string()),
        class: CARTRIDGE_CONTROLLER_CLASS.clone().into(),
    };

    genesis.classes.push(class);

    Ok(())
}

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
        let address = FieldElement::from_str(&address)?;

        const NON_STARK_OWNER_VAR_NAME: &str = "_owner_non_stark";
        let storage = get_storage_var_address(NON_STARK_OWNER_VAR_NAME, &[r#type]).unwrap();
        let storages = HashMap::from([(storage, guid)]);

        let account = GenesisContractJson {
            nonce: None,
            balance: None,
            storage: Some(storages),
            class: Some(ClassNameOrHash::Name("controller".to_string())),
        };

        (ContractAddress::from(address), account)
    };

    genesis.contracts.insert(address, contract);

    Ok(())
}
