export type {
  _SERVICE,
  EncryptedNote,
  PrincipalRule,
  HistoryEntry,
} from "./declarations/vetkd_notes/vetkd_notes.did.d.ts";
export { idlFactory } from "./declarations/vetkd_notes/vetkd_notes.did.js";
export const ENCRYPTED_NOTES_CANISTER_ID = process.env.VETKD_NOTES_CANISTER_ID;
