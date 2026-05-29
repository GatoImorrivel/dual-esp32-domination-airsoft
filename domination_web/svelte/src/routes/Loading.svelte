<script lang="ts">
  import Spinner from "../components/Spinner.svelte";

  export let message: string = "Carregando...";
</script>

<div class="container">
  <Spinner size={64} thickness={5} />

  <p>{message}</p>
</div>

<style>
  .container {
    min-height: 100dvh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
    padding: 24px;
    font-family: system-ui, sans-serif;
    text-align: center;
    color: var(--color-fg);
    background: var(--color-bg);
  }

  p {
    font-size: 1.1rem;
    color: var(--color-muted);
  }
</style>
