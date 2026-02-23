<script lang="ts">
  export let size: number = 48;
  export let thickness: number = 4;
</script>

<div
  class="spinner"
  style="
    width: {size}px;
    height: {size}px;
    border-width: {thickness}px;
  "
></div>

<style>
  .spinner {
    border-radius: 50%;
    border-style: solid;
    border-color: #3b82f6 transparent #3b82f6 transparent;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
