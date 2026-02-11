<script lang="ts">
  import Editor from "$lib/editor/Editor.svelte";
  import { userSettings } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
  import init from "$lib/converter/pkg/converter";
  import { onMount } from "svelte";
  import { ExclamationCircle, FileText } from "svelte-bootstrap-icons";
  import { slide, fade } from "svelte/transition";

  let name = $state("Anonymous" + Math.floor(Math.random() * 100));
  let color = $state("#e83d84");
  let showOutline = $state(false);
  let scenes = $state<{ name: string; pos: number }[]>([]);

  let editorRef: Editor | null = null;
  function applyUserUpdate() {
    editorRef?.updateUser({ name, color });
  }

  function updatePreview() {
    if (editorRef) {
      editorRef.updatePreview();
    }
  }

  function toggleOutline() {
    if (!showOutline && editorRef) {
      scenes = editorRef.getSceneList();
    }
    showOutline = !showOutline;
  }

  function handleSceneClick(pos: number) {
    editorRef?.scrollIntoView(pos);
    showOutline = false;
  }

  onMount(async () => {
    await init();
    updatePreview();
  });
</script>

<header
  class="flex items-center justify-between px-4 py-3 bg-[#252526] border-b border-gray-700 shadow-md z-10"
>
  <div class="flex items-center gap-4">
    <div class="flex flex-col">
      <label
        for="user-name"
        class="text-[10px] uppercase font-bold text-gray-400 mb-1 ml-1 tracking-wider"
        >User Identity</label
      >
      <div class="flex gap-2">
        <textarea
          id="user-name"
          bind:value={name}
          class="h-8 px-2 py-1 text-sm border border-gray-600 rounded bg-[#3c3c3c] text-white focus:border-blue-500 focus:outline-none resize-none w-32"
        ></textarea>
        <textarea
          bind:value={color}
          class="h-8 px-2 py-1 text-sm border border-gray-600 rounded bg-[#3c3c3c] text-white focus:border-blue-500 focus:outline-none resize-none w-24 font-mono"
        ></textarea>
      </div>
    </div>

    <button
      class="mt-5 px-3 py-1 rounded border border-gray-500 text-gray-300 text-xs font-semibold hover:bg-gray-700 hover:text-white transition h-8"
      onclick={applyUserUpdate}
    >
      Update
    </button>
  </div>

  <div class="flex items-center gap-2 mt-5">
    <button
      class="px-3 py-1 rounded border border-gray-600 text-gray-400 text-xs font-medium hover:bg-[#3c3c3c] transition h-8"
      onclick={() => (userSettings.vimEnabled = !userSettings.vimEnabled)}
    >
      {userSettings.vimEnabled ? "Vim ON" : "Vim OFF"}
    </button>

    <button
      class="px-4 py-1 rounded bg-blue-600 text-white text-xs font-bold hover:bg-blue-500 shadow-lg transition h-8 uppercase tracking-tight"
      onclick={updatePreview}
    >
      Run Preview
    </button>
  </div>
</header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <!-- Side bar -->
  <aside
    class="w-12 bg-[#333333] border-r border-gray-700 flex flex-col items-center py-4 gap-4"
  >
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      title="Document outline"
      onclick={toggleOutline}
    >
      <FileText />
    </button>

    <div class="mt-auto flex flex-col items-center gap-4">
      <a href="https://github.com/frblo/dionysus/issues" target="_blank">
        <button
          class="p-2 text-gray-400 hover:text-white transition-colors"
          title="Report issue"
        >
          <ExclamationCircle />
        </button>
      </a>
    </div>
  </aside>
  {#if showOutline}
    <div class="fixed inset-0 z-20" onclick={() => (showOutline = false)}></div>

    <aside
      class="absolute top-19 left-12 bottom-0 w-72 bg-[#252526] border-r border-gray-700 flex flex-col z-30 shadow-2xl"
      transition:slide={{ axis: "x", duration: 200 }}
    >
      <div
        class="px-4 py-3 border-b border-gray-700 flex justify-between items-center"
      >
        <span class="text-xs font-bold uppercase tracking-wider text-gray-400"
          >Outline</span
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
              {index}. {scene.name || "Untitled Scene"}
            </button>
          {/each}
        {/if}
      </div>
    </aside>
  {/if}
  <main class="flex flex-1 overflow-hidden bg-[#1e1e1e]">
    <section
      class="w-1/2 border-r border-gray-700 flex flex-col overflow-hidden"
    >
      <div class="flex-1 overflow-auto">
        <Editor bind:this={editorRef} user={{ name, color }} />
      </div>
    </section>

    <section class="w-1/2 overflow-hidden bg-[#1e1e1e]">
      {#if preview.html}
        <iframe
          title="Screenplay Preview"
          srcdoc={preview.html}
          class="w-full h-full border-none bg-white"
        ></iframe>
      {:else}
        <div
          class="flex flex-col items-center justify-center h-full text-gray-500 text-center p-12"
        >
          <p class="italic mb-2">No content rendered.</p>
          <p class="text-xs opacity-70">
            No content found. Try writing something and then click "RUN
            PREVIEW", press
            <kbd class="bg-gray-800 px-1 rounded text-gray-300">CTRL+S</kbd> or
            run command
            <kbd class="bg-gray-800 px-1 rounded text-gray-300">:w</kbd> to see preview...
          </p>
        </div>
      {/if}
    </section>
  </main>
</div>
