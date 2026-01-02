import { preview_play } from "$lib/converter/pkg/converter";

export const preview = $state({
  html: ""
});

export function generatePreview(script: string) {
  preview.html = preview_play(script);
}
