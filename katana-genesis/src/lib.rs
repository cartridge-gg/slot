use std::collections::HashMap;

use katana_primitives::{
    contract::ContractAddress,
    genesis::json::{
        ClassNameOrHash, GenesisAccountJson, GenesisClassJson, GenesisJson, PathOrFullArtifact,
    },
};
use lazy_static::lazy_static;
use serde_json::Value;
use starknet::macros::felt;

const WEBAUTHN_RP_ID: &str = "cartridge.gg";
const WEBAUTHN_ORIGIN: &str = "https://x.cartridge.gg";

lazy_static! {
    static ref CARTRIDGE_CONTROLLER_CLASS: Value = serde_json::from_str(include_str!(
        "../build/cartridge_account_CartridgeAccount.contract_class.json"
    ))
    .unwrap();
}

// build the genesis json file
fn build() -> GenesisJson {
    let controller = GenesisClassJson {
        class_hash: None, // TODO: should we use a specific hash?
        name: Some("controller".to_string()),
        class: PathOrFullArtifact::Artifact(CARTRIDGE_CONTROLLER_CLASS.clone()),
    };

    let account = {
        let address = ContractAddress::from(felt!("0x1"));
        let account = GenesisAccountJson {
            nonce: None,
            balance: None,
            storage: None,
            private_key: None,
            public_key: felt!("0x1"),
            // should correspond to the controller class' name above
            class: Some(ClassNameOrHash::Name("controller".to_string())),
        };

        (address, account)
    };

    let genesis = GenesisJson {
        classes: vec![controller],
        accounts: HashMap::from([account]),
        ..Default::default()
    };

    genesis
}
