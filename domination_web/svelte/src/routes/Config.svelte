<script lang="ts">
  import UiButton from "../components/UiButton.svelte";
  import UiCardButton from "../components/UiCardButton.svelte";
  import { BASE_URL, post, postFireAndForget } from "../lib/http";
  import { redirect } from "../lib/router";
  import { toast } from "../lib/toast";
  import { scanWifiNetworks, type WifiNetwork } from "../lib/wifi";

  type WifiMode = "ap" | "sta";
  type ConfigPhase = "form" | "sta_next_steps";

  let mode: WifiMode | null = null;
  let configPhase: ConfigPhase = "form";
  let configuredStaSsid = "";
  let password = "";
  let showPassword = false;
  let selectedSsid = "";
  let manualSsid = "";
  let showManualSsid = false;
  let networks: WifiNetwork[] = [];
  let scanning = false;
  let didAutoScan = false;

  $: effectiveSsid = showManualSsid ? manualSsid.trim() : selectedSsid;
  $: selectedNetwork = networks.find((n) => n.ssid === selectedSsid) ?? null;
  $: requiresPassword = showManualSsid
    ? true
    : (selectedNetwork?.requires_password ?? true);
  $: canConfigure =
    configPhase === "form" &&
    (mode === "ap" ||
      (mode === "sta" &&
        effectiveSsid.length > 0 &&
        (!requiresPassword || password.length > 0)));

  function selectNetwork(network: WifiNetwork) {
    selectedSsid = network.ssid;
    showManualSsid = false;
    if (!network.requires_password) {
      password = "";
    }
  }

  async function runScan() {
    scanning = true;
    try {
      networks = await scanWifiNetworks((partial) => {
        if (partial.length > 0) networks = partial;
      });
      if (networks.length === 0) {
        toast.notify("Nenhuma rede encontrada", "error");
      }
    } catch (e) {
      console.error(e);
      const msg =
        e instanceof Error && e.message ? e.message : "Falha ao buscar redes";
      toast.notify(
        msg.includes("configuração inicial") ? msg : "Falha ao buscar redes",
        "error"
      );
    } finally {
      scanning = false;
    }
  }

  $: if (mode === "sta" && !didAutoScan && configPhase === "form") {
    didAutoScan = true;
    void runScan();
  }

  $: if (mode !== "sta") {
    didAutoScan = false;
    networks = [];
    selectedSsid = "";
    manualSsid = "";
    showManualSsid = false;
    password = "";
    showPassword = false;
  }

  async function configure() {
    if (!mode || !canConfigure) return;

    if (mode === "sta") {
      configuredStaSsid = effectiveSsid;
      configPhase = "sta_next_steps";
      postFireAndForget("/app/config", {
        wifi_config: {
          ClientMode: {
            ssid: effectiveSsid,
            password,
          },
        },
      });
      return;
    }

    try {
      await post("/app/config", { wifi_config: "APMode" });
      redirect("/leaderboard");
    } catch (error) {
      console.error(error);
      toast.notify("Falha ao configurar", "error");
    }
  }

  function signalBars(rssi: number): string {
    if (rssi >= -55) return "▂▄▆█";
    if (rssi >= -67) return "▂▄▆";
    if (rssi >= -75) return "▂▄";
    return "▂";
  }
</script>

<div class="page">
  <div class="container">
    {#if configPhase === "sta_next_steps"}
      <h1>Conectando à sua rede</h1>
      <div class="next-steps">
        <p>
          O aparelho está tentando entrar na rede <strong>{configuredStaSsid}</strong>.
          Você sairá do Wi‑Fi <strong>Dominacao</strong> — isso é esperado.
        </p>
        <ol>
          <li>
            No celular, conecte-se à rede <strong>{configuredStaSsid}</strong> (a mesma
            senha que você digitou).
          </li>
          <li>Aguarde cerca de 30 segundos.</li>
          <li>
            Abra <a href={BASE_URL}>{BASE_URL}</a> no navegador.
          </li>
        </ol>
        <p class="hint">
          Se não abrir, confira se o celular está na mesma rede que o dispositivo ou
          reinicie o equipamento e repita a configuração pelo Wi‑Fi
          <strong>Dominacao</strong>.
        </p>
      </div>
    {:else}
      <h1>Configuração de Rede</h1>
      <p class="setup-hint">
        Conecte-se ao Wi‑Fi <strong>Dominacao</strong> e abra
        <a href={BASE_URL}>{BASE_URL}</a>
      </p>

      <div class="options">
        <UiCardButton on:click={() => (mode = "ap")}>
          <span slot="title">Continuar com a rede própria</span>
        </UiCardButton>

        <UiCardButton on:click={() => (mode = "sta")}>
          <span slot="title">Conectar a um Wi-Fi já existente</span>
        </UiCardButton>
      </div>

      {#if mode === "ap"}
        <p class="callout ap-note">
          O dispositivo continua no Wi‑Fi <strong>Dominacao</strong>; seu celular não será
          desconectado ao configurar.
        </p>
      {/if}

      {#if mode === "sta"}
        <p class="callout sta-warning">
          Ao configurar outra rede, você será desconectado do Wi‑Fi
          <strong>Dominacao</strong>. Esta página <strong>não</strong> mostrará se a senha
          funcionou — siga as instruções que aparecerem depois de tocar em
          <strong>Configurar</strong>.
        </p>

        <div class="sta-panel">
          <UiButton
            variant="secondary"
            block
            disabled={scanning}
            on:click={runScan}
          >
            {scanning ? "Buscando redes…" : "Buscar redes"}
          </UiButton>

          {#if networks.length > 0}
            <ul class="network-list">
              {#each networks as network (network.ssid)}
                <li class:selected={!showManualSsid && selectedSsid === network.ssid}>
                  <button
                    type="button"
                    class="network-row"
                    on:click={() => selectNetwork(network)}
                  >
                    <div class="network-info">
                      <strong>{network.ssid}</strong>
                      <span class="meta">{network.auth} · {network.rssi} dBm</span>
                    </div>
                    <span class="signal" aria-hidden="true">{signalBars(network.rssi)}</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}

          <button
            type="button"
            class="manual-toggle"
            on:click={() => {
              showManualSsid = !showManualSsid;
              if (showManualSsid) selectedSsid = "";
            }}
          >
            {showManualSsid ? "Escolher da lista" : "Rede oculta / digitar SSID"}
          </button>

          {#if showManualSsid}
            <div class="form">
              <input type="text" placeholder="SSID" bind:value={manualSsid} />
            </div>
          {/if}

          {#if effectiveSsid && requiresPassword}
            <div class="form password-row">
              <input
                type={showPassword ? "text" : "password"}
                placeholder="Senha do Wi-Fi"
                bind:value={password}
                autocomplete="off"
              />
              <button
                type="button"
                class="peek-btn"
                aria-label={showPassword ? "Ocultar senha" : "Mostrar senha"}
                title={showPassword ? "Ocultar senha" : "Mostrar senha"}
                on:click={() => (showPassword = !showPassword)}
              >
                {showPassword ? "Ocultar" : "Mostrar"}
              </button>
            </div>
          {:else if effectiveSsid && !requiresPassword}
            <p class="hint">Rede aberta — sem senha necessária.</p>
          {/if}
        </div>
      {/if}

      <UiButton variant="primary" block disabled={!canConfigure} on:click={configure}>
        Configurar
      </UiButton>
    {/if}
  </div>
</div>

<style>
  .page {
    height: 100dvh;
    max-height: 100dvh;
    overflow-y: auto;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 24px;
    color: var(--color-fg);
    background: var(--color-bg);
    font-family: system-ui, sans-serif;
    box-sizing: border-box;
  }

  .container {
    width: 100%;
    max-width: 420px;
    text-align: center;
  }

  h1 {
    margin-bottom: 12px;
    font-size: clamp(1.5rem, 5vw, 2rem);
  }

  .setup-hint {
    margin: 0 0 20px;
    font-size: 0.85rem;
    color: var(--color-muted);
    line-height: 1.4;
  }

  .setup-hint a {
    color: var(--color-fg);
    word-break: break-all;
  }

  .callout {
    margin: 0 0 16px;
    padding: 12px 14px;
    border-radius: 12px;
    border: 2px solid var(--color-border);
    font-size: 0.85rem;
    line-height: 1.45;
    text-align: left;
  }

  .sta-warning {
    border-color: var(--color-fg);
    background: color-mix(in srgb, var(--color-fg) 6%, var(--color-bg));
  }

  .ap-note {
    color: var(--color-muted);
  }

  .next-steps {
    text-align: left;
    font-size: 0.9rem;
    line-height: 1.5;
  }

  .next-steps p {
    margin: 0 0 12px;
  }

  .next-steps ol {
    margin: 0 0 16px;
    padding-left: 1.25rem;
  }

  .next-steps li {
    margin-bottom: 8px;
  }

  .next-steps a {
    color: var(--color-fg);
    word-break: break-all;
  }

  .options {
    display: grid;
    gap: 16px;
    margin-bottom: 24px;
  }

  .sta-panel {
    display: grid;
    gap: 12px;
    margin-bottom: 24px;
    text-align: left;
  }

  .network-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: grid;
    gap: 8px;
    max-height: 40vh;
    overflow-y: auto;
  }

  .network-list li {
    border-radius: 12px;
    border: 2px solid var(--color-border);
    background: var(--color-bg);
  }

  .network-list li.selected {
    border-color: var(--color-fg);
  }

  .network-row {
    width: 100%;
    min-height: 3.5rem;
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
    border: none;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    text-align: left;
    line-height: 1.35;
  }

  .network-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .network-info strong {
    font-size: 0.95rem;
    font-weight: 600;
    white-space: normal;
    word-break: break-word;
    line-height: 1.35;
  }

  .meta {
    font-size: 0.75rem;
    color: var(--color-muted);
  }

  .signal {
    font-size: 0.7rem;
    letter-spacing: 1px;
    color: var(--color-muted);
    flex-shrink: 0;
  }

  .manual-toggle {
    background: none;
    border: none;
    color: var(--color-muted);
    font-size: 0.85rem;
    text-decoration: underline;
    cursor: pointer;
    padding: 4px 0;
    text-align: left;
  }

  .form {
    display: grid;
    gap: 12px;
  }

  .password-row {
    grid-template-columns: 1fr auto;
    align-items: center;
    gap: 8px;
  }

  .password-row input {
    min-width: 0;
  }

  .peek-btn {
    flex-shrink: 0;
    padding: 10px 12px;
    border-radius: 10px;
    border: 2px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-fg);
    font-size: 0.8rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-muted);
  }

  input {
    -webkit-appearance: none;
    appearance: none;
    padding: 12px;
    border-radius: 12px;
    border: 2px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-fg);
    font-size: 1rem;
    box-sizing: border-box;
    width: 100%;
  }
</style>
