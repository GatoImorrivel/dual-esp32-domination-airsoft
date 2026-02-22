<script lang="ts">
  import { getComponent, initRouter } from "./router";
  import AdminPanel from "./routes/AdminPanel.svelte";
  import Config from "./routes/Config.svelte";
  import Leaderboard from "./routes/Leaderboard.svelte";
  import PageNotFound from "./routes/PageNotFound.svelte";

  initRouter({
    "/": () => Config,
    "/leaderboard": () => Leaderboard,
    "/admin": () => AdminPanel,
  });

  $: Component = getComponent();
</script>

{#if Component}
  <svelte:component this={Component} />
{:else}
  <svelte:component this={PageNotFound} />
{/if}
