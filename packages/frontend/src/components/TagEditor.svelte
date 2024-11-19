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
	dispatch("add", newTag);
	newTag = "";
	newTagInput.focus();
}

function remove(tag: string) {
	dispatch("remove", tag);
}

function onKeyPress(e) {
	if (e.key === "Enter" && newTag.trim().length > 0 && !tags.includes(newTag)) {
		add();
	}
}
</script>

<div class="flex flex-wrap gap-2">
  {#each tags as tag}
    <button
      class="border border-gray-400 text-gray-700 rounded-md py-1 px-2 text-sm flex items-center"
      on:click={() => remove(tag)}
    >
      <span>{tag}</span>
      <svg
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
      </svg>
    </button>
  {/each}
  <input
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
    class="text-gray-700 bg-transparent border-none py-1 px-2 text-sm hover:bg-gray-100"
    on:click={add}
    disabled={newTag.trim().length === 0 || tags.includes(newTag) || disabled}
    >Add</button
  >
</div>
