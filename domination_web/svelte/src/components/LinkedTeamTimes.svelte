<script lang="ts">
  import DurationWheelPicker from "./DurationWheelPicker.svelte";
  import { onLinkEnabled, syncTeamTimes } from "../lib/linkedTimes";

  export let redSecs: number;
  export let blueSecs: number;
  export let timesLinked: boolean;

  function toggleLink() {
    if (!timesLinked) {
      const synced = onLinkEnabled(redSecs, blueSecs);
      redSecs = synced.redSecs;
      blueSecs = synced.blueSecs;
    }
    timesLinked = !timesLinked;
  }

  function onRedChange(event: CustomEvent<number>) {
    const next = syncTeamTimes(timesLinked, "red", redSecs, blueSecs, event.detail);
    redSecs = next.redSecs;
    blueSecs = next.blueSecs;
  }

  function onBlueChange(event: CustomEvent<number>) {
    const next = syncTeamTimes(timesLinked, "blue", redSecs, blueSecs, event.detail);
    redSecs = next.redSecs;
    blueSecs = next.blueSecs;
  }
</script>

<div class="linked-times">
  <div class="header">
    <h3>Tempo para dominar</h3>
    <button
      type="button"
      class="link-toggle"
      class:linked={timesLinked}
      on:click={toggleLink}
      title={timesLinked ? "Destravar tempos" : "Travar tempos iguais"}
      aria-label={timesLinked ? "Destravar tempos" : "Travar tempos iguais"}
      aria-pressed={timesLinked}
    >
      {#if timesLinked}
        <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
          <path
            fill="currentColor"
            d="M18 8h-1V6a5 5 0 0 0-10 0v2H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V10a2 2 0 0 0-2-2zm-6 9a2 2 0 1 1 0-4 2 2 0 0 1 0 4zm3-9H9V6a3 3 0 0 1 6 0v2z"
          />
        </svg>
        <span>Travado</span>
      {:else}
        <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
          <path
            fill="currentColor"
            d="M17 8h-1V6a5 5 0 0 0-9.9-1H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2.1A5.002 5.002 0 0 0 17 8zm-5 9a2 2 0 1 1 0-4 2 2 0 0 1 0 4zm3-9H9V6a3 3 0 0 1 5.9.8H15V8z"
          />
        </svg>
        <span>Livre</span>
      {/if}
    </button>
  </div>

  <div class="form">
    <DurationWheelPicker
      bind:secs={redSecs}
      label="Vermelho"
      on:change={onRedChange}
    />
    <DurationWheelPicker
      bind:secs={blueSecs}
      label="Azul"
      disabled={timesLinked}
      on:change={onBlueChange}
    />
  </div>
</div>

<style>
  .linked-times {
    margin-bottom: 16px;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
  }

  h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  .link-toggle {
    -webkit-appearance: none;
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-radius: 999px;
    border: 2px solid var(--color-border);
    background-color: var(--color-bg);
    color: var(--color-fg);
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
    font-family: inherit;
  }

  .link-toggle.linked {
    border-color: var(--color-fg);
    background-color: var(--color-surface);
  }

  .form {
    display: grid;
    gap: 16px;
  }
</style>
