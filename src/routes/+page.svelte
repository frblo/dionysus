<script lang="ts">
	let title = ""; // The title on the page
	let typeSpeed = 100; // The speed of the typewriter effect
	const titles: string[] = ["dionysus", "dionySUS", "Dionysus", "🍇", "d10ny5u5", "Διόνυσος", "bacchus", "chanel dio-nysus"];

	function handleClick() {
		// Pick a random title from the array
		let nt = titles[Math.floor(Math.random() * titles.length)];
		writeTitle(nt);
	}

	/**
	 * Change the title on the page
	 * @param newTitle the new title
	 */
	function writeTitle(newTitle:string) {
		let i = 0;
		deleteChar(titleLikeness());

		/**
		 * Calculates the likeness between the current title
		 * and the new title and returns the number of characters
		 * which are to be removed
		*/
		function titleLikeness() {
			let likeness = 0;
			for (let i = 0; i < title.length; i++) {
				if (i > newTitle.length)
					break;
				else if (title[i] == newTitle[i])
					likeness++;
				else
					break;
			}

			return title.length - likeness;
		}

		/**
		 * Deletes a character at a time and calls itself
		 * with a timer, then calls writeChar() to write
		 * out the new title
		*/
		function deleteChar(toRemove: number) {
			if (toRemove > 0) {
				title = title.slice(0, title.length - 1);
				setTimeout(() => deleteChar(toRemove - 1), typeSpeed);
			}
			else {
				i = title.length;
				writeChar();
			}
		}

		/**
		 * The typewriting effect itself. Writes out
		 * a character at a time and calls itself with
		 * a timer
		 */
		function writeChar() {
			if (i >= newTitle.length)
				return;
			
			title += newTitle[i]
			i++;
			
			if (i < newTitle.length)
				setTimeout(writeChar, typeSpeed);
		}
	}

	// Writes the title on load
	writeTitle(titles[0]);
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