<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  let title = $state("");
  let typeSpeed = 150;

  let password = $state("");
  let error = $state("");

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

  async function submit() {
    error = "";

    const next = page.url.searchParams.get("next") ?? "/";

    const r = await fetch("/auth/login", {
      method: "POST",
      credentials: "include",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ password }),
    });

    if (r.ok) {
      await goto(next);
    } else if (r.status === 401) {
      error = "Invalid password";
    } else {
      error = "Login failed";
    }
  }

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

  <form
    onsubmit={(e) => (e.preventDefault(), submit())}
    class="flex flex-col gap-6 w-full max-w-xs"
  >
    <div class="flex flex-col gap-2">
      <label
        for="password"
        class="text-[10px] uppercase font-bold text-gray-500 tracking-widest ml-1"
      >
        Password
      </label>
      <input
        type="password"
        id="password"
        autocomplete="current-password"
        bind:value={password}
        class="bg-[#252526] border border-gray-700 rounded px-4 py-3 text-white font-mono focus:outline-none focus:border-blue-500 transition-colors"
      />
    </div>

    <input type="submit" class="login-button" value="Login" />

    {#if error}
      <p class="text-red-400 text-sm">{error}</p>
    {/if}
  </form>
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
    justify-content: center;
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
