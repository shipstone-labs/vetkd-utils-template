declare module "typewriter-editor/lib/BubbleMenu.svelte" {
  import { SvelteComponentTyped } from "svelte";
  export default class BubbleMenu extends SvelteComponentTyped {}
}

declare module "typewriter-editor/dist/asRoot.js" {
  export function asRoot(
    root: HTMLElement,
    editor: Editor
  ): {
    update: (newEditor: Editor) => void;
    destroy: () => void;
  };
}

declare module "svelte-icons/*" {
  import { SvelteComponentTyped } from "svelte";
  export default class SvelteIcon extends SvelteComponentTyped {}
}

declare module "*.svelte" {
  import { SvelteComponentTyped } from "svelte";
  export default class SvelteComponent extends SvelteComponentTyped {}
}
