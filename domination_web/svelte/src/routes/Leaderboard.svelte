<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import ThemeToggle from "../components/ThemeToggle.svelte";
  import UiIconButton from "../components/UiIconButton.svelte";
  import { durationToMs, formatDurationRange } from "../lib/duration";
  import { get } from "../lib/http";
  import { navigate } from "../lib/router";
  import { toast } from "../lib/toast";

  type Duration = { secs: number; nanos: number };
  type GameConfig = {
    red_time_to_win: Duration;
    blue_time_to_win: Duration;
  };
  type MatchProgress = {
    scores?: { red?: Duration; blue?: Duration };
    is_active?: boolean;
    winner?: "Red" | "Blue";
  };

  let progress: MatchProgress | null = null;
  let config: GameConfig | null = null;

  let intervalId: ReturnType<typeof setInterval> | null = null;
  let polling = false;
  let lastProgressErrorAt = 0;

  async function fetchGameData() {
    if (polling) return;
    polling = true;

    try {
      const [nextProgress, nextConfig] = await Promise.all([
        get<MatchProgress>("/game/progress"),
        get<GameConfig>("/game/config"),
      ]);
      progress = nextProgress;
      config = nextConfig;
    } catch {
      const now = Date.now();
      if (now - lastProgressErrorAt > 15_000) {
        lastProgressErrorAt = now;
        toast.notify("Não foi possivel buscar progresso", "error");
      }
    } finally {
      polling = false;
    }
  }

  onMount(() => {
    fetchGameData();
    intervalId = setInterval(fetchGameData, 2000);
  });

  onDestroy(() => {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
  });

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
  $: hasWinner = Boolean(progress?.winner);
</script>

<div class="page">
  <header class="top-bar">
    <ThemeToggle />
    <div class="top-bar-spacer" aria-hidden="true"></div>
    <UiIconButton
      ariaLabel="Administração"
      title="Administração"
      on:click={() => navigate("/admin")}
    >
      <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
        <path
          fill="currentColor"
          d="M12 15.5A3.5 3.5 0 0 1 8.5 12 3.5 3.5 0 0 1 12 8.5a3.5 3.5 0 0 1 3.5 3.5 3.5 3.5 0 0 1-3.5 3.5m7.43-2.53c.04-.32.07-.64.07-.97 0-.33-.03-.66-.07-1l2.11-1.65c.2-.15.25-.42.12-.64l-2-3.46c-.12-.22-.39-.3-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65A.506.506 0 0 0 14 2h-4c-.25 0-.46.18-.5.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1c-.22-.08-.49 0-.61.22l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.34-.07.67-.07 1 0 .33.03.66.07.97l-2.11 1.65c-.2.15-.25.42-.12.64l2 3.46c.12.22.39.3.61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.04.24.25.42.5.42h4c.25 0 .46-.18.5-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.22.08.49 0 .61-.22l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65z"
        />
      </svg>
    </UiIconButton>
  </header>

  <main class="scoreboard" class:with-winner={hasWinner}>
    {#if progress?.winner}
      <div
        class="winner-banner"
        class:red={progress.winner === "Red"}
        class:blue={progress.winner === "Blue"}
      >
        {progress.winner === "Red" ? "VERMELHO VENCEU" : "AZUL VENCEU"}
      </div>
    {/if}

    <h1>Placar</h1>

    {#if progress}
      <div class="status">
        <span class="badge">
          {progress.is_active ? "Partida em andamento" : "Partida parada"}
        </span>
      </div>

      <div class="teams">
        <div class="team">
          <div class="team-header">
            <span class="team-name">Vermelho</span>
            <strong>{formatDurationRange(redMs, redTarget)}</strong>
          </div>
          <div class="progress">
            <div class="bar red" style="width: {redPercent}%"></div>
          </div>
        </div>

        <div class="team">
          <div class="team-header">
            <span class="team-name">Azul</span>
            <strong>{formatDurationRange(blueMs, blueTarget)}</strong>
          </div>
          <div class="progress">
            <div class="bar blue" style="width: {bluePercent}%"></div>
          </div>
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .page {
    height: 100%;
    max-height: 100dvh;
    overflow: hidden;
    overscroll-behavior: none;
    display: flex;
    flex-direction: column;
    font-family: system-ui, sans-serif;
    color: var(--color-fg);
    background: var(--color-bg);
    box-sizing: border-box;
    padding: env(safe-area-inset-top) env(safe-area-inset-right)
      env(safe-area-inset-bottom) env(safe-area-inset-left);
  }

  .top-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
    padding: 0.75rem clamp(0.75rem, 2vw, 1.25rem);
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
  }

  .top-bar-spacer {
    flex: 1;
  }

  .scoreboard {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    width: 100%;
    margin: 0;
    padding: 0.5rem clamp(0.75rem, 2vw, 1.25rem);
    box-sizing: border-box;
    text-align: center;
    overflow: hidden;
    overscroll-behavior: none;
  }

  .scoreboard.with-winner {
    justify-content: flex-start;
    padding-top: 0.5rem;
  }

  h1 {
    flex-shrink: 0;
    margin: 0 0 0.5rem;
    font-size: clamp(1.5rem, 5vw, 2.75rem);
    line-height: 1.05;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .with-winner h1 {
    margin-bottom: 0.35rem;
    font-size: clamp(1.25rem, 4vw, 2rem);
  }

  .status {
    flex-shrink: 0;
    margin-bottom: 0.75rem;
  }

  .with-winner .status {
    margin-bottom: 0.5rem;
  }

  .badge {
    display: inline-block;
    padding: 0.4rem 0.9rem;
    border-radius: 999px;
    font-size: clamp(0.8rem, 2vw, 1rem);
    font-weight: 600;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-fg);
  }

  .teams {
    flex: 0 1 auto;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: clamp(0.75rem, 3vh, 1.75rem);
    width: 100%;
    max-height: 100%;
  }

  .with-winner .teams {
    gap: clamp(0.5rem, 2vh, 1rem);
  }

  .team {
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    text-align: left;
    width: 100%;
  }

  .team-header {
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 1rem;
    margin-bottom: 0.35rem;
    font-weight: 700;
    font-size: clamp(1rem, 3.5vw, 1.75rem);
  }

  .team-name {
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .team-header strong {
    font-variant-numeric: tabular-nums;
    font-size: clamp(0.9rem, 3vw, 1.5rem);
  }

  .progress {
    height: clamp(2.25rem, 8vh, 4rem);
    width: 100%;
    border-radius: 999px;
    background: var(--color-track);
    overflow: hidden;
    flex-shrink: 0;
  }

  .bar {
    height: 100%;
    transition: width 0.4s ease;
    min-width: 0;
  }

  .bar.red {
    background: linear-gradient(to right, #ef4444, #dc2626);
  }

  .bar.blue {
    background: linear-gradient(to right, #3b82f6, #2563eb);
  }

  .winner-banner {
    flex-shrink: 1;
    min-height: 0;
    width: 100%;
    padding: clamp(0.5rem, 2vw, 1rem) clamp(0.75rem, 2vw, 1rem);
    font-size: clamp(1rem, 3.5vw, 1.75rem);
    font-weight: 800;
    text-align: center;
    letter-spacing: 0.04em;
    border-radius: 0.75rem;
    margin-bottom: 0.5rem;
    box-sizing: border-box;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .winner-banner.red {
    background: linear-gradient(135deg, #ef4444, #b91c1c);
    color: white;
  }

  .winner-banner.blue {
    background: linear-gradient(135deg, #3b82f6, #1d4ed8);
    color: white;
  }
</style>
