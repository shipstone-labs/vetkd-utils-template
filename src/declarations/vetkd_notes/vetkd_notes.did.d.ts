import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface EncryptedNote {
  'id' : bigint,
  'read_by' : Array<string>,
  'updated_at' : bigint,
  'encrypted_text' : string,
  'owner' : string,
  'data' : string,
  'locked' : boolean,
  'history' : Array<HistoryEntry>,
  'created_at' : bigint,
  'users' : Array<[string, PrincipalRule]>,
}
export interface HistoryEntry {
  'action' : string,
  'labels' : Array<string>,
  'rule' : [] | [[string, [] | [bigint]]],
  'user' : string,
  'created_at' : bigint,
}
export interface PrincipalRule { 'when' : [] | [bigint], 'was_read' : boolean }
export interface _SERVICE {
  'add_user' : ActorMethod<[bigint, [] | [string], [] | [bigint]], undefined>,
  'create_note' : ActorMethod<[], bigint>,
  'delete_note' : ActorMethod<[bigint], undefined>,
  'encrypted_symmetric_key_for_note' : ActorMethod<
    [bigint, Uint8Array | number[]],
    string
  >,
  'get_notes' : ActorMethod<[], Array<EncryptedNote>>,
  'refresh_note' : ActorMethod<[bigint], EncryptedNote>,
  'remove_user' : ActorMethod<[bigint, [] | [string]], undefined>,
  'symmetric_key_verification_key_for_note' : ActorMethod<[], string>,
  'update_note' : ActorMethod<[bigint, string, string], undefined>,
  'whoami' : ActorMethod<[], string>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
