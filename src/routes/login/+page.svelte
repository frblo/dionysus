<script>
    let username = "";
    let password = "";


</script>

<div>
    <h1 style=margin-top:200px>Login</h1>
    <form style=margin-top:20px>
        <input type="text" bind:value={username} placeholder="Username"/>
        <input type="password" bind:value={password} placeholder="Password"/>

        <button type="submit">
            Login
        </button>
    </form>
</div>

<style>
    h1 {
        text-align: center;
    }

    input {
        margin: 30px auto;
        display: block;
        background-color: transparent;
        color: inherit;
        padding: 8px 10px;
        border: solid 1px;
    }

    button {
        margin: 30px auto;
        display: block;
        background-color: transparent;
        color: inherit;
        padding: 8px 10px;
        border: 0cap;
    }
</style>