<script lang="ts">
	// Import the sun and moon icons for darkmode button
	import darkModeSun from "./../assets/darkmodesun.svg";
	import darkModeMoon from "./../assets/darkmodemoon.svg";
	let darkModeIcon = darkModeSun;

	let title = ""; // The title on the page
	let typeSpeed = 150; // The speed of the typewriter effect in ms

	// A list of titles of varying hilarity
	const titles: string[] = ["dionysus", "dionySUS", "Dionysus", "🍇", "d10ny5u5", "Διόνυσος", "bacchus", "dionȳsus", "dionysos", "dionysaur", "di-oh-NO-sus"];

	writeTitle(titles[0]); // Writes the title on page load
	setTimeout(titleUpdater, 5000 + Math.random() * 5000); // Change the title after 5-10 seconds

	let darkMode = false; // Whether dark mode is enabled
	/**
	 * Toggles dark mode
	 */
	function enableDarkMode() {
		const darkColor = "#464646"; // The dark mode background color
		const lightColor = "#fff"; // The light mode background color

		if (darkMode) {
			document.body.style.backgroundColor = lightColor;
			document.body.style.color = darkColor;
			darkModeIcon = darkModeMoon;
		}
		else {
			document.body.style.backgroundColor = darkColor;
			document.body.style.color = lightColor;
			darkModeIcon = darkModeSun;
		}

		darkMode = !darkMode;
	}

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

<body>
	<main>
		<div>
			<h1 style = "
				font-family: :'Courier New', Courier, monospace;
				text-align:center;
				margin-top: 200px;
				height: 20px;
			">
				{title}
			</h1>
		</div>
	</main>

	<img class="dark-mode-img" src={darkModeIcon} alt="darkmode sun" on:click={() => enableDarkMode()}/>
</body>

<style>
	body {
		margin: 0;
		padding: 0;
		height: 100vh;
		overflow: hidden;
	}
	
	.dark-mode-img {
		position: fixed;
		bottom: 20px;
		left: 20px;
		display: block;
		width: 50px;
		height: 50px;
	}
</style>