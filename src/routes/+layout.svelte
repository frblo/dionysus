<script lang="ts">
    import darkModeSun from "./../assets/darkmodesun.svg";
	import darkModeMoon from "./../assets/darkmodemoon.svg";
	let darkModeIcon = darkModeMoon;

    let darkMode = false; // Whether dark mode is enabled
	let backgroundColor = "#ffffff";
	let textColor = "#464646";
	/**
	 * Toggles dark mode
	 */
	function enableDarkMode() {
		const darkColor = "#464646"; // The dark mode background color
		const lightColor = "#ffffff"; // The light mode background color

		if (darkMode) {
			backgroundColor = lightColor;
			textColor = darkColor;
			darkModeIcon = darkModeMoon;
		}
		else {
			backgroundColor = darkColor;
			textColor = lightColor;
			darkModeIcon = darkModeSun;
		}
		
		document.body.style.backgroundColor = backgroundColor;
		document.body.style.color = textColor;
		darkMode = !darkMode;
	}
</script>

<body>
    <main>
        <slot></slot>
    </main>
    <img
		class="dark-mode-img" src={darkModeIcon}
		alt="darkmode sun" on:click={() => enableDarkMode()} on:keypress={() => enableDarkMode()}
	/>
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
		width: 25px;
		height: 25px;
	}
</style>