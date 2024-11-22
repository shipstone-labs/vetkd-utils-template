declare module "*.wasm" {
  const value: () => Promise<WebAssembly.Module>;
  export default value;
}

declare module "Trash.svelte" {
  import type { ConstructorOfATypedSvelteComponent } from "svelte";
  const component: ConstructorOfATypedSvelteComponent;
  export default component;
}
