<script lang="ts">
  import UiButton from "../components/UiButton.svelte";
  import UiCardButton from "../components/UiCardButton.svelte";
  import { post } from "../lib/http";
  import { redirect } from "../lib/router";
  import { toast } from "../lib/toast";

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

    try {
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
    } catch (error) {
      console.error(error);
      toast.notify("Falha ao conectar na rede", "error");
    }
  }
</script>

<div class="page">
  <div class="container">
    <h1>Configuração de Rede</h1>

    <div class="options">
      <UiCardButton
        on:click={() => (mode = "ap")}
      >
        <span slot="title">Continuar com a rede própria</span>
      </UiCardButton>

      <UiCardButton on:click={() => (mode = "sta")}>
        <span slot="title">Conectar a um Wi-Fi já existente</span>
      </UiCardButton>
    </div>

    {#if mode === "sta"}
      <div class="form">
        <input type="text" placeholder="SSID" bind:value={ssid} />
        <input type="password" placeholder="Senha" bind:value={password} />
      </div>
    {/if}

    <UiButton
      variant="primary"
      block
      disabled={!mode || (mode === "sta" && (!ssid || !password))}
      on:click={configure}
    >
      Configurar
    </UiButton>
  </div>
</div>

<style>
  .page {
    height: 100dvh;
    max-height: 100dvh;
    overflow: hidden;
    display: flex;
    align-items: center;
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
    margin-bottom: 24px;
    font-size: clamp(1.5rem, 5vw, 2rem);
  }

  .options {
    display: grid;
    gap: 16px;
    margin-bottom: 24px;
  }

  .form {
    display: grid;
    gap: 12px;
    margin-bottom: 24px;
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
