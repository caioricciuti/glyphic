<script lang="ts">
  import type { Settings } from "$lib/types";

  interface Props {
    settings: Settings;
  }

  let { settings = $bindable() }: Props = $props();

  let newRule = $state("");
  let newRuleAction = $state<"allow" | "ask" | "deny">("allow");

  const permissions = $derived(settings.permissions ?? { allow: [], ask: [], deny: [] });

  function addRule() {
    if (!newRule.trim()) return;

    const current = { ...permissions };
    const list = current[newRuleAction] ?? [];
    if (!list.includes(newRule.trim())) {
      current[newRuleAction] = [...list, newRule.trim()];
      settings = { ...settings, permissions: current };
    }
    newRule = "";
  }

  function removeRule(action: "allow" | "ask" | "deny", rule: string) {
    const current = { ...permissions };
    current[action] = (current[action] ?? []).filter((r) => r !== rule);
    settings = { ...settings, permissions: current };
  }
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4 space-y-4">
  <h3 class="text-sm font-medium text-text-secondary">Permissions</h3>

  <!-- Add Rule -->
  <div class="flex gap-2">
    <select
      class="px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent"
      bind:value={newRuleAction}
    >
      <option value="allow">Allow</option>
      <option value="ask">Ask</option>
      <option value="deny">Deny</option>
    </select>
    <input
      type="text"
      class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
      placeholder='e.g. Bash(npm run *), Edit(/src/**/*.ts), WebFetch(domain:github.com)'
      bind:value={newRule}
      onkeydown={(e) => e.key === "Enter" && addRule()}
    />
    <button
      class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
      onclick={addRule}
    >
      Add
    </button>
  </div>

  <!-- Rules Lists -->
  {#each [
    { key: "allow" as const, label: "Allow", color: "text-success" },
    { key: "ask" as const, label: "Ask", color: "text-warning" },
    { key: "deny" as const, label: "Deny", color: "text-danger" },
  ] as section}
    {@const rules = permissions[section.key] ?? []}
    {#if rules.length > 0}
      <div>
        <p class="text-xs uppercase tracking-wider {section.color} mb-2">{section.label}</p>
        <div class="space-y-1">
          {#each rules as rule}
            <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group">
              <code class="text-sm text-text-primary font-mono">{rule}</code>
              <button
                class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs"
                onclick={() => removeRule(section.key, rule)}
              >
                remove
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/each}
</div>
