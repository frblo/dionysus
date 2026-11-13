<script lang="ts">
  import { onMount } from "svelte";
  import { syntaxHighlighting } from "@codemirror/language";
  import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
  import { basicSetup } from "codemirror";
  import { EditorView } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";

  import { setVimEnabled } from "$lib/editor/vim-setup";
  import { userSettings } from "$lib/state/settings.svelte";
  import {
    createTrailingSpaces,
    setTrailingSpacesEnabled,
  } from "$lib/editor/trailing-spaces";

  let { content = "" } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;

  onMount(() => {
    const trailingSpaces = createTrailingSpaces();

    view = new EditorView({
      parent: editorEl,
      state: EditorState.create({
        doc: content,
        extensions: [
          fountain(),
          syntaxHighlighting(fountainHighlightStyle),
          basicDark,
          trailingSpaces,
          EditorView.lineWrapping,
          EditorView.contentAttributes.of({ spellcheck: "true" }),
          EditorState.readOnly.of(true),
          basicSetup,
        ],
      }),
    });

    setVimEnabled(view, userSettings.current.vimEnabled);
    setTrailingSpacesEnabled(
      view,
      userSettings.current.highlighTrailingSpacesEnabled,
    );

    return () => {
      view?.destroy();
      view = null;
    };
  });

  $effect(() => {
    if (!view) return;

    const nextContent = content;
    if (view.state.doc.toString() === nextContent) return;

    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: nextContent },
    });
    view.scrollDOM.scrollTop = 0;
    view.scrollDOM.scrollLeft = 0;
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
