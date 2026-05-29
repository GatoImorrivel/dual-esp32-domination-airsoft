<script lang="ts">
  import BackButton from "../components/BackButton.svelte";
  import UiButton from "../components/UiButton.svelte";
  import { navigate } from "../lib/router";

  export let title: string = "Erro ao carregar";
  export let message: string = "Não foi possível carregar esta página.";
  export let retryPath: string | null = null;
  export let homePath: string = "/";

  function retry() {
    if (retryPath) navigate(retryPath);
  }
</script>

<div class="container">
  <div class="icon" aria-hidden="true">!</div>

  <h1>{title}</h1>
  <p>{message}</p>

  <div class="actions">
    {#if retryPath}
      <UiButton variant="primary" block on:click={retry}>Tentar novamente</UiButton>
    {/if}

    <BackButton label="Voltar ao início" to={homePath} />
  </div>
</div>

<style>
  .container {
    min-height: 100dvh;
    max-width: 500px;
    margin: 0 auto;
    padding: 32px 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 20px;
    text-align: center;
    font-family: system-ui, sans-serif;
    color: var(--color-fg);
    background: var(--color-bg);
  }

  .icon {
    font-size: 3rem;
    font-weight: 800;
    width: 4rem;
    height: 4rem;
    line-height: 4rem;
    border-radius: 50%;
    border: 2px solid var(--color-border);
  }

  h1 {
    font-size: 1.8rem;
    margin: 0;
  }

  p {
    color: var(--color-muted);
    font-size: 1rem;
    margin: 0;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
    max-width: 280px;
    margin-top: 12px;
    align-items: stretch;
  }

  .actions :global(.back-btn) {
    justify-content: center;
    width: 100%;
  }
</style>
