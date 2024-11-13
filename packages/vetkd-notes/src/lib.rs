use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk_macros::*;
use ic_stable_structures::{storable::Bound, Storable};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq, Hash)]
pub struct PrincipalRule {
    when: Option<u64>,
    was_read: bool,
}

pub type PrincipalName = String;
pub type NoteId = u128;

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq, Hash)]
pub struct HistoryEntry {
    action: String,
    user: Option<String>,
    rule: Option<(Option<String>, Option<u64>)>,
    created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq)]
pub struct EncryptedNote {
    id: NoteId,
    encrypted_text: String,
    data: String,
    owner: PrincipalName,
    /// Principals with whom this note is shared. Does not include the owner.
    /// Needed to be able to efficiently show in the UI with whom this note is shared.
    users: HashMap<Option<String>, PrincipalRule>,

    locked: bool,
    read_by: HashSet<PrincipalName>,
    created_at: u64,
    updated_at: u64,
    history: Vec<HistoryEntry>,
}

impl EncryptedNote {
    // Check if the user is owner or has access to the note as of right now
    pub fn is_authorized(&self, user: &PrincipalName) -> bool {
        if user == &self.owner {
            return true;
        }
        // once a non-owner reads a note it's locked and can no longer
        // be updated
        let check = &Some(user.to_string());
        if let Some(r) = self.users.get(check) {
            if r.when.is_none() || r.when.unwrap() <= ic_cdk::api::time() {
                return true;
            }
        } else if let Some(r) = self.users.get(&None) {
            if r.when.is_none() || r.when.unwrap() <= ic_cdk::api::time() {
                return true;
            }
        }
        false
    }
    // Same as above but mark it as being read by that user
    pub fn lock_authorized(&mut self) -> bool {
        let user = &caller().to_text();
        if user == &self.owner {
            return true;
        }
        // once a non-owner reads a note it's locked and can no longer
        // be updated
        let check = &Some(user.to_string());
        if let Some(r) = self.users.get_mut(check) {
            if r.when.is_none() || r.when.unwrap() <= ic_cdk::api::time() {
                r.was_read = true;
                self.history.append(&mut vec![HistoryEntry {
                    action: if self.locked {
                        "read".to_string()
                    } else {
                        "read-locked".to_string()
                    },
                    user: Some(user.to_string()),
                    rule: Some((check.clone(), r.when)),
                    created_at: ic_cdk::api::time(),
                }]);
                self.locked = true;
                self.read_by.insert(user.to_string());
                return true;
            }
        } else if let Some(r) = self.users.get_mut(&None) {
            if r.when.is_none() || r.when.unwrap() <= ic_cdk::api::time() {
                r.was_read = true;
                self.read_by.insert(user.to_string());
                self.history.append(&mut vec![HistoryEntry {
                    action: if self.locked {
                        "read".to_string()
                    } else {
                        "read-locked".to_string()
                    },
                    user: Some(user.to_string()),
                    rule: Some((None, r.when)),
                    created_at: ic_cdk::api::time(),
                }]);
                self.locked = true;
                return true;
            }
        }
        false
    }
    // add a new reader to the note
    pub fn add_reader(&mut self, user: Option<String>, when: Option<u64>) -> bool {
        if self.locked && (user.is_none() || self.read_by.contains(&user.clone().unwrap())) {
            // If this note is locked and the user has already read it then this doesn't seem useful.
            return false;
        }
        self.history.append(&mut vec![HistoryEntry {
            action: "share".to_string(),
            user: user.clone(),
            rule: Some((user.clone(), when)),
            created_at: ic_cdk::api::time(),
        }]);
        self.users.insert(
            user.clone(),
            PrincipalRule {
                was_read: false,
                when,
            },
        );
        true
    }
    // Was the note ever read by that user
    pub fn user_read(&self, user: &PrincipalName) -> bool {
        self.read_by.contains(user)
    }
    // Remove a reader (will return false if the note was already read by the user)
    pub fn remove_reader(&mut self, user: &Option<PrincipalName>) -> bool {
        if self.locked {
            if user.iter().any(|u| self.read_by.contains(u)) {
                return false;
            } else if let Some(r) = self.users.get(user) {
                if r.was_read {
                    return false;
                }
            }
        }
        if self.users.contains_key(user) {
            self.users.remove(user);
            true
        } else {
            false
        }
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

ic_cdk::export_candid!();
