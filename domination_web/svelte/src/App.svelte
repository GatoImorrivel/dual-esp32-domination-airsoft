<script lang="ts">
  import { get } from "./lib/http";
  import { currentPath, getComponent, initRouter } from "./lib/router";
  import ToastContainer from "./components/ToastContainer.svelte";
  import Loading from "./routes/Loading.svelte";
  import ErrorPage from "./routes/ErrorPage.svelte";
  import PageNotFound from "./routes/PageNotFound.svelte";

  initRouter({
    "/": async () => {
      const currentState = await get<"Setup" | "Running">("/app/status");
      return currentState === "Running"
        ? (await import("./routes/Leaderboard.svelte")).default
        : (await import("./routes/Config.svelte")).default;
    },
    "/leaderboard": async () => {
      const currentState = await get<"Setup" | "Running">("/app/status");
      return currentState === "Setup"
        ? (await import("./routes/Config.svelte")).default
        : (await import("./routes/Leaderboard.svelte")).default;
    },
    "/admin": async () => {
      const currentState = await get<"Setup" | "Running">("/app/status");
      return currentState === "Setup"
        ? (await import("./routes/Config.svelte")).default
        : (await import("./routes/AdminPanel.svelte")).default;
    },
  });

  $: ComponentPromise = getComponent($currentPath);
</script>

<ToastContainer />

{#if ComponentPromise}
  {#await ComponentPromise}
    <svelte:component this={Loading} />
  {:then component}
    <svelte:component this={component} />
  {:catch}
    <svelte:component this={ErrorPage} />
  {/await}
{:else}
  <svelte:component this={PageNotFound} />
{/if}
