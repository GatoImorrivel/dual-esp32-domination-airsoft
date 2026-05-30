<script lang="ts">
  import { createEventDispatcher, tick } from "svelte";
  import {
    clampDurationSecs,
    formatHmsPad,
    hmsToSecs,
    secsToHms,
  } from "../lib/duration";

  export let secs = 10;
  export let disabled = false;
  export let label = "";

  const dispatch = createEventDispatcher<{ change: number }>();

  const ROW_HEIGHT = 40;
  const PADDING_ROWS = 2;
  const MAX_HOURS = 23;

  const hours = Array.from({ length: MAX_HOURS + 1 }, (_, i) => i);
  const minutes = Array.from({ length: 60 }, (_, i) => i);
  const seconds = Array.from({ length: 60 }, (_, i) => i);

  let dialogEl: HTMLDialogElement | undefined;
  let triggerEl: HTMLButtonElement | undefined;
  let hourEl: HTMLDivElement | undefined;
  let minuteEl: HTMLDivElement | undefined;
  let secondEl: HTMLDivElement | undefined;

  let open = false;
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;
  let syncingScroll = false;

  function pad2(n: number): string {
    return String(n).padStart(2, "0");
  }

  function scrollTopForValue(value: number): number {
    return value * ROW_HEIGHT;
  }

  function valueFromScroll(el: HTMLDivElement, max: number): number {
    const index = Math.round(el.scrollTop / ROW_HEIGHT);
    return Math.max(0, Math.min(max, index));
  }

  async function scrollToCurrent(animate = false) {
    if (!hourEl || !minuteEl || !secondEl) return;

    const { hours: h, minutes: m, seconds: s } = secsToHms(secs);
    syncingScroll = true;
    await tick();

    const behavior: ScrollBehavior = animate ? "smooth" : "auto";
    hourEl.scrollTo({ top: scrollTopForValue(h), behavior });
    minuteEl.scrollTo({ top: scrollTopForValue(m), behavior });
    secondEl.scrollTo({ top: scrollTopForValue(s), behavior });

    requestAnimationFrame(() => {
      syncingScroll = false;
    });
  }

  function commitFromScroll() {
    if (syncingScroll || disabled || !open || !hourEl || !minuteEl || !secondEl) return;

    const h = valueFromScroll(hourEl, MAX_HOURS);
    const m = valueFromScroll(minuteEl, 59);
    const s = valueFromScroll(secondEl, 59);
    const next = clampDurationSecs(hmsToSecs(h, m, s));

    if (next !== secs) {
      secs = next;
      dispatch("change", secs);
    }

    scrollToCurrent(false);
  }

  function onColumnScroll() {
    if (syncingScroll) return;
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(commitFromScroll, 80);
  }

  function nudgeColumn(
    el: HTMLDivElement | undefined,
    max: number,
    delta: number
  ) {
    if (!el || disabled || !open) return;
    const current = valueFromScroll(el, max);
    const next = Math.max(0, Math.min(max, current + delta));
    el.scrollTo({ top: scrollTopForValue(next), behavior: "smooth" });
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(commitFromScroll, 120);
  }

  function onColumnKeydown(
    event: KeyboardEvent,
    el: HTMLDivElement | undefined,
    max: number
  ) {
    if (disabled || !open) return;
    if (event.key === "ArrowUp") {
      event.preventDefault();
      nudgeColumn(el, max, -1);
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      nudgeColumn(el, max, 1);
    }
  }

  async function openDialog() {
    if (disabled || !dialogEl) return;
    open = true;
    dialogEl.showModal();
    await tick();
    await scrollToCurrent(false);
    hourEl?.focus();
  }

  function closeDialog() {
    if (!dialogEl?.open) return;
    dialogEl.close();
  }

  function onDialogClose() {
    open = false;
    triggerEl?.focus();
  }

  $: hms = secsToHms(secs);
  $: ariaValue = formatHmsPad(hms.hours, hms.minutes, hms.seconds);

  let prevSecs = secs;
  $: if (secs !== prevSecs) {
    prevSecs = secs;
    if (open && !syncingScroll) {
      scrollToCurrent(false);
    }
  }
</script>

<div class="wheel-picker" class:disabled>
  <span class="team-label">{label}</span>

  <button
    bind:this={triggerEl}
    type="button"
    class="time-trigger"
    {disabled}
    aria-haspopup="dialog"
    aria-expanded={open}
    aria-label="{label} — {ariaValue}"
    on:click={openDialog}
  >
    <span class="time-value">{ariaValue}</span>
    <svg class="chevron" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true">
      <path fill="currentColor" d="M7 10l5 5 5-5H7z" />
    </svg>
  </button>

  <p class="sr-only" aria-live="polite">{ariaValue}</p>
</div>

<dialog
  bind:this={dialogEl}
  class="picker-dialog"
  aria-label="{label} — tempo para dominar"
  on:close={onDialogClose}
>
  <div class="dialog-panel">
    <p class="dialog-title">{label}</p>

    <div class="columns-header" aria-hidden="true">
      <span>HH</span>
      <span class="colon">:</span>
      <span>mm</span>
      <span class="colon">:</span>
      <span>ss</span>
    </div>

    <div class="columns-wrap">
      <div class="selection-band" aria-hidden="true"></div>

      <div
        class="column"
        bind:this={hourEl}
        role="listbox"
        aria-label="Horas"
        tabindex="0"
        on:scroll={onColumnScroll}
        on:keydown={(e) => onColumnKeydown(e, hourEl, MAX_HOURS)}
      >
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
        {#each hours as h}
          <div class="row" style="height: {ROW_HEIGHT}px">{pad2(h)}</div>
        {/each}
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
      </div>

      <span class="colon-sep" aria-hidden="true">:</span>

      <div
        class="column"
        bind:this={minuteEl}
        role="listbox"
        aria-label="Minutos"
        tabindex="0"
        on:scroll={onColumnScroll}
        on:keydown={(e) => onColumnKeydown(e, minuteEl, 59)}
      >
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
        {#each minutes as m}
          <div class="row" style="height: {ROW_HEIGHT}px">{pad2(m)}</div>
        {/each}
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
      </div>

      <span class="colon-sep" aria-hidden="true">:</span>

      <div
        class="column"
        bind:this={secondEl}
        role="listbox"
        aria-label="Segundos"
        tabindex="0"
        on:scroll={onColumnScroll}
        on:keydown={(e) => onColumnKeydown(e, secondEl, 59)}
      >
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
        {#each seconds as s}
          <div class="row" style="height: {ROW_HEIGHT}px">{pad2(s)}</div>
        {/each}
        {#each Array(PADDING_ROWS) as _}
          <div class="row spacer" style="height: {ROW_HEIGHT}px"></div>
        {/each}
      </div>
    </div>

    <button type="button" class="done-btn" on:click={closeDialog}>Concluir</button>
  </div>
</dialog>

<style>
  .wheel-picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .wheel-picker.disabled {
    opacity: 0.55;
  }

  .team-label {
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-fg);
  }

  .time-trigger {
    -webkit-appearance: none;
    appearance: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 10px;
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-fg);
    font-size: 1rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    font-family: inherit;
    cursor: pointer;
    text-align: left;
  }

  .time-trigger:hover:not(:disabled) {
    background: var(--color-surface);
  }

  .time-trigger:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .time-trigger:disabled {
    cursor: not-allowed;
  }

  .time-value {
    flex: 1;
  }

  .chevron {
    flex-shrink: 0;
    opacity: 0.6;
  }

  .picker-dialog {
    z-index: 10000;
    margin: 0;
    padding: 0;
    border: none;
    width: 100%;
    max-width: 100%;
    height: 100%;
    max-height: 100%;
    background: transparent;
    overflow: hidden;
  }

  .picker-dialog::backdrop {
    background: var(--color-overlay);
  }

  .picker-dialog[open] {
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    align-items: center;
  }

  @media (min-width: 480px) {
    .picker-dialog[open] {
      justify-content: center;
    }
  }

  .dialog-panel {
    width: 100%;
    max-width: 320px;
    margin: 0;
    padding: 20px 16px calc(16px + env(safe-area-inset-bottom));
    border-radius: 16px 16px 0 0;
    border: 1px solid var(--color-border);
    border-bottom: none;
    background: var(--color-bg);
    box-shadow: 0 -8px 32px var(--color-overlay);
    box-sizing: border-box;
  }

  @media (min-width: 480px) {
    .dialog-panel {
      margin: auto;
      border-radius: 16px;
      border-bottom: 1px solid var(--color-border);
      padding-bottom: 20px;
    }
  }

  .dialog-title {
    margin: 0 0 12px;
    font-size: 1rem;
    font-weight: 600;
    text-align: center;
    color: var(--color-fg);
  }

  .columns-header {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0;
    font-size: 0.7rem;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-fg);
    opacity: 0.65;
    padding: 0 0.25rem;
    margin-bottom: 6px;
  }

  .columns-header span {
    width: 3.5rem;
    text-align: center;
  }

  .columns-header .colon {
    width: auto;
    flex-shrink: 0;
  }

  .columns-wrap {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    height: calc(5 * 40px);
    border-radius: 12px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    overflow: hidden;
  }

  .columns-wrap::before {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    z-index: 2;
    background: linear-gradient(
      to bottom,
      var(--color-bg) 0%,
      transparent 28%,
      transparent 72%,
      var(--color-bg) 100%
    );
  }

  .selection-band {
    position: absolute;
    left: 0.5rem;
    right: 0.5rem;
    top: 50%;
    height: 40px;
    transform: translateY(-50%);
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    z-index: 1;
    pointer-events: none;
  }

  .column {
    position: relative;
    z-index: 3;
    width: 3.5rem;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    scroll-snap-type: y mandatory;
    -webkit-overflow-scrolling: touch;
    scrollbar-width: none;
    overscroll-behavior: contain;
  }

  .column::-webkit-scrollbar {
    display: none;
  }

  .column:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: -2px;
    border-radius: 6px;
  }

  .row {
    display: flex;
    align-items: center;
    justify-content: center;
    scroll-snap-align: center;
    font-size: 1.25rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
    color: var(--color-fg);
    user-select: none;
  }

  .row.spacer {
    visibility: hidden;
  }

  .colon-sep {
    position: relative;
    z-index: 3;
    font-size: 1.25rem;
    font-weight: 700;
    color: var(--color-fg);
    line-height: 1;
    padding: 0 2px;
    user-select: none;
  }

  .done-btn {
    -webkit-appearance: none;
    appearance: none;
    display: block;
    width: 100%;
    margin-top: 16px;
    padding: 12px 16px;
    border: none;
    border-radius: 10px;
    background: var(--color-fg);
    color: var(--color-bg);
    font-size: 1rem;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
  }

  .done-btn:hover {
    opacity: 0.9;
  }

  .done-btn:focus-visible {
    outline: 2px solid var(--color-fg);
    outline-offset: 2px;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
