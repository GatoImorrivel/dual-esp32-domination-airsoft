<script lang="ts">
  import UiButton from "../UiButton.svelte";
  import type { AudioSink, BtSinksResponse } from "../../lib/bt";

  export let btState: BtSinksResponse | null = null;
  export let btLoading = false;
  export let btActionAddress: string | null = null;
  export let onScan: () => void;
  export let onPair: (sink: AudioSink) => void;
  export let onUnpair: () => void;

  function displayName(sink: AudioSink): string {
    return sink.name ?? sink.address;
  }

  function isPaired(sink: AudioSink): boolean {
    return (
      btState?.paired?.address.toLowerCase() === sink.address.toLowerCase()
    );
  }
</script>

<section class="panel-section">
  <h2>Áudio Bluetooth</h2>
  <p class="section-hint">Selecione o alto-falante para anunciar eventos da partida.</p>

  <UiButton variant="secondary" block disabled={btLoading} on:click={onScan}>
    {btLoading ? "Buscando…" : "Buscar dispositivos"}
  </UiButton>

  {#if btState?.paired}
    <div class="paired-banner" class:disconnected={btState.connected === false}>
      <span>Emparelhado:</span>
      <strong>{displayName(btState.paired)}</strong>
      {#if btState.connected === false}
        <span class="link-warn">Sem conexão de áudio</span>
      {:else if btState.connected === true}
        <span class="link-ok">Conectado</span>
      {/if}
      <UiButton
        variant="secondary"
        small
        pill
        disabled={btActionAddress !== null}
        on:click={onUnpair}
      >
        {btActionAddress === "unpair" ? "…" : "Desemparelhar"}
      </UiButton>
    </div>
  {:else}
    <p class="empty">Nenhum alto-falante emparelhado.</p>
  {/if}

  {#if btState && btState.discovered.length > 0}
    <ul class="sink-list">
      {#each btState.discovered as sink (sink.address)}
        <li class:active={isPaired(sink)}>
          <div class="sink-info">
            <strong>{displayName(sink)}</strong>
            <span class="mac">{sink.address}</span>
          </div>
          {#if isPaired(sink)}
            <span class="tag" class:tag-off={btState?.connected === false}>
              {btState?.connected === false ? "Desconectado" : "Ativo"}
            </span>
          {:else}
            <UiButton
              variant="secondary"
              small
              disabled={btActionAddress !== null}
              on:click={() => onPair(sink)}
            >
              {btActionAddress === sink.address ? "…" : "Emparelhar"}
            </UiButton>
          {/if}
        </li>
      {/each}
    </ul>
  {:else if !btLoading}
    <p class="empty">Nenhum dispositivo encontrado. Toque em buscar.</p>
  {/if}
</section>

<style>
  .panel-section {
    margin-bottom: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .panel-section :global(.ui-btn.block) {
    margin-bottom: 16px;
  }

  h2 {
    margin: 0 0 8px;
    font-size: 1.15rem;
    color: var(--color-fg);
  }

  .section-hint {
    margin: 0 0 16px;
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  .paired-banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-radius: 12px;
    border: 2px solid var(--color-border);
    background: var(--color-surface);
    margin-bottom: 12px;
    font-size: 0.9rem;
  }

  .paired-banner :global(.ui-btn) {
    margin-left: auto;
  }

  .paired-banner.disconnected {
    border-color: color-mix(in srgb, var(--color-fg) 30%, transparent);
  }

  .link-warn {
    font-size: 0.8rem;
    color: #c45c26;
    font-weight: 600;
  }

  .link-ok {
    font-size: 0.8rem;
    color: #2a7a3b;
    font-weight: 600;
  }

  .tag-off {
    opacity: 0.85;
  }

  .empty {
    font-size: 0.9rem;
    color: var(--color-muted);
    margin: 8px 0;
  }

  .sink-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 8px;
  }

  .sink-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px;
    border-radius: 12px;
    border: 2px solid var(--color-border);
    background: var(--color-bg);
  }

  .sink-list li.active {
    border-color: var(--color-fg);
  }

  .sink-info {
    display: grid;
    gap: 2px;
    min-width: 0;
  }

  .sink-info strong {
    font-size: 0.95rem;
  }

  .mac {
    font-size: 0.75rem;
    color: var(--color-muted);
    font-family: monospace;
  }

  .tag {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--color-fg);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
</style>
