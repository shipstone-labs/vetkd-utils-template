<script lang="ts">
import { createEventDispatcher } from "svelte";

export let tags: string[];
// biome-ignore lint/style/useConst: <explanation>
export let disabled = false;

let newTag = "";
let newTagInput: HTMLInputElement;

const dispatch = createEventDispatcher<{
	add: string;
	remove: string;
}>();

function add() {
  if (disabled) {
    return;
  }
	dispatch("add", newTag);
	newTag = "";
	newTagInput.focus();
}

function remove(tag: string) {
  if (disabled) {
    return;
  }
  dispatch("remove", tag);
}

function onKeyPress(e: KeyboardEvent) {
	if (e.key === "Enter" && newTag.trim().length > 0 && !tags.includes(newTag) && !disabled) {
		add();
	}
}
</script>

<div class="flex flex-wrap gap-2">
  {#each tags as tag}
    <button
      class="btn btn-outline btn-sm flex items-center"
      disabled={disabled}
      on:click={() => remove(tag)}
    >
      <span>{tag}</span>
      {#if !disabled}<svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        class="inline-block w-4 h-4 stroke-current"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M6 18L18 6M6 6l12 12"
        />
      </svg>{/if}
    </button>
  {/each}
  {#if !disabled}<input
    bind:value={newTag}
    placeholder="Add tag..."
    class="bg-transparent  text-base  rounded-lg h-8 px-3 w-32 {disabled
      ? 'opacity-50'
      : ''}"
    bind:this={newTagInput}
    on:keypress={onKeyPress}
    {disabled}
  />
  <button
    class="btn btn-sm btn-ghost"
    on:click={add}
    disabled={newTag.trim().length === 0 || tags.includes(newTag) || disabled}
    >Add</button
  >{/if}
</div>
