<script lang="ts">
  import type { HookHandler } from "$lib/types";

  interface Props {
    handler: HookHandler;
    onchange: (handler: HookHandler) => void;
    onremove: () => void;
  }

  const { handler, onchange, onremove }: Props = $props();

  function updateField<K extends keyof HookHandler>(key: K, value: HookHandler[K]) {
    onchange({ ...handler, [key]: value });
  }

  function changeType(type: HookHandler["type"]) {
    // Preserve existing values when switching types
    onchange({ ...handler, type });
  }
</script>

<div class="bg-bg-tertiary rounded-md p-3 space-y-2">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      {#each ["command", "http", "prompt", "agent"] as type}
        <button
          class="px-2 py-0.5 text-xs rounded transition-colors
            {handler.type === type
              ? 'bg-accent text-white'
              : 'bg-bg-secondary text-text-muted hover:text-text-secondary'}"
          onclick={() => changeType(type as HookHandler["type"])}
        >
          {type}
        </button>
      {/each}
    </div>
    <button
      class="text-xs text-text-muted hover:text-danger"
      onclick={onremove}
    >
      remove
    </button>
  </div>

  {#if handler.type === "command"}
    <input
      type="text"
      class="w-full px-2 py-1.5 text-sm bg-bg-secondary border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
      placeholder="shell command, e.g. .claude/hooks/validate.sh"
      value={handler.command ?? ""}
      oninput={(e) => updateField("command", (e.target as HTMLInputElement).value)}
    />
  {:else if handler.type === "http"}
    <input
      type="text"
      class="w-full px-2 py-1.5 text-sm bg-bg-secondary border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
      placeholder="URL, e.g. http://localhost:8080/hook"
      value={handler.url ?? ""}
      oninput={(e) => updateField("url", (e.target as HTMLInputElement).value)}
    />
  {:else if handler.type === "prompt" || handler.type === "agent"}
    <textarea
      class="w-full px-2 py-1.5 text-sm bg-bg-secondary border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono resize-y"
      rows="2"
      placeholder="Prompt text"
      value={handler.prompt ?? ""}
      oninput={(e) => updateField("prompt", (e.target as HTMLTextAreaElement).value)}
    ></textarea>
  {/if}

  <!-- Optional fields -->
  <div class="flex items-center gap-4 text-xs">
    <label class="flex items-center gap-1 text-text-muted">
      Timeout:
      <input
        type="number"
        class="w-16 px-1 py-0.5 bg-bg-secondary border border-border rounded text-text-primary focus:outline-none focus:border-accent"
        placeholder="600"
        value={handler.timeout ?? ""}
        oninput={(e) => {
          const val = parseInt((e.target as HTMLInputElement).value);
          updateField("timeout", isNaN(val) ? undefined : val);
        }}
      />
      <span>s</span>
    </label>
    <label class="flex items-center gap-1 text-text-muted">
      Status:
      <input
        type="text"
        class="w-32 px-1 py-0.5 bg-bg-secondary border border-border rounded text-text-primary focus:outline-none focus:border-accent"
        placeholder="message..."
        value={handler.statusMessage ?? ""}
        oninput={(e) => updateField("statusMessage", (e.target as HTMLInputElement).value || undefined)}
      />
    </label>
  </div>
</div>
