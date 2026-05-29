<script lang="ts">
  import { onMount } from "svelte";
  import LinkedTeamTimes from "../components/LinkedTeamTimes.svelte";
  import {
    clearSession,
    isLoggedIn,
    loginAndStore,
  } from "../lib/auth";
  import {
    listSinks,
    pairSink,
    scanSinks,
    unpairSink,
    type AudioSink,
    type BtSinksResponse,
  } from "../lib/bt";
  import { authorizedPost, get } from "../lib/http";
  import { toast } from "../lib/toast";

  type Duration = { secs: number; nanos: number };
  type GameConfig = {
    red_time_to_win: Duration;
    blue_time_to_win: Duration;
  };

  let showLoginModal = false;
  let username = "";
  let password = "";
  let loggingIn = false;
  let authenticated = false;

  let config: GameConfig | null = null;
  let redSecs = 10;
  let blueSecs = 10;
  let timesLinked = true;
  let progress: { is_active?: boolean } | null = null;

  let btState: BtSinksResponse | null = null;
  let btLoading = false;
  let btActionAddress: string | null = null;
  let loadingState = false;

  async function loadState() {
    if (loadingState) return;
    loadingState = true;
    try {
      config = await get<GameConfig>("/game/config");
      redSecs = config.red_time_to_win.secs;
      blueSecs = config.blue_time_to_win.secs;
      progress = await get("/game/progress");
      await loadBt();
    } finally {
      loadingState = false;
    }
  }

  async function loadBt() {
    try {
      btState = await listSinks();
    } catch (e) {
      console.error(e);
      btState = null;
    }
  }

  onMount(async () => {
    if (isLoggedIn()) {
      authenticated = true;
      try {
        await loadState();
      } catch (e) {
        console.error(e);
        toast.notify("Falha ao carregar painel admin", "error");
      }
      return;
    }
    showLoginModal = true;
  });

  async function submitLogin() {
    if (!username || !password) return;
    loggingIn = true;
    try {
      await loginAndStore(username, password);
      authenticated = true;
      showLoginModal = false;
      await loadState();
      toast.notify("Autenticado", "info");
    } catch (e) {
      console.error(e);
      toast.notify("Credenciais inválidas", "error");
    } finally {
      loggingIn = false;
    }
  }

  function logout() {
    clearSession();
    authenticated = false;
    showLoginModal = true;
    username = "";
    password = "";
    btState = null;
  }

  async function scanBluetooth() {
    btLoading = true;
    try {
      btState = await scanSinks();
      if (btState.scanning) {
        toast.notify("Buscando dispositivos…", "info");
        const deadline = Date.now() + 20_000;
        while (Date.now() < deadline) {
          await new Promise((r) => setTimeout(r, 1500));
          btState = await listSinks();
          if (!btState.scanning) {
            toast.notify("Busca concluída", "info");
            return;
          }
        }
        toast.notify("Busca demorou demais", "error");
      } else {
        toast.notify("Busca concluída", "info");
      }
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao buscar dispositivos", "error");
    } finally {
      btLoading = false;
    }
  }

  async function handlePair(sink: AudioSink) {
    btActionAddress = sink.address;
    try {
      btState = await pairSink(sink.address);
      toast.notify(`Emparelhado com ${displayName(sink)}`, "info");
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao emparelhar", "error");
    } finally {
      btActionAddress = null;
    }
  }

  async function handleUnpair() {
    btActionAddress = "unpair";
    try {
      btState = await unpairSink();
      toast.notify("Desemparelhado", "info");
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao desemparelhar", "error");
    } finally {
      btActionAddress = null;
    }
  }

  function displayName(sink: AudioSink): string {
    return sink.name ?? sink.address;
  }

  function isPaired(sink: AudioSink): boolean {
    return (
      btState?.paired?.address.toLowerCase() === sink.address.toLowerCase()
    );
  }

  async function startGame() {
    try {
      await authorizedPost("/game/start", {});
      await loadState();
      toast.notify("Partida iniciada", "info");
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao iniciar partida", "error");
    }
  }

  async function stopGame() {
    try {
      await authorizedPost("/game/stop", {});
      await loadState();
      toast.notify("Partida parada", "info");
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao parar partida", "error");
    }
  }

  async function saveConfig() {
    const payload: GameConfig = {
      red_time_to_win: { secs: redSecs, nanos: 0 },
      blue_time_to_win: { secs: blueSecs, nanos: 0 },
    };
    try {
      await authorizedPost("/game/config", payload);
      await loadState();
      toast.notify("Configuração salva", "info");
    } catch (e) {
      console.error(e);
      toast.notify("Falha ao salvar configuração", "error");
    }
  }
</script>

{#if showLoginModal}
  <div class="modal-backdrop" role="presentation">
    <div class="modal" role="dialog" aria-labelledby="login-title">
      <h2 id="login-title">Login administrador</h2>
      <p class="hint">Credenciais ficam salvas neste navegador para renovar o token.</p>
      <label>
        Usuário
        <input type="text" bind:value={username} autocomplete="username" />
      </label>
      <label>
        Senha
        <input
          type="password"
          bind:value={password}
          autocomplete="current-password"
        />
      </label>
      <button
        class="primary"
        disabled={loggingIn || !username || !password}
        on:click={submitLogin}
      >
        {loggingIn ? "Entrando…" : "Entrar"}
      </button>
    </div>
  </div>
{/if}

{#if authenticated}
  <div class="container">
    <header>
      <h1>Administração</h1>
      <button class="logout" on:click={logout}>Sair</button>
    </header>

    <section class="panel-section">
      <h2>Áudio Bluetooth</h2>
      <p class="section-hint">Selecione o alto-falante para anunciar eventos da partida.</p>

      <button
        class="secondary"
        on:click={scanBluetooth}
        disabled={btLoading}
      >
        {btLoading ? "Buscando…" : "Buscar dispositivos"}
      </button>

      {#if btState?.paired}
        <div class="paired-banner">
          <span>Emparelhado:</span>
          <strong>{displayName(btState.paired)}</strong>
          <button
            class="text-btn"
            on:click={handleUnpair}
            disabled={btActionAddress !== null}
          >
            {btActionAddress === "unpair" ? "…" : "Desemparelhar"}
          </button>
        </div>
      {:else}
        <p class="empty">Nenhum alto-falante emparelhado.</p>
      {/if}

      {#if btState && btState.discovered.length > 0}
        <ul class="sink-list">
          {#each btState.discovered as sink (sink.address)}
            <li class:paired={isPaired(sink)}>
              <div class="sink-info">
                <strong>{displayName(sink)}</strong>
                <span class="mac">{sink.address}</span>
              </div>
              {#if isPaired(sink)}
                <span class="tag">Ativo</span>
              {:else}
                <button
                  class="small"
                  on:click={() => handlePair(sink)}
                  disabled={btActionAddress !== null}
                >
                  {btActionAddress === sink.address ? "…" : "Emparelhar"}
                </button>
              {/if}
            </li>
          {/each}
        </ul>
      {:else if !btLoading}
        <p class="empty">Nenhum dispositivo encontrado. Toque em buscar.</p>
      {/if}
    </section>

    <section class="panel-section">
      <h2>Partida</h2>
      {#if progress}
        <div class="status">
          {#if progress.is_active}
            <span class="badge running">Partida em andamento</span>
          {:else}
            <span class="badge stopped">Partida parada</span>
          {/if}
        </div>

        <div class="controls">
          <button on:click={startGame} disabled={progress.is_active}>
            Iniciar
          </button>
          <button on:click={stopGame} disabled={!progress.is_active}>
            Parar
          </button>
        </div>
      {/if}
    </section>

    <section class="panel-section">
      <LinkedTeamTimes bind:redSecs bind:blueSecs bind:timesLinked />
      <button class="primary save" on:click={saveConfig}>
        Salvar configuração
      </button>
    </section>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 16px;
  }

  .modal {
    background: #1e1e1e;
    color: #f5f5f5;
    border-radius: 16px;
    padding: 24px;
    width: 100%;
    max-width: 360px;
    display: grid;
    gap: 12px;
    font-family: system-ui, sans-serif;
  }

  .modal h2 {
    margin: 0;
  }

  .hint,
  .section-hint {
    margin: 0;
    font-size: 0.85rem;
    color: #a3a3a3;
  }

  .modal label {
    display: grid;
    gap: 6px;
    font-size: 0.9rem;
  }

  .modal input {
    padding: 10px;
    border-radius: 8px;
    border: 1px solid #444;
    background: #111;
    color: inherit;
  }

  .container {
    max-width: 520px;
    margin: 40px auto;
    padding: 24px;
    font-family: system-ui, sans-serif;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 28px;
  }

  h1 {
    margin: 0;
    font-size: 1.75rem;
  }

  h2 {
    margin: 0 0 8px;
    font-size: 1.15rem;
  }

  .panel-section {
    margin-bottom: 32px;
    padding-bottom: 28px;
    border-bottom: 1px solid #e5e5e5;
  }

  .panel-section:last-child {
    border-bottom: none;
    margin-bottom: 0;
  }

  .logout {
    padding: 8px 14px;
    border-radius: 8px;
    border: 1px solid #666;
    background: transparent;
    color: inherit;
    cursor: pointer;
  }

  .status {
    margin: 16px 0;
  }

  .badge {
    display: inline-block;
    padding: 8px 14px;
    border-radius: 999px;
    font-size: 0.9rem;
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

  .controls {
    display: flex;
    gap: 12px;
  }

  .controls button,
  .primary,
  .secondary {
    padding: 14px;
    border-radius: 12px;
    border: none;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.95rem;
  }

  .controls button,
  .primary {
    flex: 1;
    background: #3b82f6;
    color: white;
  }

  .secondary {
    width: 100%;
    margin-bottom: 16px;
    background: #e5e7eb;
    color: #111;
  }

  .primary.save {
    width: 100%;
    margin-top: 8px;
  }

  .controls button:disabled,
  .primary:disabled,
  .secondary:disabled {
    background: #9ca3af;
    cursor: not-allowed;
    color: #f3f4f6;
  }

  .paired-banner {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    padding: 12px;
    border-radius: 10px;
    background: #eff6ff;
    margin-bottom: 12px;
    font-size: 0.9rem;
  }

  .text-btn {
    margin-left: auto;
    padding: 6px 10px;
    border: none;
    background: transparent;
    color: #1d4ed8;
    font-weight: 600;
    cursor: pointer;
  }

  .empty {
    font-size: 0.9rem;
    color: #6b7280;
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
    border-radius: 10px;
    border: 1px solid #e5e7eb;
    background: #fafafa;
  }

  .sink-list li.paired {
    border-color: #3b82f6;
    background: #f0f7ff;
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
    color: #6b7280;
    font-family: monospace;
  }

  .tag {
    font-size: 0.75rem;
    font-weight: 700;
    color: #1d4ed8;
    text-transform: uppercase;
  }

  .small {
    padding: 8px 12px;
    border-radius: 8px;
    border: none;
    background: #3b82f6;
    color: white;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .small:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }
</style>
