use std::collections::HashMap;

use alloy_primitives::{hex, U256};
use katana_primitives::{
    contract::ContractAddress,
    genesis::{
        json::{
            ClassNameOrHash, GenesisAccountJson, GenesisClassJson, GenesisJson, PathOrFullArtifact,
        },
        Genesis,
    },
    FieldElement,
};
use lazy_static::lazy_static;
use serde_json::Value;
use sha2::{digest::Update, Digest, Sha256};
use starknet::{
    core::utils::get_storage_var_address,
    macros::{felt, short_string},
};
use starknet_crypto::poseidon_hash_many;

const WEBAUTHN_RP_ID: &str = "cartridge.gg";
const WEBAUTHN_ORIGIN: &str = "https://x.cartridge.gg";

lazy_static! {
    static ref CARTRIDGE_CONTROLLER_CLASS: Value = serde_json::from_str(include_str!(
        "../build/cartridge_account_CartridgeAccount.contract_class.json"
    ))
    .unwrap();
}

pub struct WebAuthnSigner<'a, 'b> {
    origin: &'a str,
    rp_id: &'b str,
    pubkey: U256,
    guid: Option<FieldElement>,
}

impl<'a, 'b> WebAuthnSigner<'a, 'b> {
    pub const fn new_cartidge(pubkey: U256) -> Self {
        Self {
            pubkey,
            guid: None,
            rp_id: WEBAUTHN_RP_ID,
            origin: WEBAUTHN_ORIGIN,
        }
    }

    pub const fn new(origin: &'a str, rp_id: &'b str, pubkey: U256) -> Self {
        Self {
            origin,
            rp_id,
            pubkey,
            guid: None,
        }
    }

    pub fn storage_value(&self) -> (FieldElement, FieldElement) {
        let signer_type = FieldElement::from(4u8);
        let guid = self.guid();
        (signer_type, guid)
    }

    pub fn guid(&self) -> FieldElement {
        self.guid.unwrap_or_else(|| self.compute_guid())
    }

    fn compute_guid(&self) -> FieldElement {
        let mut buffer: Vec<FieldElement> = vec![short_string!("Webauthn Signer")];

        buffer.push(self.origin.len().into());
        for b in self.origin.as_bytes() {
            buffer.push((*b).into());
        }

        let hash: [u8; 32] = Sha256::new().chain(self.rp_id).finalize().into();
        let rp_id_hash = U256::from_be_bytes(hash);

        let rp_id_hash_low: u128 = (rp_id_hash & U256::from(u128::MAX)).to();
        let rp_id_hash_high: u128 = U256::from(rp_id_hash >> 128).to();

        buffer.push(rp_id_hash_low.into());
        buffer.push(rp_id_hash_high.into());

        let pub_key_low: u128 = (self.pubkey & U256::from(u128::MAX)).to();
        let pub_key_high: u128 = U256::from(self.pubkey >> 128).to();

        buffer.push(pub_key_low.into());
        buffer.push(pub_key_high.into());

        poseidon_hash_many(&buffer)
    }
}

// build the genesis json file
#[test]
fn build() {
    let controller = GenesisClassJson {
        class_hash: None,
        name: Some("controller".to_string()),
        class: PathOrFullArtifact::Artifact(CARTRIDGE_CONTROLLER_CLASS.clone()),
    };

    // TODO: get public key from user
    let pubkey = hex!("e03a1caadf5cdfe8d05b8cd283bfd8c8b7da904235bc79ae967e6a0215158067");
    let signer = WebAuthnSigner::new_cartidge(U256::from_be_bytes(pubkey));
    let (r#type, guid) = signer.storage_value();

    let account = {
        // TODO: get address from user
        let address = ContractAddress::from(felt!("0x1"));

        const NON_STARK_OWNER_VAR_NAME: &str = "_owner_non_stark";
        let storage = get_storage_var_address(NON_STARK_OWNER_VAR_NAME, &[r#type]).unwrap();
        let storages = HashMap::from([(storage, guid)]);

        let account = GenesisAccountJson {
            nonce: None,
            balance: None,
            private_key: None,
            storage: Some(storages),
            public_key: FieldElement::default(),
            // should correspond to the controller class' name above
            class: Some(ClassNameOrHash::Name("controller".to_string())),
        };

        (address, account)
    };

    let json = GenesisJson {
        classes: vec![controller],
        accounts: HashMap::from([account]),
        ..Default::default()
    };

    let _ = Genesis::try_from(json).unwrap();
}

#[test]
fn test_generate_storage_values() {
    let pubkey = hex!("e03a1caadf5cdfe8d05b8cd283bfd8c8b7da904235bc79ae967e6a0215158067");
    let signer = WebAuthnSigner::new_cartidge(U256::from_be_bytes(pubkey));

    let (r#type, guid) = signer.storage_value();

    let expected_type = felt!("0x4");
    let expected_guid = felt!("0x33b501c1720abbf891580658cc5308faddc3d85705267dce1f6d9922d0d6a3d");

    assert_eq!(expected_type, r#type);
    assert_eq!(expected_guid, guid);
}
