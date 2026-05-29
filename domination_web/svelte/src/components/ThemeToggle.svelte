<script lang="ts">
  import { theme, toggleTheme } from "../lib/theme";
  import UiIconButton from "./UiIconButton.svelte";

  $: isDark = $theme === "dark";
  $: ariaLabel = isDark ? "Tema claro" : "Tema escuro";
</script>

<UiIconButton ariaLabel={ariaLabel} title={ariaLabel} on:click={toggleTheme}>
  {#if isDark}
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <path
        fill="currentColor"
        d="M12 7a5 5 0 1 0 0 10 5 5 0 0 0 0-10M12 1a1 1 0 0 1 1 1v2a1 1 0 1 1-2 0V2a1 1 0 0 1 1-1m0 18a1 1 0 0 1 1 1v2a1 1 0 1 1-2 0v-2a1 1 0 0 1 1-1M4.22 4.22a1 1 0 0 1 1.42 0l1.42 1.42a1 1 0 1 1-1.42 1.42L4.22 5.64a1 1 0 0 1 0-1.42m14.14 0a1 1 0 0 1 0 1.42l-1.42 1.42a1 1 0 1 1-1.42-1.42l1.42-1.42a1 1 0 0 1 1.42 0M1 12a1 1 0 0 1 1-1h2a1 1 0 1 1 0 2H2a1 1 0 0 1-1-1m18 0a1 1 0 0 1 1-1h2a1 1 0 1 1 0 2h-2a1 1 0 0 1-1-1M4.22 19.78a1 1 0 0 1 0-1.42l1.42-1.42a1 1 0 1 1 1.42 1.42l-1.42 1.42a1 1 0 0 1-1.42 0m14.14 0a1 1 0 0 1-1.42 0l-1.42-1.42a1 1 0 1 1 1.42-1.42l1.42 1.42a1 1 0 0 1-1.42 1.42"
      />
    </svg>
  {:else}
    <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
      <path
        fill="currentColor"
        d="M12 3a9 9 0 1 0 9 9c0-.46-.04-.92-.1-1.36a7 7 0 1 1-8.9-8.9A8.96 8.96 0 0 0 12 3"
      />
    </svg>
  {/if}
</UiIconButton>
