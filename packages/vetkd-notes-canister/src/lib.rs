use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::api::time;
use ic_cdk_macros::*;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{
    storable::Bound, DefaultMemoryImpl, StableBTreeMap, StableCell, Storable,
};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq, Hash)]
struct PrincipalEntry {
    name: Option<String>,
    when: Option<u64>,
}

type PrincipalName = String;
type Memory = VirtualMemory<DefaultMemoryImpl>;
type NoteId = u128;

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq, Hash)]
pub struct HistoryEntry {
    action: String,
    user: Option<String>,
    when: Option<u64>,
    created_at: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize, Eq, PartialEq, Hash)]
pub struct EncryptedNote {
    id: NoteId,
    encrypted_text: String,
    data: String,
    owner: PrincipalName,
    /// Principals with whom this note is shared. Does not include the owner.
    /// Needed to be able to efficiently show in the UI with whom this note is shared.
    users: Vec<PrincipalEntry>,

    locked: bool,
    created_at: u64,
    updated_at: u64,
    history: Vec<HistoryEntry>,
}

impl Default for EncryptedNote {
    fn default() -> Self {
        EncryptedNote {
            id: 0,
            encrypted_text: "".to_string(),
            data: "".to_string(),
            owner: "".to_string(),
            users: vec![],
            locked: false,
            created_at: time(),
            updated_at: time(),
            history: vec![],
        }
    }
}

impl EncryptedNote {
    pub fn is_authorized(&self, user: &PrincipalName) -> bool {
        user == &self.owner
            || self.users.iter().any(|u| {
                let name = u.name.clone();
                (name.is_none() || &name.unwrap() == user)
                    && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
            })
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

// We use a canister's stable memory as storage. This simplifies the code and makes the appliation
// more robust because no (potentially failing) pre_upgrade/post_upgrade hooks are needed.
// Note that stable memory is less performant than heap memory, however.
// Currently, a single canister smart contract is limited to 96 GB of stable memory.
// For the current limits see https://internetcomputer.org/docs/current/developer-docs/production/resource-limits.
// To ensure that our canister does not exceed the limit, we put various restrictions (e.g., number of users) in place.
static MAX_USERS: u64 = 1_000;
static MAX_NOTES_PER_USER: usize = 50;
static MAX_NOTE_CHARS: usize = 100000;
static MAX_SHARES_PER_NOTE: usize = 50;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static NEXT_NOTE_ID: RefCell<StableCell<NoteId, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(0))),
            1
        ).expect("failed to init NEXT_NOTE_ID")
    );

    static NOTES: RefCell<StableBTreeMap<NoteId, EncryptedNote, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(1))),
        )
    );

    static NOTE_OWNERS: RefCell<StableBTreeMap<PrincipalName, NoteIds, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(2))),
        )
    );

    static NOTE_SHARES: RefCell<StableBTreeMap<PrincipalName, NoteIds, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with_borrow(|m| m.get(MemoryId::new(3))),
        )
    );
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

/// --- Queries vs. Updates ---
///
/// Note that our public methods are declared as an *updates* rather than *queries*, e.g.:
/// #[update(name = "notesCnt")] ...
/// rather than
/// #[query(name = "notesCnt")] ...
///
/// While queries are significantly faster than updates, they are not certified by the IC.
/// Thus, we avoid using queries throughout this dapp, ensuring that the result of our
/// methods gets through consensus. Otherwise, this method could e.g. omit some notes
/// if it got executed by a malicious node. (To make the dapp more efficient, one could
/// use an approach in which both queries and updates are combined.)
///
/// See https://internetcomputer.org/docs/current/concepts/canisters-code#query-and-update-methods

/// Reflects the [caller]'s identity by returning (a future of) its principal.
/// Useful for debugging.
#[update]
fn whoami() -> String {
    ic_cdk::caller().to_string()
}

/// General assumptions
/// -------------------
/// All the functions of this canister's public API should be available only to
/// registered users, with the exception of [whoami].

/// Returns (a future of) this [caller]'s notes.
/// Panics:
///     [caller] is the anonymous identity
#[update]
fn get_notes() -> Vec<EncryptedNote> {
    let user_str = caller().to_string();
    NOTES.with_borrow(|notes| {
        let mut result = HashSet::<EncryptedNote>::new();
        let owned = NOTE_OWNERS.with_borrow(|ids| {
            ids.get(&user_str)
                .unwrap_or_default()
                .iter()
                .map(|id| notes.get(id).ok_or(format!("missing note with ID {id}")))
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|err| ic_cdk::trap(&err))
        });
        let shared = NOTE_SHARES.with_borrow(|ids| {
            ids.get(&user_str)
                .unwrap_or_default()
                .iter()
                .map(|id| notes.get(id).ok_or(format!("missing note with ID {id}")))
                .filter(|note| {
                    if let Ok(item) = note {
                        if owned.clone().iter().any(|u| u.id == item.id) {
                            return false;
                        }
                        item.users.iter().any(|u| {
                            if let Some(name) = &u.name {
                                name == &user_str
                                    && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
                            } else {
                                u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time()
                            }
                        })
                    } else {
                        false
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|err| ic_cdk::trap(&err))
        });
        let public = NOTE_SHARES.with_borrow(|ids| {
            ids.get(&"everybody".to_string())
                .unwrap_or_default()
                .iter()
                .map(|id| notes.get(id).ok_or(format!("missing note with ID {id}")))
                .filter(|note| {
                    if let Ok(item) = note {
                        if owned.clone().iter().any(|u| u.id == item.id)
                            || shared.clone().iter().any(|u| u.id == item.id)
                        {
                            return false;
                        }
                        item.users.iter().any(|u| {
                            if let Some(name) = &u.name {
                                name == &user_str
                                    && (u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time())
                            } else {
                                u.when.is_none() || u.when.unwrap() <= ic_cdk::api::time()
                            }
                        })
                    } else {
                        false
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|err| ic_cdk::trap(&err))
        });
        // Use `extend` to add the notes to the `HashSet`
        result.extend(owned); // Convert Vec into iterator and extend the HashSet
        result.extend(shared);
        result.extend(public);

        // Convert the HashSet into a Vec to return the unique values
        let mut output: Vec<_> = result.into_iter().collect();
        output.sort_by_key(|note| note.id);
        output
    })
}

/// Delete this [caller]'s note with given id. If none of the
/// existing notes have this id, do nothing.
/// [id]: the id of the note to be deleted
///
/// Returns:
///      Future of unit
/// Panics:
///      [caller] is the anonymous identity
///      [caller] is not the owner of note with id `note_id`
#[update]
fn delete_note(note_id: u128) {
    let user_str = caller().to_string();
    NOTES.with_borrow_mut(|notes| {
        if let Some(note_to_delete) = notes.get(&note_id) {
            let owner = &note_to_delete.owner;
            if owner != &user_str || note_to_delete.locked {
                ic_cdk::trap("only the owner can delete unlocked notes");
            }
            NOTE_OWNERS.with_borrow_mut(|owner_to_nids| {
                if let Some(mut owner_ids) = owner_to_nids.get(owner) {
                    owner_ids.ids.retain(|&id| id != note_id);
                    if !owner_ids.ids.is_empty() {
                        owner_to_nids.insert(owner.clone(), owner_ids);
                    } else {
                        owner_to_nids.remove(owner);
                    }
                }
            });
            NOTE_SHARES.with_borrow_mut(|share_to_nids| {
                for share in note_to_delete.users {
                    let share_key = share.name.unwrap_or_else(|| "everybody".to_string());
                    if let Some(mut share_ids) = share_to_nids.get(&share_key) {
                        share_ids.ids.retain(|&id| id != note_id);
                        if !share_ids.ids.is_empty() {
                            share_to_nids.insert(share_key, share_ids);
                        } else {
                            share_to_nids.remove(&share_key);
                        }
                    }
                }
            });
            notes.remove(&note_id);
        }
    });
}

/// Replaces the encrypted text of note with ID [id] with [encrypted_text].
///
/// Panics:
///     [caller] is the anonymous identity
///     [caller] is not the note's owner and not a user with whom the note is shared
///     [encrypted_text] exceeds [MAX_NOTE_CHARS]
#[update]
fn update_note(id: NoteId, data: String, encrypted_text: String) {
    let user_str = caller().to_string();

    NOTES.with_borrow_mut(|notes| {
        if let Some(mut note_to_update) = notes.get(&id) {
            if !note_to_update.is_authorized(&user_str) || note_to_update.locked {
                ic_cdk::trap("unauthorized update");
            }
            assert!(encrypted_text.chars().count() <= MAX_NOTE_CHARS);
            note_to_update.encrypted_text = encrypted_text;
            note_to_update.data = data;
            note_to_update.updated_at = ic_cdk::api::time();
            note_to_update.history.push(HistoryEntry {
                action: "updated".to_string(),
                user: Some(user_str),
                when: None,
                created_at: ic_cdk::api::time(),
            });

            notes.insert(id, note_to_update);
        }
    })
}

/// Add new empty note for this [caller].
///
/// Returns:
///      Future of ID of new empty note
/// Panics:
///      [caller] is the anonymous identity
///      User already has [MAX_NOTES_PER_USER] notes
///      This is the first note for [caller] and [MAX_USERS] is exceeded
#[update]
fn create_note() -> NoteId {
    let owner = caller().to_string();

    NOTES.with_borrow_mut(|id_to_note| {
        NOTE_OWNERS.with_borrow_mut(|owner_to_nids| {
            let next_note_id = NEXT_NOTE_ID.with_borrow(|id| *id.get());
            let new_note = EncryptedNote {
                id: next_note_id,
                owner: owner.clone(),
                data: String::new(),
                users: vec![],
                encrypted_text: String::new(),
                locked: false,
                created_at: time(),
                updated_at: time(),
                history: vec![HistoryEntry {
                    action: "created".to_string(),
                    user: Some(owner.clone()),
                    when: None,
                    created_at: ic_cdk::api::time(),
                }],
            };

            if let Some(mut owner_nids) = owner_to_nids.get(&owner) {
                assert!(owner_nids.ids.len() < MAX_NOTES_PER_USER);
                owner_nids.ids.push(new_note.id);
                owner_to_nids.insert(owner, owner_nids);
            } else {
                assert!(owner_to_nids.len() < MAX_USERS);
                owner_to_nids.insert(
                    owner,
                    NoteIds {
                        ids: vec![new_note.id],
                    },
                );
            }
            assert_eq!(id_to_note.insert(new_note.id, new_note), None);

            NEXT_NOTE_ID.with_borrow_mut(|next_note_id| {
                next_note_id
                    .set(next_note_id.get() + 1)
                    .unwrap_or_else(|_e| ic_cdk::trap("failed to set NEXT_NOTE_ID"))
            });
            next_note_id
        })
    })
}

/// Shares the note with ID `note_id`` with the `user`.
/// Has no effect if the note is already shared with that user.
///
/// Panics:
///      [caller] is the anonymous identity
///      [caller] is not the owner of note with id `note_id`
#[update]
fn add_user(note_id: NoteId, user: PrincipalEntry) {
    let caller_str = caller().to_string();
    NOTES.with_borrow_mut(|notes| {
        NOTE_SHARES.with_borrow_mut(|user_to_nids| {
            if let Some(mut note) = notes.get(&note_id) {
                let owner = &note.owner;
                if owner != &caller_str {
                    ic_cdk::trap("only the owner can share the note");
                }
                assert!(note.users.len() < MAX_SHARES_PER_NOTE);

                if !note.locked {
                    note.locked = true;
                    note.history.push(HistoryEntry {
                        action: "locked".to_string(),
                        user: None,
                        when: None,
                        created_at: ic_cdk::api::time(),
                    });
                }
                note.history.push(HistoryEntry {
                    action: "shared".to_string(),
                    user: user.name.clone(),
                    when: user.when,
                    created_at: ic_cdk::api::time(),
                });
                if let Some(entry) = note.users.iter().position(|u| u.clone().name == user.name) {
                    let entry_item = note.users.get_mut(entry).unwrap();
                    entry_item.when = user.when;
                }
                if !note.users.contains(&user) {
                    note.users.push(user.clone());
                    notes.insert(note_id, note);
                }
                let user_name = user.name.unwrap_or_else(|| "everybody".to_string());
                if let Some(mut user_ids) = user_to_nids.get(&user_name) {
                    if !user_ids.ids.contains(&note_id) {
                        user_ids.ids.push(note_id);
                        user_to_nids.insert(user_name, user_ids);
                    }
                } else {
                    user_to_nids.insert(user_name, NoteIds { ids: vec![note_id] });
                }
            }
        })
    });
}

/// Unshares the note with ID `note_id`` with the `user`.
/// Has no effect if the note is not shared with that user.
///
/// Panics:
///      [caller] is the anonymous identity
///      [caller] is not the owner of note with id `note_id`
#[update]
fn remove_user(note_id: NoteId, user: Option<PrincipalName>) {
    let caller_str = caller().to_string();
    NOTES.with_borrow_mut(|notes| {
        NOTE_SHARES.with_borrow_mut(|user_to_nids| {
            if let Some(mut note) = notes.get(&note_id) {
                let owner = &note.owner;
                if owner != &caller_str {
                    ic_cdk::trap("only the owner can share the note");
                }

                note.users.retain(|u| u.name != user);
                note.history.push(HistoryEntry {
                    action: "unshared".to_string(),
                    user: user.clone(),
                    when: None,
                    created_at: ic_cdk::api::time(),
                });
                notes.insert(note_id, note);

                let user_name = user.unwrap_or_else(|| "everybody".to_string());
                if let Some(mut user_ids) = user_to_nids.get(&user_name) {
                    user_ids.ids.retain(|&id| id != note_id);
                    if !user_ids.ids.is_empty() {
                        user_to_nids.insert(user_name, user_ids);
                    } else {
                        user_to_nids.remove(&user_name);
                    }
                }
            }
        })
    });
}

mod vetkd_types;

const VETKD_SYSTEM_API_CANISTER_ID: &str = "nn664-2iaaa-aaaao-a3tqq-cai";

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

#[update]
async fn encrypted_symmetric_key_for_note(
    note_id: NoteId,
    encryption_public_key: Vec<u8>,
) -> String {
    let user_str = caller().to_string();
    let request = NOTES.with_borrow_mut(|notes| {
        if let Some(mut note) = notes.get(&note_id) {
            if !note.is_authorized(&user_str) {
                ic_cdk::trap(&format!("unauthorized key request by user {user_str}"));
            }
            let user_str_clone = &Some(user_str.clone());
            if !note
                .history
                .iter()
                .any(|entry| entry.action == "read" && &entry.user == user_str_clone)
            {
                note.history.push(HistoryEntry {
                    action: "read".to_string(),
                    user: Some(user_str.clone()),
                    when: None,
                    created_at: ic_cdk::api::time(),
                });
            }
            VetKDEncryptedKeyRequest {
                derivation_id: {
                    let mut buf = vec![];
                    buf.extend_from_slice(&note_id.to_be_bytes()); // fixed-size encoding
                    buf.extend_from_slice(note.owner.as_bytes());
                    buf // prefix-free
                },
                public_key_derivation_path: vec![b"note_symmetric_key".to_vec()],
                key_id: bls12_381_test_key_1(),
                encryption_public_key,
            }
        } else {
            ic_cdk::trap(&format!("note with ID {note_id} does not exist"));
        }
    });

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
