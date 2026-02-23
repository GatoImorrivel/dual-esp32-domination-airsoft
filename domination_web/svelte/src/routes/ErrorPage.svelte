<script lang="ts">
  import { navigate } from "../lib/router";

  export let title: string = "Erro ao carregar";
  export let message: string = "Não foi possível carregar esta página.";
  export let retryPath: string | null = null;
  export let homePath: string = "/";

  function retry() {
    if (retryPath) navigate(retryPath);
  }

  function goHome() {
    navigate(homePath);
  }
</script>

<div class="container">
  <div class="icon">⚠️</div>

  <h1>{title}</h1>
  <p>{message}</p>

  <div class="actions">
    {#if retryPath}
      <button class="primary" on:click={retry}> Tentar novamente </button>
    {/if}

    <button class="secondary" on:click={goHome}> Voltar ao início </button>
  </div>
</div>

<style>
  .container {
    min-height: 100vh;
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
  }

  .icon {
    font-size: 48px;
  }

  h1 {
    font-size: 1.8rem;
  }

  p {
    color: #6b7280;
    font-size: 1rem;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 12px;
    width: 100%;
    margin-top: 12px;
  }

  button {
    padding: 14px;
    border-radius: 14px;
    font-weight: 600;
    border: none;
    cursor: pointer;
    font-size: 1rem;
  }

  .primary {
    background: #3b82f6;
    color: white;
  }

  .secondary {
    background: #e5e7eb;
    color: #111827;
  }

  button:hover {
    opacity: 0.9;
  }
</style>
