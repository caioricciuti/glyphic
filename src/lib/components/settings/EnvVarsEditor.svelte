<script lang="ts">
  import type { Settings } from "$lib/types";

  interface Props {
    settings: Settings;
  }

  let { settings = $bindable() }: Props = $props();

  let newKey = $state("");
  let newValue = $state("");

  const envVars = $derived(Object.entries(settings.env ?? {}));

  function addVar() {
    if (!newKey.trim()) return;

    const current = { ...(settings.env ?? {}) };
    current[newKey.trim()] = newValue;
    settings = { ...settings, env: current };
    newKey = "";
    newValue = "";
  }

  function removeVar(key: string) {
    const current = { ...(settings.env ?? {}) };
    delete current[key];
    settings = { ...settings, env: Object.keys(current).length > 0 ? current : undefined };
  }
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4 space-y-4">
  <h3 class="text-sm font-medium text-text-secondary">Environment Variables</h3>

  <!-- Add -->
  <div class="flex gap-2">
    <input
      type="text"
      class="w-48 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
      placeholder="KEY"
      bind:value={newKey}
      onkeydown={(e) => e.key === "Enter" && addVar()}
    />
    <input
      type="text"
      class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
      placeholder="value"
      bind:value={newValue}
      onkeydown={(e) => e.key === "Enter" && addVar()}
    />
    <button
      class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
      onclick={addVar}
    >
      Add
    </button>
  </div>

  <!-- List -->
  {#if envVars.length > 0}
    <div class="space-y-1">
      {#each envVars as [key, value]}
        <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group">
          <div class="flex items-center gap-2 font-mono text-sm">
            <span class="text-accent">{key}</span>
            <span class="text-text-muted">=</span>
            <span class="text-text-primary">{value}</span>
          </div>
          <button
            class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs"
            onclick={() => removeVar(key)}
          >
            remove
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-xs text-text-muted">No environment variables configured</p>
  {/if}
</div>
