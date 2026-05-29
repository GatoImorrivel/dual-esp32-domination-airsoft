<script lang="ts">
  export let disabled = false;
</script>

<button type="button" class="ui-card" {disabled} on:click>
  <span class="title"><slot name="title" /></span>
  {#if $$slots.desc}
    <span class="desc"><slot name="desc" /></span>
  {/if}
</button>

<style>
  .ui-card {
    -webkit-appearance: none;
    appearance: none;
    box-sizing: border-box;
    display: grid;
    gap: 6px;
    width: 100%;
    margin: 0;
    padding: 16px 18px;
    border-radius: 14px;
    border: 2px solid var(--color-border);
    background-color: var(--color-bg);
    color: var(--color-fg);
    font-family: inherit;
    text-align: left;
    cursor: pointer;
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease;
  }

  .ui-card:hover:not(:disabled) {
    border-color: var(--color-fg);
    background-color: var(--color-surface);
  }

  .ui-card:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .ui-card:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .title {
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--color-fg);
  }

  .desc {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--color-muted);
    line-height: 1.35;
  }
</style>
