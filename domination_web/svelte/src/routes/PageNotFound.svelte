<script lang="ts">
  import BackButton from "../components/BackButton.svelte";
</script>

<div class="wrapper">
  <div class="card">
    <div class="code">404</div>
    <h1>Página não encontrada</h1>
    <p>
      Opa… essa página não existe ou foi movida.
      Se você está configurando o dispositivo, volte para a tela inicial.
    </p>

    <BackButton label="Voltar para início" to="/" />
  </div>
</div>

<style>
  .wrapper {
    min-height: 100dvh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 24px;
    font-family: system-ui, sans-serif;
    color: var(--color-fg);
    background: var(--color-bg);
    box-sizing: border-box;
  }

  .card {
    text-align: center;
    padding: 48px 32px;
    border-radius: 24px;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    max-width: 420px;
    width: 100%;
    display: grid;
    gap: 16px;
    justify-items: center;
  }

  .code {
    font-size: 4rem;
    font-weight: 800;
    letter-spacing: 0.1em;
    color: var(--color-fg);
    line-height: 1;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  p {
    margin: 0;
    color: var(--color-muted);
    line-height: 1.5;
  }
</style>
