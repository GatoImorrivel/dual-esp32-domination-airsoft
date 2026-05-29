<script lang="ts">
  import { toast } from "../lib/toast";
  import { fly, fade } from "svelte/transition";
</script>

<div class="container">
  {#each $toast as t (t.id)}
    <div class="toast {t.type}" in:fly={{ y: -20, duration: 200 }} out:fade>
      <span>{t.message}</span>
      <button type="button" on:click={() => toast.remove(t.id)}>✕</button>
    </div>
  {/each}
</div>

<style>
  .container {
    position: fixed;
    top: 16px;
    left: 0;
    right: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
    z-index: 9999;
    pointer-events: none;
  }

  .toast {
    min-width: 280px;
    max-width: 90%;
    padding: 14px 16px;
    border-radius: 14px;
    font-weight: 500;
    display: flex;
    justify-content: space-between;
    align-items: center;
    pointer-events: auto;
    box-shadow: 0 6px 18px var(--color-overlay);
    font-family: system-ui, sans-serif;
    border: 1px solid var(--color-border);
  }

  .toast span {
    margin-right: 12px;
  }

  .toast button {
    border: none;
    background: transparent;
    font-size: 1rem;
    cursor: pointer;
    opacity: 0.7;
    color: inherit;
    padding: 0.25rem;
  }

  .toast button:hover {
    opacity: 1;
  }

  .info,
  .warn {
    background: var(--color-toast-info-bg);
    color: var(--color-toast-info-fg);
  }

  .error {
    background: var(--color-toast-error-bg);
    color: var(--color-toast-error-fg);
  }
</style>
