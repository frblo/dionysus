<script lang="ts">
  import { slide } from "svelte/transition";
  import { XSquare } from "svelte-bootstrap-icons";

  let { scenes = [], handleSceneClick, toggleOutline } = $props();
</script>

<aside
  class="absolute top-19 left-12 bottom-0 w-72 bg-[#252526] border-r border-gray-700 flex flex-col z-30 shadow-2xl"
  transition:slide={{ axis: "x", duration: 200 }}
>
  <div
    class="px-4 py-3 border-b border-gray-700 flex justify-between items-center"
  >
    <span class="text-xs font-bold uppercase tracking-wider text-gray-400"
      >Scenes Outline</span
    >
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      onclick={toggleOutline}
      type="button"><XSquare /></button
    >
  </div>

  <div class="flex-1 overflow-y-auto custom-scrollbar">
    {#if scenes.length === 0}
      <p class="p-4 text-xs text-gray-500 italic">No scenes found</p>
    {:else}
      {#each scenes as scene, index}
        <button
          class="w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-[#37373d] hover:text-white transition-colors truncate border-l-2 border-transparent focus:border-blue-500 outline-none"
          onclick={() => handleSceneClick(scene.pos)}
        >
          <span class="text-gray-600">{index + 1}.</span>
          {scene.name || "Untitled Scene"}
        </button>
      {/each}
    {/if}
  </div>
</aside>
