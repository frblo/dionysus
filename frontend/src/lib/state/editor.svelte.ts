class EditorState {
  targetLine = $state(1);
  scrollTick = $state(0);

  scrollToLine(line: number) {
    this.targetLine = line;
    this.scrollTick++;
  }
}

export const editor = new EditorState();
