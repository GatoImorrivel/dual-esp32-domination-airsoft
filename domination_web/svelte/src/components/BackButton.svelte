<script lang="ts">
  import { navigate } from "../lib/router";

  export let label = "Voltar";
  /** Hash route, e.g. `/leaderboard` */
  export let to: string | undefined = undefined;
  /** Local back (e.g. mobile admin hub) */
  export let onBack: (() => void) | undefined = undefined;

  function handleClick() {
    if (onBack) {
      onBack();
    } else if (to) {
      navigate(to);
    }
  }
</script>

<button
  type="button"
  class="back-btn"
  aria-label={label}
  on:click={handleClick}
  disabled={!to && !onBack}
>
  <svg viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
    <path
      fill="currentColor"
      d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"
    />
  </svg>
  <span>{label}</span>
</button>

<style>
  .back-btn {
    -webkit-appearance: none;
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.5rem 0.75rem 0.5rem 0.5rem;
    border: 2px solid var(--color-border);
    border-radius: 999px;
    background: var(--color-bg);
    color: var(--color-fg);
    font-size: 0.9rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .back-btn:hover:not(:disabled) {
    background: var(--color-surface);
  }

  .back-btn:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .back-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
