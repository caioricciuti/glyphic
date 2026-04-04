<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { KeybindingEntry } from "$lib/tauri/commands";
  import {
    Keyboard,
    Plus,
    Trash2,
    RotateCcw,
    Save,
  } from "lucide-svelte";

  let bindings = $state<KeybindingEntry[]>([]);
  let defaults = $state<KeybindingEntry[]>([]);
  let loading = $state(true);
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let hasChanges = $state(false);

  async function load() {
    loading = true;
    try {
      const [current, defs] = await Promise.all([
        api.keybindings.read(),
        api.keybindings.getDefaults(),
      ]);
      bindings = current.length > 0 ? current : defs;
      defaults = defs;
    } catch (e) {
      console.error("Failed to load keybindings:", e);
      bindings = [];
    } finally {
      loading = false;
    }
  }

  async function save() {
    saving = true;
    saveMessage = null;
    try {
      await api.keybindings.write(bindings);
      saveMessage = "Saved!";
      hasChanges = false;
      setTimeout(() => (saveMessage = null), 2000);
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  function addBinding() {
    bindings = [
      ...bindings,
      { key: "", command: "", description: "", when: null },
    ];
    hasChanges = true;
  }

  function removeBinding(index: number) {
    bindings = bindings.filter((_, i) => i !== index);
    hasChanges = true;
  }

  function resetToDefaults() {
    bindings = [...defaults];
    hasChanges = true;
  }

  function markChanged() {
    hasChanges = true;
  }

  onMount(load);
</script>

<div class="p-6 overflow-y-auto h-full flex flex-col gap-4">
  <!-- Toolbar -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <button
        class="flex items-center gap-2 px-3 py-1.5 text-sm bg-bg-tertiary hover:bg-bg-hover text-text-secondary rounded-lg transition-colors"
        onclick={addBinding}
      >
        <Plus size={14} />
        Add Binding
      </button>
      <button
        class="flex items-center gap-2 px-3 py-1.5 text-sm bg-bg-tertiary hover:bg-bg-hover text-text-secondary rounded-lg transition-colors"
        onclick={resetToDefaults}
      >
        <RotateCcw size={14} />
        Reset Defaults
      </button>
    </div>

    <div class="flex items-center gap-3">
      {#if saveMessage}
        <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">
          {saveMessage}
        </span>
      {/if}
      <button
        class="flex items-center gap-2 px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
        onclick={save}
        disabled={saving || !hasChanges}
      >
        <Save size={14} />
        {saving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>

  <p class="text-xs text-text-muted font-mono">~/.claude/keybindings.json</p>

  {#if loading}
    <p class="text-sm text-text-muted">Loading keybindings...</p>
  {:else if bindings.length === 0}
    <div class="flex flex-col items-center justify-center h-48 gap-3">
      <Keyboard size={32} class="text-text-muted" />
      <p class="text-sm text-text-muted">No keybindings configured</p>
      <button
        class="px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors"
        onclick={resetToDefaults}
      >
        Load Defaults
      </button>
    </div>
  {:else}
    <!-- Table -->
    <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-border text-xs text-text-muted uppercase tracking-wider">
            <th class="text-left px-4 py-3 w-48">Key</th>
            <th class="text-left px-4 py-3 w-48">Command</th>
            <th class="text-left px-4 py-3">Description</th>
            <th class="text-left px-4 py-3 w-48">When</th>
            <th class="px-4 py-3 w-16"></th>
          </tr>
        </thead>
        <tbody>
          {#each bindings as binding, i}
            <tr class="border-b border-border last:border-b-0 hover:bg-bg-hover/50 transition-colors">
              <td class="px-4 py-2">
                <input
                  type="text"
                  bind:value={binding.key}
                  oninput={markChanged}
                  placeholder="e.g. Ctrl+K"
                  class="w-full px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
                />
              </td>
              <td class="px-4 py-2">
                <input
                  type="text"
                  bind:value={binding.command}
                  oninput={markChanged}
                  placeholder="e.g. submit"
                  class="w-full px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
                />
              </td>
              <td class="px-4 py-2">
                <input
                  type="text"
                  bind:value={binding.description}
                  oninput={markChanged}
                  placeholder="What this does"
                  class="w-full px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
                />
              </td>
              <td class="px-4 py-2">
                <input
                  type="text"
                  bind:value={binding.when}
                  oninput={markChanged}
                  placeholder="(optional)"
                  class="w-full px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
                />
              </td>
              <td class="px-4 py-2 text-center">
                <button
                  class="p-1.5 rounded-md text-text-muted hover:text-danger hover:bg-danger/10 transition-colors"
                  onclick={() => removeBinding(i)}
                  aria-label="Remove binding"
                >
                  <Trash2 size={14} />
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Help -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <p class="text-xs font-medium text-text-secondary mb-2">Key format reference</p>
      <div class="grid grid-cols-2 gap-x-8 gap-y-1 text-xs text-text-muted">
        <span><code class="text-accent">Ctrl+K</code> — modifier + key</span>
        <span><code class="text-accent">Escape Escape</code> — chord (sequential)</span>
        <span><code class="text-accent">Shift+Tab</code> — shift modifier</span>
        <span><code class="text-accent">Enter</code> — single key</span>
        <span><code class="text-accent">Ctrl+Shift+P</code> — multi-modifier</span>
        <span><code class="text-accent">Up</code> / <code class="text-accent">Down</code> — arrow keys</span>
      </div>
    </div>
  {/if}
</div>
