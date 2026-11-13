<script lang="ts">
  import { onMount } from "svelte";
  import { syntaxHighlighting } from "@codemirror/language";
  import { basicDark } from "@fsegurai/codemirror-theme-basic-dark";
  import { basicSetup } from "codemirror";
  import { EditorView } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { fountain, fountainHighlightStyle } from "$lib/fountain-highlight";

  import * as Y from "yjs";

  import { setVimEnabled } from "$lib/editor/vim-setup";
  import { userSettings } from "$lib/state/settings.svelte";
  import { sceneScanner } from "$lib/state/scenes.svelte";
  import {
    createTrailingSpaces,
    setTrailingSpacesEnabled,
  } from "$lib/editor/trailing-spaces";

  let { room = "", sourceContent = "" } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;
  let undoManager: Y.UndoManager | null = null;

  onMount(() => {
    const ydoc = new Y.Doc();
    const ytext = ydoc.getText("codemirror");

    undoManager = new Y.UndoManager(ytext);

    const trailingSpaces = createTrailingSpaces();

    view = new EditorView({
      parent: editorEl,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          fountain(),
          syntaxHighlighting(fountainHighlightStyle),
          basicDark,
          trailingSpaces,
          EditorView.lineWrapping,
          EditorView.contentAttributes.of({ spellcheck: "true" }),
          EditorState.readOnly.of(true),
          sceneScanner,
          basicSetup,
        ],
      }),
    });

    return () => {
      view?.destroy();
      ydoc.destroy();
    };
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
