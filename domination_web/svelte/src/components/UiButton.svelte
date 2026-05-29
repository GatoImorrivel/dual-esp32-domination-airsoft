<script lang="ts">
  export let variant: "primary" | "secondary" = "secondary";
  export let block = false;
  export let pill = false;
  export let small = false;
  export let disabled = false;
</script>

<button
  type="button"
  class="ui-btn"
  class:primary={variant === "primary"}
  class:secondary={variant === "secondary"}
  class:block
  class:pill
  class:small
  {disabled}
  on:click
>
  <slot />
</button>

<style>
  .ui-btn {
    -webkit-appearance: none;
    appearance: none;
    box-sizing: border-box;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin: 0;
    padding: 14px 20px;
    border-radius: 14px;
    border: 2px solid var(--color-border);
    background-color: var(--color-bg);
    color: var(--color-fg);
    font-family: inherit;
    font-size: 1rem;
    font-weight: 600;
    line-height: 1.2;
    cursor: pointer;
    transition:
      border-color 0.15s ease,
      background-color 0.15s ease,
      filter 0.15s ease;
  }

  .ui-btn:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .ui-btn:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .ui-btn.primary {
    border-color: var(--color-primary);
    background-color: var(--color-primary);
    color: var(--color-primary-fg);
  }

  .ui-btn.primary:hover:not(:disabled) {
    filter: brightness(0.92);
  }

  .ui-btn.secondary:hover:not(:disabled) {
    border-color: var(--color-fg);
    background-color: var(--color-surface);
  }

  .ui-btn.block {
    width: 100%;
  }

  .ui-btn.pill {
    border-radius: 999px;
    padding: 10px 16px;
  }

  .ui-btn.small {
    padding: 10px 14px;
    font-size: 0.85rem;
    border-radius: 12px;
  }
</style>
