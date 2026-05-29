<script lang="ts">
  import { onMount } from "svelte";
  import AdminBluetooth from "../components/admin/AdminBluetooth.svelte";
  import AdminGame from "../components/admin/AdminGame.svelte";
  import BackButton from "../components/BackButton.svelte";
  import UiButton from "../components/UiButton.svelte";
  import UiCardButton from "../components/UiCardButton.svelte";
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

  type AdminView = "hub" | "bluetooth" | "game";

  let showLoginModal = false;
  let username = "";
  let password = "";
  let loggingIn = false;
  let authenticated = false;
  let adminView: AdminView = "hub";

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
    adminView = "hub";
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
      toast.notify(`Emparelhado com ${sink.name ?? sink.address}`, "info");
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
      <UiButton
        variant="primary"
        block
        disabled={loggingIn || !username || !password}
        on:click={submitLogin}
      >
        {loggingIn ? "Entrando…" : "Entrar"}
      </UiButton>
    </div>
  </div>
{/if}

{#if authenticated}
  <div class="page">
    <header class="page-header">
      <div class="header-start">
        {#if adminView === "hub"}
          <BackButton to="/leaderboard" />
        {:else}
          <BackButton label="Voltar" onBack={() => (adminView = "hub")} />
        {/if}
      </div>
      <h1>Administração</h1>
      <div class="logout-wrap">
        <UiButton variant="secondary" pill on:click={logout}>Sair</UiButton>
      </div>
    </header>

    <!-- Mobile: hub + drill-down -->
    <div class="content" class:scroll-inner={adminView !== "hub"}>
      {#if adminView === "hub"}
        <div class="hub">
          <UiCardButton on:click={() => (adminView = "bluetooth")}>
            <span slot="title">Áudio Bluetooth</span>
            <span slot="desc">Emparelhar alto-falante e buscar dispositivos</span>
          </UiCardButton>
          <UiCardButton on:click={() => (adminView = "game")}>
            <span slot="title">Partida</span>
            <span slot="desc">Iniciar, parar e configurar tempos</span>
          </UiCardButton>
        </div>
      {:else if adminView === "bluetooth"}
        <AdminBluetooth
          {btState}
          {btLoading}
          {btActionAddress}
          onScan={scanBluetooth}
          onPair={handlePair}
          onUnpair={handleUnpair}
        />
      {:else}
        <AdminGame
          {progress}
          bind:redSecs
          bind:blueSecs
          bind:timesLinked
          onStart={startGame}
          onStop={stopGame}
          onSave={saveConfig}
        />
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: var(--color-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 16px;
  }

  .modal {
    background: var(--color-bg);
    color: var(--color-fg);
    border: 1px solid var(--color-border);
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

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  .modal label {
    display: grid;
    gap: 6px;
    font-size: 0.9rem;
  }

  .modal input {
    padding: 10px;
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: inherit;
  }

  .page {
    height: 100dvh;
    max-height: 100dvh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    font-family: system-ui, sans-serif;
    color: var(--color-fg);
    background: var(--color-bg);
    padding: env(safe-area-inset-top) env(safe-area-inset-right)
      env(safe-area-inset-bottom) env(safe-area-inset-left);
    box-sizing: border-box;
  }

  .page-header {
    flex-shrink: 0;
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
    padding: clamp(0.75rem, 2vw, 1.25rem) clamp(1rem, 4vw, 2rem);
    border-bottom: 1px solid var(--color-border);
  }

  .header-start {
    justify-self: start;
  }

  .page-header h1 {
    margin: 0;
    font-size: clamp(1.25rem, 4vw, 1.75rem);
    text-align: center;
    justify-self: center;
  }

  .logout-wrap {
    justify-self: end;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    padding: clamp(0.75rem, 3vw, 1.25rem);
    box-sizing: border-box;
  }

  .content.scroll-inner {
    overflow-y: auto;
    -webkit-overflow-scrolling: touch;
    overscroll-behavior: contain;
  }

  .content.scroll-inner :global(.panel-section) {
    max-width: 560px;
    margin: 0 auto;
    width: 100%;
  }

  .hub {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 16px;
    max-width: 480px;
    width: 100%;
    margin: 0 auto;
    overflow: hidden;
  }
</style>
