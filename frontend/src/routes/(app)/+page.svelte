<script lang="ts">
  import Editor from "$lib/editor/Editor.svelte";
  import { userSettings } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
  import init from "$lib/converter/pkg/converter";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { ExclamationCircle } from "svelte-bootstrap-icons";

  const colors = [
    "#e83d84", // pink
    "#4fc4cf", // cyan
    "#f4a261", // warm orange
    "#7bed9f", // mint green
    "#a29bfe", // lavender
    "#ffd32a", // yellow
    "#ff6b6b", // coral red
    "#48dbfb", // sky blue
    "#ff9ff3", // light pink
    "#54a0ff", // blue
    "#5f27cd", // purple
    "#00d2d3", // teal
    "#ff9f43", // peach
    "#c8d6e5", // steel blue-grey
    "#01abc2", // deep cyan
    "#fd79a8", // rose
  ];

  let name = page.data.me?.display_name;
  let color = colors[Math.floor(Math.random() * colors.length)];

  let editorRef: Editor | null = null;

  function updatePreview() {
    if (editorRef) {
      editorRef.updatePreview();
    }
  }

  onMount(async () => {
    await init();
    updatePreview();
  });
</script>

<header
  class="flex items-center justify-between px-4 py-3 bg-[#252526] border-b border-gray-700 shadow-md z-10"
>
  <div class="flex items-center gap-2 mt-5 ml-auto">
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
  <aside
    class="w-12 bg-[#333333] border-r border-gray-700 flex flex-col items-center py-4 gap-4"
  >
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
