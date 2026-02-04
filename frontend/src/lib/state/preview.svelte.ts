import { preview_play } from "$lib/converter/pkg/converter";

export const preview = $state({
  html: ""
});

export function generatePreview(script: string) {
  if (script == "") {
    preview.html = "";
    return;
  }
  preview.html = preview_play(script);
}
