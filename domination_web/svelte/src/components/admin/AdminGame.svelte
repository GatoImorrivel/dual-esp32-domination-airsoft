<script lang="ts">
  import LinkedTeamTimes from "../LinkedTeamTimes.svelte";
  import UiButton from "../UiButton.svelte";

  export let progress: { is_active?: boolean } | null = null;
  export let redSecs = 10;
  export let blueSecs = 10;
  export let timesLinked = true;
  export let onStart: () => void;
  export let onStop: () => void;
  export let onSave: () => void;
</script>

<section class="panel-section">
  <h2>Partida</h2>

  {#if progress}
    <div class="status">
      <span class="badge">
        {progress.is_active ? "Partida em andamento" : "Partida parada"}
      </span>
    </div>

    <div class="controls">
      <UiButton
        variant="primary"
        block
        disabled={progress.is_active}
        on:click={onStart}
      >
        Iniciar
      </UiButton>
      <UiButton
        variant="secondary"
        block
        disabled={!progress.is_active}
        on:click={onStop}
      >
        Parar
      </UiButton>
    </div>
  {/if}

  <LinkedTeamTimes bind:redSecs bind:blueSecs bind:timesLinked />

  <UiButton variant="primary" block on:click={onSave}>Salvar configuração</UiButton>
</section>

<style>
  .panel-section {
    margin-bottom: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  h2 {
    margin: 0;
    font-size: 1.15rem;
    color: var(--color-fg);
  }

  .status {
    margin: 0;
  }

  .badge {
    display: inline-block;
    padding: 8px 14px;
    border-radius: 999px;
    font-size: 0.9rem;
    font-weight: 600;
    border: 2px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-fg);
  }

  .controls {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
</style>
