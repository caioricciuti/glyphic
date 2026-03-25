<script lang="ts">
  import type { HookEvent, HookEventConfig, HookHandler } from "$lib/types";
  import { HOOK_EVENT_DESCRIPTIONS } from "$lib/types";
  import HookHandlerForm from "./HookHandlerForm.svelte";

  interface Props {
    event: HookEvent;
    configs: HookEventConfig[] | undefined;
    onchange: () => void;
  }

  let { event, configs = $bindable(), onchange }: Props = $props();

  function addConfig() {
    const newConfig: HookEventConfig = {
      hooks: [{ type: "command", command: "" }],
    };
    configs = [...(configs ?? []), newConfig];
    onchange();
  }

  function removeConfig(index: number) {
    configs = (configs ?? []).filter((_, i) => i !== index);
    if (configs.length === 0) configs = undefined;
    onchange();
  }

  function updateConfig(index: number, config: HookEventConfig) {
    const updated = [...(configs ?? [])];
    updated[index] = config;
    configs = updated;
    onchange();
  }

  function addHandler(configIndex: number) {
    const config = (configs ?? [])[configIndex];
    if (!config) return;
    const updated: HookEventConfig = {
      ...config,
      hooks: [...config.hooks, { type: "command", command: "" }],
    };
    updateConfig(configIndex, updated);
  }

  function removeHandler(configIndex: number, handlerIndex: number) {
    const config = (configs ?? [])[configIndex];
    if (!config) return;
    const updated: HookEventConfig = {
      ...config,
      hooks: config.hooks.filter((_, i) => i !== handlerIndex),
    };
    if (updated.hooks.length === 0) {
      removeConfig(configIndex);
    } else {
      updateConfig(configIndex, updated);
    }
  }

  function updateHandler(configIndex: number, handlerIndex: number, handler: HookHandler) {
    const config = (configs ?? [])[configIndex];
    if (!config) return;
    const updatedHooks = [...config.hooks];
    updatedHooks[handlerIndex] = handler;
    updateConfig(configIndex, { ...config, hooks: updatedHooks });
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-base font-medium text-text-primary">{event}</h3>
      <p class="text-xs text-text-muted">{HOOK_EVENT_DESCRIPTIONS[event]}</p>
    </div>
    <button
      class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
      onclick={addConfig}
    >
      Add Hook
    </button>
  </div>

  {#if !configs || configs.length === 0}
    <div class="bg-bg-secondary border border-border rounded-lg p-6 space-y-4">
      <p class="text-sm text-text-muted text-center">No hooks configured for this event</p>

      <!-- Quick templates -->
      <div class="grid grid-cols-2 gap-2">
        {#each [
          { label: "Shell command", desc: "Run a script before/after", matcher: "", type: "command" as const, command: "echo 'hook triggered'" },
          { label: "HTTP webhook", desc: "Send to an endpoint", matcher: "", type: "http" as const, url: "http://localhost:8080/hook" },
          { label: "Prompt guard", desc: "AI validates the action", matcher: "", type: "prompt" as const, prompt: "Validate this action is safe" },
          { label: "Bash validator", desc: "Validate bash commands", matcher: "Bash", type: "command" as const, command: ".claude/hooks/validate.sh" },
        ] as tpl}
          <button
            class="text-left p-3 bg-bg-tertiary border border-border rounded-md hover:border-accent/30 transition-colors"
            onclick={() => {
              const handler = { type: tpl.type, command: tpl.command, url: tpl.url, prompt: tpl.prompt } as import("$lib/types").HookHandler;
              const newConfig = { matcher: tpl.matcher || undefined, hooks: [handler] };
              configs = [...(configs ?? []), newConfig];
              onchange();
            }}
          >
            <p class="text-xs font-medium text-accent">{tpl.label}</p>
            <p class="text-[10px] text-text-muted">{tpl.desc}</p>
          </button>
        {/each}
      </div>
      <button
        class="mt-3 px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
        onclick={addConfig}
      >
        Add First Hook
      </button>
    </div>
  {:else}
    {#each configs as config, ci}
      <div class="bg-bg-secondary border border-border rounded-lg p-4 space-y-3">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-xs text-text-muted">Hook {ci + 1}</span>
            <span class="text-text-muted">|</span>
            <span class="text-xs text-text-muted">Matcher:</span>
            <input
              type="text"
              aria-label="Matcher pattern"
              class="w-48 px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
              placeholder="* (all tools)"
              value={config.matcher ?? ""}
              oninput={(e) => {
                const val = (e.target as HTMLInputElement).value;
                updateConfig(ci, { ...config, matcher: val || undefined });
              }}
            />
          </div>
          <div class="flex items-center gap-2">
            <button
              class="text-xs text-accent hover:text-accent-hover"
              onclick={() => addHandler(ci)}
            >
              + handler
            </button>
            <button
              class="text-xs text-text-muted hover:text-danger"
              onclick={() => removeConfig(ci)}
            >
              remove
            </button>
          </div>
        </div>

        {#each config.hooks as handler, hi}
          <HookHandlerForm
            {handler}
            onchange={(h) => updateHandler(ci, hi, h)}
            onremove={() => removeHandler(ci, hi)}
          />
        {/each}
      </div>
    {/each}
  {/if}
</div>
