<script>
    let username = "";
    let password = "";


</script>

<h1 style=margin-top:20%>Login</h1>
<form style=margin-top:20px>
    <input type="text" bind:value={username} placeholder="Username"/>
    <input type="password" bind:value={password} placeholder="Password"/>

    <button type="submit">
        Sign in
    </button>
</form>
    
<style>
    h1 {
        text-align: center;
    }

    input {
        margin: 20px auto;
        display: block;
        background-color: transparent;
        color: inherit;
        padding: 8px 10px;
        border: solid 1px;
        font: inherit;
    }

    button {
        margin: 0 auto;
        display: block;
        background-color: transparent;
        color: inherit;
        padding: 8px 10px;
        border: solid 1px;
        font: inherit;
    }
</style>