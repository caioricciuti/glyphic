<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import { Server, Zap, Shield, CheckCircle, AlertTriangle } from "lucide-svelte";
  import { navigateTo, type Page } from "$lib/stores/navigation.svelte";

  let mcpCount = $state(0);
  let hookCount = $state(0);
  let hasPermissions = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      const [globalSettings, hooks, mcpServers] = await Promise.all([
        api.settings.read("global"),
        api.hooks.get("global"),
        api.mcp.list("global"),
      ]);
      mcpCount = mcpServers ? Object.keys(mcpServers).length : 0;
      hookCount = hooks ? Object.values(hooks).flat().length : 0;
      hasPermissions = !!globalSettings?.permissions;
    } catch {
      // silent
    } finally {
      loading = false;
    }
  });

  interface HealthItem {
    label: string;
    value: string;
    ok: boolean;
    icon: typeof Server;
    page: Page;
  }

  const items = $derived<HealthItem[]>([
    {
      label: "MCP Servers",
      value: mcpCount > 0 ? `${mcpCount} connected` : "None",
      ok: mcpCount > 0,
      icon: Server,
      page: "mcp",
    },
    {
      label: "Hooks",
      value: hookCount > 0 ? `${hookCount} active` : "None",
      ok: hookCount > 0,
      icon: Zap,
      page: "hooks",
    },
    {
      label: "Permissions",
      value: hasPermissions ? "Configured" : "Not set",
      ok: hasPermissions,
      icon: Shield,
      page: "settings",
    },
  ]);
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4">
  <h3 class="text-sm font-medium text-text-secondary mb-3">Config Health</h3>
  {#if loading}
    <p class="text-xs text-text-muted">Loading...</p>
  {:else}
    <div class="space-y-2">
      {#each items as item}
        <button
          class="w-full flex items-center gap-3 px-3 py-2 rounded-md bg-bg-tertiary hover:bg-bg-hover transition-colors text-left"
          onclick={() => navigateTo(item.page)}
        >
          <item.icon size={14} class="text-text-muted shrink-0" />
          <span class="text-xs text-text-primary flex-1">{item.label}</span>
          <span class="flex items-center gap-1 text-xs {item.ok ? 'text-success' : 'text-text-muted'}">
            {item.value}
            {#if item.ok}
              <CheckCircle size={12} />
            {:else}
              <AlertTriangle size={12} class="text-warning" />
            {/if}
          </span>
        </button>
      {/each}
    </div>
  {/if}
</div>
