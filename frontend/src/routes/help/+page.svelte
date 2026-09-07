<script lang="ts">
  import Header from "$lib/common/Header.svelte";
</script>

<svelte:head>
  <title>Help - Dionysus</title>
</svelte:head>

<Header title="Help"></Header>

<main class="flex-1 overflow-auto bg-[#1e1e1e] px-6 py-10 flex justify-center">
  <div class="max-w-2xl w-full space-y-10">
    <section>
      <h2
        class="text-lg font-mono font-bold text-gray-100 mb-2 border-b border-gray-700 pb-1"
      >
        Getting Started
      </h2>
      <p class="text-sm text-gray-400 leading-relaxed font-mono">
        Dionysus is an open source real-time collaborative screenplay editor
        built on the <a href="https://fountain.io">Fountain</a>
        format, using the purpose-built Fountain parser and compiler
        <a href="https://github.com/frblo/rustwell">Rustwell</a>.
        <br />
        To get started, simply create a new screenplay and start writing, given you
        possess the right permissions - otherwise contact the admin(s) responsible
        for your instance.
      </p>
    </section>

    <section>
      <h2
        class="text-lg font-mono font-bold text-gray-100 mb-2 border-b border-gray-700 pb-1"
      >
        Issues
      </h2>
      <p class="text-sm text-gray-400 leading-relaxed font-mono mb-2">
        If you experience any issues, please report them via one of the
        following links:
      </p>
      <ul class="text-sm text-gray-400 font-mono space-y-2 pl-4 list-disc">
        <li>
          <a href="https://github.com/frblo/rustwell/issues">Rustwell</a>
          <span>
            - issues related to the export features, e.g. the PDF documents
            generated.</span
          >
        </li>
        <li>
          <a href="https://github.com/frblo/dionysus/issues">Dionysus</a>
          <span> - issues related to any parts of the web application.</span>
        </li>
      </ul>
    </section>

    <section>
      <h2
        class="text-lg font-mono font-bold text-gray-100 mb-2 border-b border-gray-700 pb-1"
      >
        Fountain Syntax
      </h2>
      <p class="text-sm text-gray-400 leading-relaxed font-mono">
        Screenplays are written in Fountain, a simple plain-text markup
        language. The full syntax documentation can be found on <a
          href="https://fountain.io/syntax">Fountain's own website</a
        >, but below is a very simple example to illustrate how easy it is to
        start writing.
      </p>
    </section>

    <div class="space-y-6">
      <div>
        <h3 class="text-sm font-mono font-semibold text-gray-300 mb-2">
          Example screenplay
        </h3>
        <pre
          class="bg-[#252526] border border-gray-700 rounded-md p-4 text-sm text-gray-200 font-mono overflow-x-auto whitespace-pre">
Title: STAR WARfareS
Credit: Written by
Author: the Dionysus devs
Source: Based on nothing. It's **wholly** original.
Contact: Don't sue us, please

INT. PRESIDENT FRANK'S EVIL OFFICE

President Frank is sitting on a swivel chair in front of a large window, looking at space (there is a lot of it). Darth Cyborg is standing around.

PRESIDENT FRANK
You know, being the *parliament* is quite exhausting.

President Frank sighs.

PRESIDENT FRANK
I need a vacation. I hear Alderaan is nice this time of year.

DARTH CYBORG
(hesitant)
Uhm... I think Alderaan is quite...
*gone* this time of year.

PRESIDENT FRANK
Oh, yeah. Cloud planet it is then.

EXT. CLOUD PLANET

Darth Cyborg is standing around. Luke Spacewalker is hanging on a 5G antenna above a huge OSHA violation.

LUKE SPACEWALKER
Get away!

DARTH CYBORG
Luke! Get up from there or you'll hurt yourself.

LUKE SPACEWALKER
Nooo!

DARTH CYBORG
Let's just go home and talk about this. I'm still your dad, remember?

LUKE SPACEWALKER
Oh, yeah. Home it is then.

> ROLL CREDITS
</pre>
      </div>
    </div>
  </div>
</main>

<style>
  a {
    color: #4daafc;
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }
</style>
