export const idlFactory = ({ IDL }) => {
  const HistoryEntry = IDL.Record({
    'action' : IDL.Text,
    'labels' : IDL.Vec(IDL.Text),
    'rule' : IDL.Opt(IDL.Tuple(IDL.Text, IDL.Opt(IDL.Nat64))),
    'user' : IDL.Text,
    'created_at' : IDL.Nat64,
  });
  const PrincipalRule = IDL.Record({
    'when' : IDL.Opt(IDL.Nat64),
    'was_read' : IDL.Bool,
  });
  const EncryptedNote = IDL.Record({
    'id' : IDL.Nat,
    'read_by' : IDL.Vec(IDL.Text),
    'updated_at' : IDL.Nat64,
    'encrypted_text' : IDL.Text,
    'owner' : IDL.Text,
    'data' : IDL.Text,
    'locked' : IDL.Bool,
    'history' : IDL.Vec(HistoryEntry),
    'created_at' : IDL.Nat64,
    'users' : IDL.Vec(IDL.Tuple(IDL.Text, PrincipalRule)),
  });
  return IDL.Service({
    'add_user' : IDL.Func(
        [IDL.Nat, IDL.Opt(IDL.Text), IDL.Opt(IDL.Nat64)],
        [],
        [],
      ),
    'create_note' : IDL.Func([], [IDL.Nat], []),
    'delete_note' : IDL.Func([IDL.Nat], [], []),
    'encrypted_symmetric_key_for_note' : IDL.Func(
        [IDL.Nat, IDL.Vec(IDL.Nat8)],
        [IDL.Text],
        [],
      ),
    'get_notes' : IDL.Func([], [IDL.Vec(EncryptedNote)], []),
    'refresh_note' : IDL.Func([IDL.Nat], [EncryptedNote], []),
    'remove_user' : IDL.Func([IDL.Nat, IDL.Opt(IDL.Text)], [], []),
    'symmetric_key_verification_key_for_note' : IDL.Func([], [IDL.Text], []),
    'update_note' : IDL.Func([IDL.Nat, IDL.Text, IDL.Text], [], []),
    'whoami' : IDL.Func([], [IDL.Text], []),
  });
};
export const init = ({ IDL }) => { return []; };
