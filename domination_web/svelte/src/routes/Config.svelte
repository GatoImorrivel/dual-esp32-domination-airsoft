<script lang="ts">
  import { post } from "../lib/http";
  import { redirect } from "../lib/router";

  type WifiMode = "ap" | "sta";

  let mode: WifiMode | null = null;
  let ssid: string = "";
  let password: string = "";

  async function configure() {
    if (!mode) return;

    const payload =
      mode === "ap"
        ? { mode: "ap" as const }
        : { mode: "sta" as const, ssid, password };

    await post("/app/config", {
      wifi_config: (() => {
        switch (payload.mode) {
          case "ap":
            return "APMode";
          case "sta":
            return {
              ClientMode: {
                ssid: payload.ssid,
                password: payload.password,
              },
            };
        }
      })(),
    });
    redirect("/leaderboard");
  }
</script>

<div class="container">
  <h1>Configuração de Rede</h1>

  <div class="options">
    <button class:selected={mode === "ap"} on:click={() => (mode = "ap")}>
      <span>Continuar com a rede própria</span>
    </button>

    <button class:selected={mode === "sta"} on:click={() => (mode = "sta")}>
      <span>Conectar a um Wi-Fi já existente</span>
    </button>
  </div>

  {#if mode === "sta"}
    <div class="form">
      <input type="text" placeholder="SSID" bind:value={ssid} />
      <input type="password" placeholder="Senha" bind:value={password} />
    </div>
  {/if}

  <button
    class="configure"
    disabled={!mode || (mode === "sta" && (!ssid || !password))}
    on:click={configure}
  >
    Configurar
  </button>
</div>

<style>
  .container {
    max-width: 420px;
    margin: 40px auto;
    padding: 24px;
    font-family: system-ui, sans-serif;
    text-align: center;
  }

  h1 {
    margin-bottom: 24px;
  }

  .options {
    display: grid;
    gap: 16px;
    margin-bottom: 24px;
  }

  .options button {
    padding: 16px;
    border-radius: 12px;
    border: 2px solid #ddd;
    background: #fafafa;
    cursor: pointer;
    text-align: left;
    font-size: 1rem;
  }

  .options button span {
    display: block;
    font-size: 0.85rem;
    color: #666;
    margin-top: 4px;
  }

  .options button.selected {
    border-color: #3b82f6;
    background: #eff6ff;
  }

  .form {
    display: grid;
    gap: 12px;
    margin-bottom: 24px;
  }

  input {
    padding: 12px;
    border-radius: 8px;
    border: 1px solid #ccc;
    font-size: 1rem;
  }

  .configure {
    width: 100%;
    padding: 16px;
    font-size: 1.1rem;
    font-weight: 600;
    border-radius: 14px;
    border: none;
    background: #3b82f6;
    color: white;
    cursor: pointer;
  }

  .configure:disabled {
    background: #9ca3af;
    cursor: not-allowed;
  }
</style>
