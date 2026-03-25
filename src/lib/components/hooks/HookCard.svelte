<script lang="ts">
  import type { HookHandler } from "$lib/types";
  import { HOOK_EVENT_DESCRIPTIONS, type HookEvent } from "$lib/types";
  import { Zap, Globe, MessageSquare, Bot, Trash2, ChevronDown } from "lucide-svelte";

  interface Props {
    event: string;
    matcher: string | undefined;
    handler: HookHandler;
    onupdate: (handler: HookHandler) => void;
    ondelete: () => void;
  }

  const { event, matcher, handler, onupdate, ondelete }: Props = $props();

  let expanded = $state(true);

  const TYPE_ICONS = { command: Zap, http: Globe, prompt: MessageSquare, agent: Bot };
  const TYPE_LABELS = { command: "Shell Command", http: "HTTP Webhook", prompt: "AI Prompt", agent: "Agent" };

  function updateField<K extends keyof HookHandler>(key: K, value: HookHandler[K]) {
    onupdate({ ...handler, [key]: value });
  }

  const Icon = $derived(TYPE_ICONS[handler.type] ?? Zap);
  const description = $derived(HOOK_EVENT_DESCRIPTIONS[event as HookEvent] ?? "");
</script>

<div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
  <!-- Header -->
  <div class="flex items-center gap-3 px-4 py-3 cursor-pointer hover:bg-bg-hover/50 transition-colors"
    role="button" tabindex="0"
    onclick={() => (expanded = !expanded)}
    onkeydown={(e) => e.key === "Enter" && (expanded = !expanded)}
  >
    <div class="w-7 h-7 rounded-md bg-accent/10 text-accent flex items-center justify-center shrink-0">
      <Icon size={14} />
    </div>
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium text-text-primary">{event}</span>
        {#if matcher}
          <span class="text-xs px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted font-mono">{matcher}</span>
        {/if}
        <span class="text-xs text-text-muted">· {TYPE_LABELS[handler.type]}</span>
      </div>
      <p class="text-xs text-text-muted truncate">{description}</p>
    </div>
    <div class="flex items-center gap-2 shrink-0">
      <button
        class="text-xs px-2 py-1 rounded text-text-muted hover:text-danger transition-colors"
        onclick={(e) => { e.stopPropagation(); ondelete(); }}
        aria-label="Delete hook"
      >
        <Trash2 size={14} />
      </button>
      <ChevronDown size={14} class="text-text-muted transition-transform {expanded ? 'rotate-180' : ''}" />
    </div>
  </div>

  <!-- Body -->
  {#if expanded}
    <div class="px-4 pb-4 pt-1 space-y-3 border-t border-border">
      <!-- Type selector -->
      <div class="flex items-center gap-2">
        <span class="text-xs text-text-muted w-14 shrink-0">Type</span>
        <div class="flex gap-1" role="group" aria-label="Hook type">
          {#each (["command", "http", "prompt", "agent"] as const) as type}
            <button
              class="px-2.5 py-1 text-xs rounded-md transition-colors
                {handler.type === type
                  ? 'bg-accent text-white'
                  : 'bg-bg-tertiary text-text-muted hover:text-text-secondary'}"
              onclick={() => updateField("type", type)}
            >
              {type}
            </button>
          {/each}
        </div>
      </div>

      <!-- Config field -->
      <div class="flex items-start gap-2">
        <span class="text-xs text-text-muted w-14 shrink-0 pt-2">
          {handler.type === "command" ? "Command" : handler.type === "http" ? "URL" : "Prompt"}
        </span>
        {#if handler.type === "command"}
          <input
            type="text"
            aria-label="Command"
            class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent"
            placeholder="/path/to/script.sh or shell command"
            value={handler.command ?? ""}
            oninput={(e) => updateField("command", (e.target as HTMLInputElement).value)}
          />
        {:else if handler.type === "http"}
          <input
            type="text"
            aria-label="URL"
            class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent"
            placeholder="https://example.com/webhook"
            value={handler.url ?? ""}
            oninput={(e) => updateField("url", (e.target as HTMLInputElement).value)}
          />
        {:else}
          <textarea
            aria-label="Prompt"
            class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent resize-y"
            rows="2"
            placeholder="Describe what to validate or check..."
            value={handler.prompt ?? ""}
            oninput={(e) => updateField("prompt", (e.target as HTMLTextAreaElement).value)}
          ></textarea>
        {/if}
      </div>

      <!-- Advanced: timeout + status -->
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <span class="text-xs text-text-muted">Timeout</span>
          <input
            type="number"
            aria-label="Timeout seconds"
            class="w-16 px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent"
            placeholder="600"
            value={handler.timeout ?? ""}
            oninput={(e) => {
              const val = parseInt((e.target as HTMLInputElement).value);
              updateField("timeout", isNaN(val) ? undefined : val);
            }}
          />
          <span class="text-xs text-text-muted">s</span>
        </div>
        <div class="flex items-center gap-2 flex-1">
          <span class="text-xs text-text-muted">Status</span>
          <input
            type="text"
            aria-label="Status message"
            class="flex-1 px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent"
            placeholder="Validating..."
            value={handler.statusMessage ?? ""}
            oninput={(e) => updateField("statusMessage", (e.target as HTMLInputElement).value || undefined)}
          />
        </div>
      </div>
    </div>
  {/if}
</div>
