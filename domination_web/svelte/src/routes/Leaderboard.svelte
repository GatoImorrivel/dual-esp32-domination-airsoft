<script lang="ts">
  import { get, post } from "../lib/http";

  type Duration = { secs: number; nanos: number };

  let progress: any = null;
  let config: any = null;

  function durationToMs(d: Duration | null | undefined): number {
    if (!d) return 0;
    return d.secs * 1000 + d.nanos / 1_000_000;
  }

  async function load() {
    progress = await get("/game/progress");
    config = await get("/game/config");
  }

  async function start() {
    await post("/game/start");
    await load();
  }

  async function stop() {
    await post("/game/stop");
    await load();
  }

  const interval = setInterval(async () => {
    progress = await get("/game/progress");
  }, 2000);
  load();

  $: redMs = durationToMs(progress?.scores?.red);
  $: blueMs = durationToMs(progress?.scores?.blue);

  $: redTarget = durationToMs(config?.red_time_to_win);
  $: blueTarget = durationToMs(config?.blue_time_to_win);

  function percent(value: number, max: number) {
    if (!max || max <= 0) return 0;
    const p = (value / max) * 100;
    return Math.max(0, Math.min(100, p));
  }

  $: redPercent = percent(redMs, redTarget);
  $: bluePercent = percent(blueMs, blueTarget);
</script>

<div class="container">
  <h1>Placar</h1>

  {#if progress}
    <div class="status">
      {#if progress.is_active}
        <div class="badge running">Partida em andamento</div>
      {:else}
        <div class="badge stopped">Partida parada</div>
      {/if}

      {#if progress.winner}
        <div class="winner">
          🏆 Vencedor: {progress.winner === "red" ? "Vermelho" : "Azul"}
        </div>
      {/if}
    </div>

    <div class="teams">
      <!-- RED -->
      <div class="team">
        <div class="team-header">
          <span>Vermelho</span>
          <strong>{Math.floor(redMs / 1000)}s</strong>
        </div>

        <div class="progress">
          <div class="bar red" style="width: {redPercent}%"></div>
        </div>
      </div>

      <!-- BLUE -->
      <div class="team">
        <div class="team-header">
          <span>Azul</span>
          <strong>{Math.floor(blueMs / 1000)}s</strong>
        </div>

        <div class="progress">
          <div class="bar blue" style="width: {bluePercent}%"></div>
        </div>
      </div>
    </div>

    <div class="controls">
      <button on:click={start} disabled={progress.is_active}> Iniciar </button>
      <button on:click={stop} disabled={!progress.is_active}> Parar </button>
    </div>
  {/if}
</div>

<style>
  .container {
    width: 100%;
    max-width: 640px; /* wider but still mobile-first */
    margin: 40px auto;
    padding: 24px;
    font-family: system-ui, sans-serif;
    text-align: center;
  }

  h1 {
    margin-bottom: 28px;
    font-size: 2rem;
  }

  .status {
    margin-bottom: 28px;
  }

  .badge {
    display: inline-block;
    padding: 10px 18px;
    border-radius: 999px;
    font-size: 0.95rem;
    font-weight: 600;
  }

  .running {
    background: #dcfce7;
    color: #166534;
  }

  .stopped {
    background: #fee2e2;
    color: #991b1b;
  }

  .winner {
    margin-top: 14px;
    font-weight: 600;
    font-size: 1.1rem;
  }

  .teams {
    display: grid;
    gap: 32px;
    margin-bottom: 32px;
  }

  .team {
    text-align: left;
  }

  .team-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 10px;
    font-weight: 600;
    font-size: 1.1rem;
  }

  .progress {
    width: 100%;
    height: 28px; /* thicker bars */
    border-radius: 16px;
    background: #2a2a2a;
    overflow: hidden;
  }

  .bar {
    height: 100%;
    transition: width 0.4s ease;
  }

  .bar.red {
    background: linear-gradient(to right, #ef4444, #dc2626);
  }

  .bar.blue {
    background: linear-gradient(to right, #3b82f6, #2563eb);
  }

  .controls {
    display: flex;
    gap: 16px;
  }

  .controls button {
    flex: 1;
    padding: 16px;
    border-radius: 14px;
    border: none;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    background: #3b82f6;
    color: white;
  }

  .controls button:disabled {
    background: #6b7280;
    cursor: not-allowed;
  }
</style>
