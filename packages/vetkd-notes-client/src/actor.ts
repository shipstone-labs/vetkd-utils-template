import {
  Actor,
  type ActorConfig,
  type ActorSubclass,
  HttpAgent,
  type HttpAgentOptions,
} from "@dfinity/agent";
import { ENCRYPTED_NOTES_CANISTER_ID, type _SERVICE } from "./backend.js";
import { idlFactory } from "./backend.js";

export type BackendActor = ActorSubclass<_SERVICE>;

export function createActor(_options?: {
  agentOptions?: HttpAgentOptions;
  actorOptions?: ActorConfig;
}): BackendActor {
  const hostOptions = {
    host:
      process.env.DFX_NETWORK === "ic"
        ? `https://${ENCRYPTED_NOTES_CANISTER_ID}.ic0.app`
        : "http://localhost:8000",
  };
  let options = _options;
  if (!options) {
    options = {
      agentOptions: hostOptions,
    };
  } else if (!options.agentOptions) {
    options.agentOptions = hostOptions;
  } else {
    options.agentOptions.host = hostOptions.host;
  }

  const agent = new HttpAgent({ ...options.agentOptions });
  // Fetch root key for certificate validation during development
  if (process.env.NODE_ENV !== "production") {
    console.log("Dev environment - fetching root key...");

    agent.fetchRootKey().catch((err) => {
      console.warn(
        "Unable to fetch root key. Check to ensure that your local replica is running"
      );
      console.error(err);
    });
  }

  // Creates an actor with using the candid interface and the HttpAgent
  return Actor.createActor(idlFactory, {
    agent,
    canisterId: ENCRYPTED_NOTES_CANISTER_ID,
    ...options?.actorOptions,
  });
}
