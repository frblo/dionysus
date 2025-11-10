<script lang="ts">
  import { onMount } from "svelte";
  import { basicSetup } from "codemirror";
  import { EditorView, keymap } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";

  import * as Y from "yjs";
  import { WebsocketProvider } from "y-websocket";
  import { yCollab } from "y-codemirror.next";

  let {
    room = "demo-room-1",
    serverUrl = "ws://localhost:1234",
    user = {
      name: "Anonymous" + Math.floor(Math.random() * 100),
      color: "#30bced",
    },
  } = $props();

  let editorEl: HTMLDivElement;

  let view: EditorView | null = null;
  let provider: WebsocketProvider | null = null;

  onMount(() => {
    const ydoc = new Y.Doc();
    provider = new WebsocketProvider(serverUrl, room, ydoc);
    const ytext = ydoc.getText("codemirror");

    const undoManager = new Y.UndoManager(ytext);

    provider.awareness.setLocalStateField("user", user);

    view = new EditorView({
      parent: editorEl,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          yCollab(ytext, provider.awareness, { undoManager }),
          keymap.of([
            {
              key: "Mod-z",
              run: () => {
                undoManager.undo();
                return true;
              },
            },
            {
              key: "Mod-Shift-z",
              run: () => {
                undoManager.redo();
                return true;
              },
            },
          ]),
          basicSetup,
        ],
      }),
    });

    provider.on("status", (e) => {
      console.log(`[yws] ${e.status} ${serverUrl}/${room}`);
    });

    return () => {
      view?.destroy();
      provider?.destroy();
      ydoc.destroy();
    };
  });

  export function updateUser(user: { name: string; color: string }) {
    if (provider) {
      provider.awareness.setLocalStateField("user", user);
    }
  }
</script>

<div bind:this={editorEl} class="editor"></div>
