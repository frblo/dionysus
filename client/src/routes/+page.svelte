<script lang="ts">
  import Editor from "./Editor.svelte";
  import { userSettings } from "$lib/state/settings.svelte";
  import init, { preview_play } from "$lib/converter/pkg/converter";
  import { onMount } from "svelte";

  let name = $state("Anonymous" + Math.floor(Math.random() * 100));
  let color = $state("#e83d84");

  let editorRef: Editor | null = null;
  function applyUserUpdate() {
    editorRef?.updateUser({ name, color });
  }

  function showPreview() {
    if (editorRef) {
      const source = editorRef?.getContent();
      const screenplay = preview_play(source);
      console.log(screenplay);
    }
  }

  onMount(async () => {
    await init();
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
  onclick={showPreview}
>
  Preview
</button>

<Editor bind:this={editorRef} user={{ name, color }} />
