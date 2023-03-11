<!-- <h1>Welcome to SvelteKit</h1> -->
<!-- <p>Smaka bly, bitch!</p> -->

<script lang="ts">
	let title = ""; // The title on the page
	let typeSpeed = 100; // The speed of the typewriter effect

	function handleClick() {
		writeTitle("dionysus");
	}

	/**
	 * Change the title on the page
	 * @param newTitle the new title
	 */
	function writeTitle(newTitle:string) {
		let i = 0;
		writeChar();

		/**
		 * The typewriting effect itself. Writes out
		 * a character at a time and calls itself with
		 * a timer
		 */
		function writeChar() {
			title += newTitle[i]
			i++;
			
			if (i < newTitle.length)
				setTimeout(writeChar, typeSpeed);
		}
	}

	// Writes the title on load
	writeTitle("dionysus");
</script>

<div>
	<h1 style = "
		font-family: :'Courier New', Courier, monospace;
		text-align:center;
		"
	>
		{title}
	</h1>

	<button style="display: block; margin: auto;" on:click={handleClick}>
		write title
	</button>
</div>