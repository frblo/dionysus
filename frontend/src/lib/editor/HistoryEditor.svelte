<script lang="ts">
  import { onMount } from "svelte";
  import { unifiedMergeView } from "@codemirror/merge";
  import type { EditorView } from "@codemirror/view";
  import type { Extension } from "@codemirror/state";

  import { createBasicVim } from "$lib/editor/vim-setup";
  import {
    applyEditorSettings,
    createEditorView,
    getEditorContent,
  } from "$lib/editor/editor-setup";

  let {
    content = "",
    previousContent = null,
  }: {
    content?: string;
    previousContent?: string | null;
  } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;
  let renderedContent: string | null = null;
  let renderedPreviousContent: string | null = null;

  const vimExt = createBasicVim();

  function createView(currentContent: string, previous: string | null) {
    view?.destroy();

    const extensions: Extension[] = [vimExt];

    if (previous !== null) {
      extensions.push(
        unifiedMergeView({
          original: previous,
          mergeControls: false,
        }),
      );
    }

    view = createEditorView({
      parent: editorEl,
      doc: currentContent,
      readOnly: true,
      extensions,
    });

    renderedContent = currentContent;
    renderedPreviousContent = previous;
  }

  onMount(() => {
    createView(content, previousContent);

    return () => {
      view?.destroy();
      view = null;
    };
  });

  $effect(() => {
    if (!view) return;
    if (
      content === renderedContent &&
      previousContent === renderedPreviousContent
    ) {
      return;
    }

    createView(content, previousContent);
  });

  $effect(() => {
    if (!view) return;
    applyEditorSettings(view);
  });

  export function getContent() {
    return getEditorContent(view);
  }
</script>

<div bind:this={editorEl} class="editor"></div>
