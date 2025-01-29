import type { BackendActor } from "./actor.js";
import type { EncryptedNote, PrincipalRule, HistoryEntry } from "./backend.js";
import type { CryptoService } from "./crypto.js";
import type { Principal } from "@dfinity/principal";

export interface NoteModel {
  id: bigint;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
  tags: Array<string>;
  owner: string;
  users: Array<[string, PrincipalRule]>;
  locked: boolean;
  history: Array<HistoryEntry>;
}

type SerializableNoteModel = Pick<NoteModel, "content">;
type SerializableNoteMetadataModel = Pick<
  NoteModel,
  "title" | "createdAt" | "updatedAt" | "tags"
>;

export function noteFromContent(
  content: string,
  tags: string[],
  self_principal: Principal
): NoteModel {
  const title = extractTitle(content);
  const creationTime = Date.now();

  return {
    id: BigInt(0),
    title,
    content,
    tags,
    owner: self_principal.toString(),
    users: [],
    createdAt: creationTime,
    updatedAt: creationTime,
    locked: false,
    history: [],
  };
}

export async function serialize(
  note: NoteModel,
  cryptoService: CryptoService
): Promise<EncryptedNote> {
  const data: SerializableNoteMetadataModel = {
    title: note.title,
    createdAt: note.createdAt,
    updatedAt: note.updatedAt,
    tags: note.tags,
  };
  const serializableNote: SerializableNoteModel = {
    content: note.content,
  };
  const encryptedNote = await cryptoService.encryptWithNoteKey(
    note.id,
    note.owner,
    JSON.stringify(serializableNote)
  );
  return {
    id: note.id,
    encrypted_text: encryptedNote,
    data: JSON.stringify(data),
    owner: note.owner,
    users: note.users,
    history: [],
    read_by: [],
    created_at: BigInt(note.createdAt * 1000000),
    updated_at: BigInt(note.updatedAt * 1000000),
    locked: note.locked,
  };
}

export async function deserialize(
  enote_: EncryptedNote,
  cryptoService: CryptoService,
  actor: BackendActor
): Promise<NoteModel> {
  let enote = enote_;
  const refresh = async (note_id: bigint) => {
    // This is during initial load of the note so we don't need to immediately notify.
    enote = await actor.refresh_note(note_id);
    return enote.encrypted_text;
  };
  const serializedNote = await cryptoService.decryptWithNoteKey(
    enote.id,
    enote.owner,
    enote.encrypted_text,
    refresh
  );
  let deserializedNote: SerializableNoteModel = {
    content:
      "<b>Decryption corrupted (possibly different temporary vetkd_system_api)</b>",
  };
  try {
    deserializedNote = JSON.parse(serializedNote);
  } catch (e) {
    console.error("Failed to parse note content", e, serializedNote);
  }
  let data: SerializableNoteMetadataModel = {
    tags: [],
  } as SerializableNoteMetadataModel;
  try {
    data = JSON.parse(enote.data || '{"tags": []}');
  } catch (e) {
    console.error("Failed to parse note data", e, enote.data);
  }
  return {
    id: enote.id,
    owner: enote.owner,
    users: enote.users,
    ...deserializedNote,
    ...data,
    history: enote.history.map((entry) => ({
      action: entry.action,
      user: entry.user,
      rule: entry.rule,
      labels: entry.labels,
      created_at: entry.created_at,
    })),
    createdAt: Number(enote.created_at / BigInt(1000000)),
    updatedAt: Number(enote.updated_at / BigInt(1000000)),
    locked: enote.locked,
  };
}

export function summarize(note: NoteModel, maxLength = 50) {
  const div = document.createElement("div");
  div.innerHTML = note.content;

  let text = div.innerText;
  const title = extractTitleFromDomEl(div);
  if (title) {
    text = text.replace(title, "");
  }

  return text.slice(0, maxLength) + (text.length > maxLength ? "..." : "");
}

function extractTitleFromDomEl(el: HTMLElement) {
  const title = el.querySelector("h1");
  if (title) {
    return title.innerText;
  }

  const blockElements = el.querySelectorAll(
    "h1,h2,p,li"
  ) as NodeListOf<HTMLElement>;
  for (const el of blockElements) {
    if (el.innerText?.trim().length > 0) {
      return el.innerText.trim();
    }
  }
  return "";
}

export function extractTitle(html: string) {
  const div = document.createElement("div");
  div.innerHTML = html;
  return extractTitleFromDomEl(div);
}
