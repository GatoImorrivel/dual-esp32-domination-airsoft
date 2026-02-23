<div class="wrapper">
  <div class="card">
    <div class="code">404</div>
    <h1>Página não encontrada</h1>
    <p>
      Opa… essa página não existe ou foi movida.
      <br />
      Se você está configurando o dispositivo, volte para a tela inicial.
    </p>

    <a href="/" class="home">Voltar para início</a>
  </div>
</div>

<style>
  .wrapper {
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    background: radial-gradient(circle at top, #eff6ff, #ffffff 60%);
    font-family: system-ui, sans-serif;
  }

  .card {
    text-align: center;
    padding: 48px 32px;
    border-radius: 24px;
    background: white;
    box-shadow:
      0 20px 40px rgba(0, 0, 0, 0.08),
      0 4px 12px rgba(0, 0, 0, 0.06);
    max-width: 420px;
    width: 100%;
  }

  .code {
    font-size: 4rem;
    font-weight: 800;
    letter-spacing: 0.1em;
    color: #3b82f6;
    margin-bottom: 12px;
  }

  h1 {
    margin: 0 0 12px;
    font-size: 1.5rem;
  }

  p {
    margin: 0 0 32px;
    color: #555;
    line-height: 1.5;
  }

  .home {
    display: inline-block;
    padding: 14px 28px;
    border-radius: 999px;
    background: #3b82f6;
    color: white;
    font-weight: 600;
    text-decoration: none;
    transition:
      transform 0.1s ease,
      box-shadow 0.1s ease;
  }

  .home:hover {
    transform: translateY(-1px);
    box-shadow: 0 8px 16px rgba(59, 130, 246, 0.35);
  }
</style>
