<script lang="ts">
	let title = ""; // The title on the page
	let typeSpeed = 150; // The speed of the typewriter effect in ms

	// A list of titles of varying hilarity
	const titles: string[] = ["dionysus", "dionySUS", "Dionysus", "🍇", "d10ny5u5", "Διόνυσος", "bacchus", "dionȳsus", "dionysos", "dionysaur", "di-oh-NO-sus"];

	writeTitle(titles[0]); // Writes the title on page load
	setTimeout(titleUpdater, 5000 + Math.random() * 5000); // Change the title after 5-10 seconds

	/**
	 * Calls pickNewTitle() and then calls itself
	 * with a timer to continously change the title
	 * every 5-10 seconds
	*/
	function titleUpdater() {
		pickNewTitle();
		setTimeout(titleUpdater, 5000 + Math.random() * 5000);
	}

	/**
	 * Picks a new title from the array and calls
	 * writeTitle() to change the title on the page
	 * with a typewriter effect
	*/
	function pickNewTitle() {
		// Pick a random title from the array
		let nt = titles[Math.floor(Math.random() * titles.length)];
		writeTitle(nt);
	}

	/**
	 * Change the title on the page
	 * @param newTitle the new title
	 */
	function writeTitle(newTitle:string) {
		let i = 0; // Index for writeChar()
		deleteChar(titleLikeness());

		/**
		 * Calculates the likeness between the current title
		 * and the new title and returns the number of characters
		 * which are to be removed
		*/
		function titleLikeness() {
			let likeness = 0;
			for (let i = 0; i < title.length && i < newTitle.length; i++) {
				if (title[i] == newTitle[i])
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
		 * @param toRemove the number of characters to remove
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
</script>

<div>
	<div>
		<h1 class="title">
			{title}
		</h1>
	</div>
	<div>
		<a class="login-button" href="/login">
			Login
		</a>
	</div>
</div>

<style>
	.title {
		font-family: 'Courier New', Courier, monospace;
		font-size: 60px;
		text-align: center;
		margin-top: 240px;
		height: 20px;
	}

	.login-button {
		margin: 150px auto;
		display: block;
		width: 100px;
		padding: 5px;
		border-width: thick;
		border-style: double;
		border-radius: 5px;
		background-color: transparent;
		color: inherit;
		font-size: 20px;
		font-family: 'Courier New', Courier, monospace;
		text-align: center;
	}
</style>