export function scrollPreviewToLine(
  cursorLine: number,
  previewEl: HTMLElement,
  scrollBehavior: ScrollLogicalPosition,
) {
  const elements = Array.from(
    previewEl.querySelectorAll<HTMLElement>("[data-start-line]"),
  );

  let best: HTMLElement | null = null;
  let bestLine = -1;

  for (const el of elements) {
    const startLine = parseInt(el.dataset.startLine!);
    if (startLine <= cursorLine && startLine > bestLine) {
      bestLine = startLine;
      best = el;
    }
  }

  best?.scrollIntoView({ behavior: "smooth", block: scrollBehavior });
}
