<script lang="ts">
import { fly } from "svelte/transition";
import DisclaimerCopy from "./DisclaimerCopy.svelte";
let isDismissed = !!window.localStorage.getItem("disclaimer-dismissed");

function dismiss() {
	window.localStorage.setItem("disclaimer-dismissed", "yes");
	isDismissed = true;
}
</script>

{#if !isDismissed}
  <div
    class="sticky bottom-0 p-4 text-xs bg-base-300 mt-4 sm:flex"
    out:fly={{ y: 50 }}
  >
    <p class="opacity-90 sm:flex-1">
      <DisclaimerCopy />
    </p>

    <button
      class="border border-gray-400 text-gray-700 rounded-md py-0.5 px-1 text-xs sm:text-sm sm:self-start mt-4 sm:mt-0 sm:ml-4 opacity-90 hover:bg-gray-100"
      on:click={dismiss}>I understand</button
    >
  </div>
{/if}
