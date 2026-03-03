<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  let title = $state("");
  let typeSpeed = 150;

  let error = $state("");
  let providerIds = $state<string[]>([]);
  let loading = $state(true);

  const titles: string[] = [
    "dionysus",
    "dionySUS",
    "Dionysus",
    "d10ny5u5",
    "Διόνυσος",
    "bacchus",
    "dionȳsus",
    "dionysos",
    "dionysaur",
    "di-oh-NO-sus",
  ];

  function titleUpdater() {
    pickNewTitle();
    setTimeout(titleUpdater, 5000 + Math.random() * 5000);
  }

  function pickNewTitle() {
    let nt = titles[Math.floor(Math.random() * titles.length)];
    writeTitle(nt);
  }

  function writeTitle(newTitle: string) {
    let i = 0;
    deleteChar(titleLikeness());

    function titleLikeness() {
      let likeness = 0;
      for (let i = 0; i < title.length && i < newTitle.length; i++) {
        if (title[i] == newTitle[i]) likeness++;
        else break;
      }
      return title.length - likeness;
    }

    function deleteChar(toRemove: number) {
      if (toRemove > 0) {
        title = title.slice(0, title.length - 1);
        setTimeout(() => deleteChar(toRemove - 1), typeSpeed);
      } else {
        i = title.length;
        writeChar();
      }
    }

    function writeChar() {
      if (i >= newTitle.length) return;
      title += newTitle[i];
      i++;
      if (i < newTitle.length) setTimeout(writeChar, typeSpeed);
    }
  }

  async function loadProviders() {
    try {
      const res = await fetch("/auth/providers");
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      providerIds = (await res.json()).providers as string[];
    } catch (err) {
      error = err instanceof Error ? err.message : "Unknown error";
    } finally {
      loading = false;
    }
  }

  function login(provider: string) {
    const next = encodeURIComponent(page.url.searchParams.get("next") ?? "/");
    window.location.href = `/auth/login?provider=${provider}&next=${next}`;
  }

  loadProviders();
  writeTitle(titles[0]);
  setTimeout(titleUpdater, 5000 + Math.random() * 5000);
</script>

<svelte:head>
  <title>Login - Dionysus</title>
</svelte:head>

<main
  class="flex flex-col items-center justify-center min-h-screen bg-[#1e1e1e] text-gray-100 p-4"
>
  <div class="text-center mb-12">
    <h1 class="title-text">
      {title}<span class="cursor">_</span>
    </h1>
  </div>

  {#if loading}
    <p>Loading...</p>
  {:else if error}
    <p>Error: {error}</p>
  {:else}
    {#each providerIds as id}
      <button onclick={() => login(id)} class="login-button">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
          <polyline points="10 17 15 12 10 7" />
          <line x1="15" y1="12" x2="3" y2="12" />
        </svg>
        {id}
      </button>
    {/each}
  {/if}
</main>

<style>
  .title-text {
    font-family: "Courier New", Courier, monospace;
    font-size: clamp(2rem, 10vw, 60px);
    font-weight: bold;
    letter-spacing: -0.05em;
    height: 1.2em;
    display: flex;
    align-items: center;
    justify-content: flex-start;
  }

  .login-button {
    font-family: "Courier New", Courier, monospace;
    font-size: 18px;
    padding: 12px 32px;
    border: 2px solid #4a4a4a;
    border-radius: 4px;
    color: #a0a0a0;
    transition: all 0.2s ease;
    text-decoration: none;
    letter-spacing: 2px;
    width: 240px;
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }

  .login-button:hover {
    background-color: #333333;
    color: #ffffff;
    border-color: #a0a0a0;
  }

  .cursor {
    animation: blink 1s step-end infinite;
    margin-left: 4px;
  }

  @keyframes blink {
    from,
    to {
      opacity: 1;
    }
    50% {
      opacity: 0;
    }
  }
</style>
