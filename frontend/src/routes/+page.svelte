<script lang="ts">
  import Editor from "$lib/editor/Editor.svelte";
  import { userSettings } from "$lib/state/settings.svelte";
  import { preview } from "$lib/state/preview.svelte";
  import init from "$lib/converter/pkg/converter";
  import { onMount } from "svelte";

  let name = $state("Anonymous" + Math.floor(Math.random() * 100));
  let color = $state("#e83d84");

  let editorRef: Editor | null = null;
  function applyUserUpdate() {
    editorRef?.updateUser({ name, color });
  }

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

<textarea bind:value={name}></textarea>
<textarea bind:value={color}></textarea>

<button
  class="px-4 py-2 rounded-lg border border-blue-600 text-blue-600 font-semibold hover:bg-blue-50 active:bg-blue-100 transition"
  onclick={applyUserUpdate}
>
  Update identity
</button>

<button
  class="px-4 py-2 rounded-lg border border-blue-600 text-blue-600 font-semibold hover:bg-blue-50 active:bg-blue-100 transition"
  onclick={() => (userSettings.vimEnabled = !userSettings.vimEnabled)}
>
  Toogle Vim
</button>

<button
  class="px-4 py-2 rounded-lg border border-blue-600 text-blue-600 font-semibold hover:bg-blue-50 active:bg-blue-100 transition"
  onclick={updatePreview}
>
  Preview
</button>

<main class="flex h-[calc(100vh-80px)] overflow-hidden">
  <section class="w-1/2 border-r overflow-auto bg-gray-50">
    <Editor bind:this={editorRef} user={{ name, color }} />
  </section>

  <section class="w-1/2 overflow-auto p-8 bg-white prose max-w-none">
    {#if preview.html}
      <iframe
        title="Screenplay Preview"
        srcdoc={preview.html}
        class="w-full h-full border-none"
      ></iframe>
    {:else}
      <div class="p-8 text-gray-400 italic">
        No content found. Try writing something and then click "Preview", press
        "CTRL+s" or run command ":w", to see preview...
      </div>
    {/if}
  </section>
</main>
