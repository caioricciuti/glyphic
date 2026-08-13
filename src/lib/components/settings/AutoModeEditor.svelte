<script lang="ts">
  import type { Settings, AutoModeSettings } from "$lib/types";
  import { Wand2 } from "lucide-svelte";

  interface Props {
    settings: Settings;
    // autoMode is only honored from user settings and managed settings
    scopeSupported?: boolean;
  }

  let { settings = $bindable(), scopeSupported = true }: Props = $props();

  const autoMode = $derived(settings.autoMode ?? {});

  type RuleList = "environment" | "allow" | "soft_deny" | "hard_deny";

  let newRule = $state("");
  let newRuleList = $state<RuleList>("allow");

  const listMeta: Record<RuleList, { label: string; color: string; hint: string }> = {
    environment: { label: "Environment", color: "text-info", hint: "Facts about your environment the classifier should know" },
    allow: { label: "Allow", color: "text-success", hint: "Commands matching these prose rules run without asking" },
    soft_deny: { label: "Soft deny", color: "text-warning", hint: "Commands matching these ask for confirmation" },
    hard_deny: { label: "Hard deny", color: "text-danger", hint: "Commands matching these are always blocked" },
  };

  function patch(partial: Partial<AutoModeSettings>) {
    settings = { ...settings, autoMode: { ...autoMode, ...partial } };
  }

  function addRule() {
    const r = newRule.trim();
    if (!r) return;
    const list = autoMode[newRuleList] ?? [];
    if (!list.includes(r)) {
      patch({ [newRuleList]: [...list, r] });
    }
    newRule = "";
  }

  function removeRule(key: RuleList, rule: string) {
    patch({ [key]: (autoMode[key] ?? []).filter((r) => r !== rule) });
  }
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4 space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-text-secondary flex items-center gap-1.5">
      <Wand2 size={14} />
      Auto Mode Classifier
    </h3>
    <label class="flex items-center gap-2 text-xs text-text-muted cursor-pointer">
      <input
        type="checkbox"
        class="accent-accent"
        checked={autoMode.classifyAllShell ?? false}
        onchange={(e) => patch({ classifyAllShell: (e.target as HTMLInputElement).checked })}
      />
      Classify all shell commands
    </label>
  </div>
  <p class="text-xs text-text-muted">
    Plain-language rules that decide which commands run, ask, or get blocked in auto mode.
    Add <code class="font-mono">"$defaults"</code> to a list to keep the built-in rules alongside yours.
  </p>
  {#if !scopeSupported}
    <p class="text-xs text-warning">
      Claude Code reads autoMode from user and managed settings only; rules saved here are ignored.
    </p>
  {/if}

  <div class="flex gap-2">
    <select class="px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent" bind:value={newRuleList}>
      {#each Object.entries(listMeta) as [key, meta]}
        <option value={key}>{meta.label}</option>
      {/each}
    </select>
    <input
      type="text"
      class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
      placeholder='e.g. "installing packages with the project package manager is fine"'
      bind:value={newRule}
      onkeydown={(e) => e.key === "Enter" && addRule()}
    />
    <button class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={addRule}>Add</button>
  </div>

  {#each Object.entries(listMeta) as [key, meta]}
    {@const k = key as RuleList}
    {@const rules = autoMode[k] ?? []}
    {#if rules.length > 0}
      <div>
        <p class="text-xs uppercase tracking-wider {meta.color} mb-1" title={meta.hint}>{meta.label}</p>
        <div class="space-y-1">
          {#each rules as rule}
            <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group">
              <span class="text-sm text-text-primary">{rule}</span>
              <button class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs" onclick={() => removeRule(k, rule)}>remove</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/each}
</div>
