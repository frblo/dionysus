<script lang="ts">
  import { onMount } from "svelte";
  import { unifiedMergeView } from "@codemirror/merge";
  import { syntaxHighlighting } from "@codemirror/language";
  import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
  import { basicSetup } from "codemirror";
  import { EditorView } from "@codemirror/view";
  import { EditorState, type Extension } from "@codemirror/state";
  import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";

  import { setVimEnabled } from "$lib/editor/vim-setup";
  import { userSettings } from "$lib/state/settings.svelte";
  import {
    createTrailingSpaces,
    setTrailingSpacesEnabled,
  } from "$lib/editor/trailing-spaces";

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

  function createView(currentContent: string, previous: string | null) {
    view?.destroy();

    const trailingSpaces = createTrailingSpaces();
    const extensions: Extension[] = [
      fountain(),
      syntaxHighlighting(fountainHighlightStyle),
      basicDark,
      trailingSpaces,
      EditorView.lineWrapping,
      EditorView.contentAttributes.of({ spellcheck: "true" }),
      EditorState.readOnly.of(true),
      basicSetup,
    ];

    if (previous !== null) {
      extensions.push(
        unifiedMergeView({
          original: previous,
          mergeControls: false,
        }),
      );
    }

    view = new EditorView({
      parent: editorEl,
      state: EditorState.create({
        doc: currentContent,
        extensions,
      }),
    });

    setVimEnabled(view, userSettings.current.vimEnabled);
    setTrailingSpacesEnabled(
      view,
      userSettings.current.highlighTrailingSpacesEnabled,
    );

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
    setVimEnabled(view, userSettings.current.vimEnabled);
    setTrailingSpacesEnabled(
      view,
      userSettings.current.highlighTrailingSpacesEnabled,
    );
  });

  export function getContent() {
    return view ? view.state.doc.toString() : "";
  }
</script>

<div bind:this={editorEl} class="editor"></div>
