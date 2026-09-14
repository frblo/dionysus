import type { Me } from "$lib/api/generated/Me";

class SessionState {
  current = $state<Me | null>(null);
}

export const sessionState = new SessionState();
