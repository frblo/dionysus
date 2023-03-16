<script>
    import {page} from '$app/stores';
</script>

{#if $page.error}
    <h1>{$page.status}: {$page.error.message}</h1>
{:else}
    <h1>An error has occurred</h1>
{/if}

<style>
    h1 {
        margin-top: 250px;
       text-align: center;
    }
</style>