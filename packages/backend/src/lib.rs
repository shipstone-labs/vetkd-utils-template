use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk_macros::*;
use ic_stable_structures::{storable::Bound, Storable};
use std::{borrow::Cow, collections::HashMap};

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq)]
pub struct PrincipalEntry {
    name: Option<String>,
    when: Option<u64>,
    was_read: bool,
    last_read: Option<u64>,
}

pub type PrincipalName = String;
pub type NoteId = u128;

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq)]
pub struct EncryptedNote {
    id: NoteId,
    encrypted_text: String,
    data: String,
    owner: PrincipalName,
    locked: bool,
    /// Principals with whom this note is shared. Does not include the owner.
    /// Needed to be able to efficiently show in the UI with whom this note is shared.
    users: Vec<PrincipalEntry>,
    read_by_first: HashMap<PrincipalName, u64>,
    read_by_last: HashMap<PrincipalName, u64>,
}

impl EncryptedNote {
    // Check if the user is owner or has access to the note as of right now
    pub fn is_authorized(&self, user: &PrincipalName) -> bool {
        user == &self.owner
            || self.users.iter().any(|u| {
                let name = u.name.clone();
                (name.is_none() || &name.unwrap() == user)
                    && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
            })
    }
    // Same as above but mark it as being read by that user
    pub fn lock_authorized(&mut self) -> bool {
        let user = &caller().to_text();
        if user == &self.owner {
            return true;
        }
        if self.is_authorized(user) {
            // once a non-owner reads a note it's locked and can no longer
            // be updated
            if !self.users.iter_mut().any(|u| {
                if u.name.as_ref() == Some(user)
                    && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
                {
                    u.was_read = true;
                    u.last_read = Some(ic_cdk::api::time());
                    self.locked = true;
                    self.read_by_last
                        .insert(user.to_string(), ic_cdk::api::time());
                    if !self.read_by_first.contains_key(user) {
                        self.read_by_first
                            .insert(user.to_string(), ic_cdk::api::time());
                    }
                    true
                } else {
                    false
                }
            }) && !self.users.iter_mut().any(|u| {
                if u.name.is_none() && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
                {
                    u.was_read = true;
                    u.last_read = Some(ic_cdk::api::time());
                    self.locked = true;
                    self.read_by_last
                        .insert(user.to_string(), ic_cdk::api::time());
                    if !self.read_by_first.contains_key(user) {
                        self.read_by_first
                            .insert(user.to_string(), ic_cdk::api::time());
                    }
                    true
                } else {
                    false
                }
            }) {
                return false;
            }
            true
        } else {
            false
        }
    }
    // add a new reader to the note
    pub fn add_reader(&mut self, user: PrincipalEntry) -> bool {
        if self.locked && self.users.iter().any(|u| u.name == user.name && u.was_read) {
            return false;
        }
        self.users.retain(|u| u.name.as_ref() != user.name.as_ref());
        self.users.push(user);
        true
    }
    // Was the note ever read by that user
    pub fn user_read_when(&self, user: &PrincipalName) -> Option<u64> {
        self.read_by_first.get(user).cloned()
    }
    // Remove a reader (will return false if the note was already read by the user)
    pub fn remove_reader(&mut self, user: &PrincipalName) -> bool {
        if self.locked
            && self
                .users
                .iter()
                .any(|u| u.name.iter().any(|u| u == user) && u.was_read)
        {
            return false;
        }
        self.users.retain(|u| u.name.as_ref() != Some(user));
        true
    }
    // Update the data. This is only allowed by the owner before the note was locked
    pub fn set_data(&mut self, data: String) -> bool {
        if self.locked && caller().to_text() != self.owner {
            return false;
        }
        self.data = data;
        true
    }
    // Is the note shared at all?
    pub fn is_shared(&self) -> bool {
        !self.users.is_empty()
    }
    // Has any reader read it?
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

impl Storable for EncryptedNote {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Default)]
pub struct NoteIds {
    ids: Vec<NoteId>,
}

impl NoteIds {
    pub fn iter(&self) -> impl std::iter::Iterator<Item = &NoteId> {
        self.ids.iter()
    }
}

impl Storable for NoteIds {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
    const BOUND: Bound = Bound::Unbounded;
}

/// Unlike Motoko, the caller identity is not built into Rust.
/// Thus, we use the ic_cdk::caller() method inside this wrapper function.
/// The wrapper prevents the use of the anonymous identity. Forbidding anonymous
/// interactions is the recommended default behavior for IC canisters.
fn caller() -> Principal {
    let caller = ic_cdk::caller();
    // The anonymous principal is not allowed to interact with the
    // encrypted notes canister.
    if caller == Principal::anonymous() {
        panic!("Anonymous principal not allowed to make calls.")
    }
    caller
}

mod vetkd_types;

const VETKD_SYSTEM_API_CANISTER_ID: &str = "s55qq-oqaaa-aaaaa-aaakq-cai";

use vetkd_types::{
    CanisterId, VetKDCurve, VetKDEncryptedKeyReply, VetKDEncryptedKeyRequest, VetKDKeyId,
    VetKDPublicKeyReply, VetKDPublicKeyRequest,
};

#[update]
async fn symmetric_key_verification_key_for_note() -> String {
    let request = VetKDPublicKeyRequest {
        canister_id: None,
        derivation_path: vec![b"note_symmetric_key".to_vec()],
        key_id: bls12_381_test_key_1(),
    };

    let (response,): (VetKDPublicKeyReply,) = ic_cdk::call(
        vetkd_system_api_canister_id(),
        "vetkd_public_key",
        (request,),
    )
    .await
    .expect("call to vetkd_public_key failed");

    hex::encode(response.public_key)
}

pub async fn encrypted_symmetric_key_for_note(
    note_id: NoteId,
    owner: &PrincipalName,
    encryption_public_key: Vec<u8>,
) -> String {
    let request = VetKDEncryptedKeyRequest {
        derivation_id: {
            let mut buf = vec![];
            buf.extend_from_slice(&note_id.to_be_bytes()); // fixed-size encoding
            buf.extend_from_slice(owner.as_bytes());
            buf // prefix-free
        },
        public_key_derivation_path: vec![b"note_symmetric_key".to_vec()],
        key_id: bls12_381_test_key_1(),
        encryption_public_key,
    };

    let (response,): (VetKDEncryptedKeyReply,) = ic_cdk::call(
        vetkd_system_api_canister_id(),
        "vetkd_encrypted_key",
        (request,),
    )
    .await
    .expect("call to vetkd_encrypted_key failed");

    hex::encode(response.encrypted_key)
}

fn bls12_381_test_key_1() -> VetKDKeyId {
    VetKDKeyId {
        curve: VetKDCurve::Bls12_381,
        name: "test_key_1".to_string(),
    }
}

fn vetkd_system_api_canister_id() -> CanisterId {
    use std::str::FromStr;
    CanisterId::from_str(VETKD_SYSTEM_API_CANISTER_ID).expect("failed to create canister ID")
}
