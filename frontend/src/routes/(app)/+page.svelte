<script lang="ts">
  import Header from "$lib/common/Header.svelte";
  import Sidebar from "$lib/common/Sidebar.svelte";
  import { FileEarmarkPlus } from "svelte-bootstrap-icons";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
  console.log(data);
  console.log("hl");
</script>

<Header title="Screenplays"></Header>

<div class="flex flex-1 h-[calc(100vh-64px)] overflow-hidden">
  <Sidebar>
    <button
      class="p-2 text-gray-400 hover:text-white transition-colors"
      title="Create new screenplay"
      onclick={() => console.log("hello")}
    >
      <FileEarmarkPlus />
    </button>
  </Sidebar>

  <main class="flex flex-1 overflow-hidden bg-[#1e1e1e]"></main>
</div>
