<script lang="ts">
  import { slide } from "svelte/transition";
  import {
    editorViewSettings,
    userSettings,
    SidebarMenus,
  } from "$lib/state/settings.svelte";
</script>

{#if editorViewSettings.sidebarMenuOpen === SidebarMenus.Settings}
  <aside
    class="relative h-full w-72 bg-[#252526] border-r border-gray-700 flex flex-col z-30"
    transition:slide={{ axis: "x", duration: 200 }}
  >
    <div
      class="px-4 py-3 border-b border-gray-700 flex justify-between items-center"
    >
      <span class="text-xs font-bold uppercase tracking-wider text-gray-400"
        >Settings</span
      >
    </div>

    <div class="flex-1 overflow-y-auto custom-scrollbar">
      <label
        class="flex items-center justify-between gap-3 px-4 py-3 hover:bg-[#37373d] transition-colors cursor-pointer"
      >
        <span class="text-sm text-gray-300">Vim mode</span>
        <input
          type="checkbox"
          class="w-4 h-4 accent-blue-500 cursor-pointer"
          bind:checked={userSettings.current.vimEnabled}
        />
      </label>

      <label
        class="flex items-center justify-between gap-3 px-4 py-3 hover:bg-[#37373d] transition-colors cursor-pointer"
      >
        <span class="text-sm text-gray-300">Highlight trailing spaces</span>
        <input
          type="checkbox"
          class="w-4 h-4 accent-blue-500 cursor-pointer"
          bind:checked={userSettings.current.highlighTrailingSpacesEnabled}
        />
      </label>
    </div>
  </aside>
{/if}
