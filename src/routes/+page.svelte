<!-- <h1>Welcome to SvelteKit</h1> -->
<!-- <p>Smaka bly, bitch!</p> -->

<script lang="ts">
	let title = "";
	let typeSpeed = 100;
	title = "";


	function handleClick() {
		writeTitle("dionysus");
	}

	function writeTitle(newTitle:string) {
		let i = 0;
		writeChar();

		function writeChar() {
			title += newTitle[i]
			i++;
			
			if (i < newTitle.length)
				setTimeout(writeChar, typeSpeed);
		}
	}
</script>

<div>
	<h1 style = "
		font-family: :'Courier New', Courier, monospace;
		text-align:center;
		"
	>
		{title}
	</h1>
	<button style="margin: 0 50%;" on:click={handleClick}>
		write title
	</button>
</div>

<!-- <style>
	body {
		background-color:#707070;
	}
</style> -->