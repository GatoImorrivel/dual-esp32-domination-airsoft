<script lang="ts">
  export let ariaLabel: string;
  export let title: string | undefined = undefined;
  export let disabled = false;
</script>

<button
  type="button"
  class="ui-icon-btn"
  aria-label={ariaLabel}
  {title}
  {disabled}
  on:click
>
  <slot />
</button>

<style>
  .ui-icon-btn {
    -webkit-appearance: none;
    appearance: none;
    box-sizing: border-box;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 2.75rem;
    height: 2.75rem;
    margin: 0;
    padding: 0;
    border: 2px solid var(--color-border);
    border-radius: 50%;
    background-color: var(--color-bg);
    color: var(--color-fg);
    font-family: inherit;
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      border-color 0.15s ease;
  }

  .ui-icon-btn:hover:not(:disabled) {
    border-color: var(--color-fg);
    background-color: var(--color-surface);
  }

  .ui-icon-btn:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .ui-icon-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
