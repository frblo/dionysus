import { generate_html } from "$lib/converter/pkg/converter";

class PreviewState {
  html = $state("");
  targetLine = $state(0);
  scrollTick = $state(0);
  scrollBehavior: ScrollLogicalPosition = $state("nearest");

  generatePreview = (script: string) => {
    this.html = generate_html(script);
  };

  scrollToLine = (line: number) => {
    this.targetLine = line;
    this.scrollTick++;
    this.scrollBehavior = "nearest";
  };

  jumpToLine = (line: number) => {
    this.targetLine = line;
    this.scrollTick++;
    this.scrollBehavior = "start";
  };
}

export const preview = new PreviewState();
