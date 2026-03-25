<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { Puzzle, Download, Search, Trash2, Package, Star, Clock, Shield } from "lucide-svelte";

  interface InstalledPlugin {
    name: string;
    marketplace: string;
    scope: string;
    version: string;
    installedAt: string;
  }

  interface BlockedEntry {
    plugin: string;
    added_at: string;
    reason: string;
  }

  interface InstallCount {
    plugin: string;
    unique_installs: number;
  }

  let activeTab = $state<"installed" | "marketplace">("installed");
  let installed = $state<InstalledPlugin[]>([]);
  let blocked = $state<BlockedEntry[]>([]);
  let installCounts = $state<InstallCount[]>([]);
  let loading = $state(true);
  let search = $state("");
  let installing = $state<string | null>(null);
  let actionMessage = $state<string | null>(null);

  // Uninstall
  let uninstallingPlugin = $state<string | null>(null);

  const filteredMarketplace = $derived(
    installCounts
      .filter((p) => !search || p.plugin.toLowerCase().includes(search.toLowerCase()))
      .sort((a, b) => b.unique_installs - a.unique_installs),
  );

  const filteredInstalled = $derived(
    installed.filter((p) => !search || p.name.toLowerCase().includes(search.toLowerCase())),
  );

  function isBlocked(pluginName: string): boolean {
    return blocked.some((b) => b.plugin === pluginName || b.plugin.startsWith(pluginName + "@"));
  }

  function isInstalled(fullName: string): boolean {
    const name = fullName.split("@")[0];
    return installed.some((p) => p.name === name || p.name === fullName);
  }

  async function loadData() {
    loading = true;
    try {
      const [inst, block, counts] = await Promise.all([
        api.plugins.getInstalled(),
        api.plugins.getBlocked(),
        api.plugins.getInstallCounts(),
      ]);

      // Parse installed — plugins is Record<string, InstallEntry[]>
      const instData = inst as { version?: number; plugins?: Record<string, Array<{ scope?: string; version?: string; installedAt?: string }>> };
      const pluginsMap = instData?.plugins;
      if (pluginsMap && typeof pluginsMap === "object" && !Array.isArray(pluginsMap)) {
        installed = Object.entries(pluginsMap).map(([fullName, entries]) => {
          const latest = entries[entries.length - 1] ?? {};
          const [name, marketplace] = fullName.includes("@") ? [fullName.split("@")[0], fullName.split("@")[1]] : [fullName, ""];
          return {
            name,
            marketplace: marketplace || "unknown",
            scope: latest.scope ?? "user",
            version: latest.version ?? "unknown",
            installedAt: latest.installedAt ?? "",
          };
        });
      } else {
        installed = [];
      }

      blocked = (block as BlockedEntry[]) ?? [];

      // Parse install counts — data is { counts: [...] }
      const countsData = counts as { counts?: InstallCount[] } | InstallCount[];
      installCounts = Array.isArray(countsData) ? countsData : (countsData?.counts ?? []);
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loading = false;
    }
  }

  async function installPlugin(fullName: string) {
    installing = fullName;
    actionMessage = null;
    try {
      await api.plugins.install(fullName);
      actionMessage = `Installed ${fullName.split("@")[0]}!`;
      setTimeout(() => (actionMessage = null), 3000);
      await loadData();
    } catch (e) {
      actionMessage = `Error: ${e}`;
      setTimeout(() => (actionMessage = null), 5000);
    } finally {
      installing = null;
    }
  }

  async function uninstallPlugin() {
    if (!uninstallingPlugin) return;
    const name = uninstallingPlugin;
    try {
      await api.plugins.install(`uninstall ${name}`); // Use the install command with "uninstall" — hack but works
      actionMessage = `Uninstalled ${name}!`;
      setTimeout(() => (actionMessage = null), 3000);
      await loadData();
    } catch (e) {
      actionMessage = `Error: ${e}`;
    } finally {
      uninstallingPlugin = null;
    }
  }

  function formatInstalls(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  }

  function formatDate(dateStr: string): string {
    if (!dateStr) return "";
    return new Date(dateStr).toLocaleDateString("en", { month: "short", day: "numeric", year: "numeric" });
  }

  onMount(loadData);
</script>

<ConfirmDialog
  open={uninstallingPlugin !== null}
  title="Uninstall Plugin"
  message="'{uninstallingPlugin}' will be removed."
  confirmLabel="Uninstall"
  onconfirm={uninstallPlugin}
  oncancel={() => (uninstallingPlugin = null)}
/>

<div class="p-6 overflow-y-auto h-full space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
      <button class="px-4 py-1.5 text-sm rounded-md transition-colors {activeTab === 'installed' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { activeTab = "installed"; search = ""; }}>
        Installed
        {#if installed.length > 0}
          <span class="ml-1 text-xs text-accent">{installed.length}</span>
        {/if}
      </button>
      <button class="px-4 py-1.5 text-sm rounded-md transition-colors {activeTab === 'marketplace' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { activeTab = "marketplace"; search = ""; }}>
        Marketplace
        {#if installCounts.length > 0}
          <span class="ml-1 text-xs text-text-muted">{installCounts.length}</span>
        {/if}
      </button>
    </div>
    <div class="flex items-center gap-3">
      {#if actionMessage}
        <span class="text-xs {actionMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{actionMessage}</span>
      {/if}
      {#if blocked.length > 0}
        <span class="text-xs text-text-muted flex items-center gap-1">
          <Shield size={12} />
          {blocked.length} blocked
        </span>
      {/if}
    </div>
  </div>

  <!-- Search -->
  <div class="relative">
    <Search size={14} class="absolute left-3 top-2.5 text-text-muted" />
    <input type="text" class="w-full pl-9 pr-3 py-2 text-sm bg-bg-secondary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search {activeTab === 'installed' ? 'installed' : 'marketplace'} plugins..." bind:value={search} />
  </div>

  {#if loading}
    <p class="text-sm text-text-muted">Loading...</p>
  {:else if activeTab === "installed"}
    <!-- Installed plugins -->
    {#if filteredInstalled.length > 0}
      <div class="grid grid-cols-2 gap-3">
        {#each filteredInstalled as plugin}
          <div class="bg-bg-secondary border border-border rounded-lg p-4 group">
            <div class="flex items-start justify-between">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg bg-accent/10 flex items-center justify-center shrink-0">
                  <Package size={18} class="text-accent" />
                </div>
                <div class="min-w-0">
                  <p class="text-sm font-medium text-text-primary">{plugin.name}</p>
                  <p class="text-xs text-text-muted">{plugin.marketplace}</p>
                </div>
              </div>
              <button
                class="p-1.5 text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={() => (uninstallingPlugin = `${plugin.name}@${plugin.marketplace}`)}
                aria-label="Uninstall"
              >
                <Trash2 size={14} />
              </button>
            </div>
            <div class="flex items-center gap-3 mt-3 text-xs text-text-muted">
              <span class="flex items-center gap-1">
                <Star size={10} />
                v{plugin.version}
              </span>
              <span class="flex items-center gap-1">
                <Clock size={10} />
                {formatDate(plugin.installedAt)}
              </span>
              <span class="px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted">{plugin.scope}</span>
              {#if isBlocked(plugin.name)}
                <span class="px-1.5 py-0.5 rounded bg-danger/10 text-danger">blocked</span>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="bg-bg-secondary border border-border rounded-lg p-12 text-center">
        <Puzzle size={32} class="mx-auto mb-3 opacity-20 text-text-muted" />
        <p class="text-sm text-text-muted mb-1">{search ? "No matching plugins" : "No plugins installed"}</p>
        <p class="text-xs text-text-muted">Browse the marketplace to discover plugins</p>
      </div>
    {/if}

    <!-- Blocked plugins -->
    {#if blocked.length > 0}
      <div>
        <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-2 flex items-center gap-1.5">
          <Shield size={12} /> Blocked
        </h3>
        <div class="space-y-1">
          {#each blocked as b}
            <div class="bg-danger/5 border border-danger/20 rounded-md px-3 py-2 flex items-center justify-between">
              <span class="text-sm text-text-primary">{b.plugin}</span>
              <span class="text-xs text-text-muted">{b.reason}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <!-- Marketplace -->
    {#if filteredMarketplace.length > 0}
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        {#each filteredMarketplace as plugin}
          {@const name = plugin.plugin.split("@")[0]}
          {@const alreadyInstalled = isInstalled(plugin.plugin)}
          <div class="bg-bg-secondary border border-border rounded-lg p-4 flex flex-col justify-between group hover:border-accent/30 transition-colors">
            <div>
              <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-8 h-8 rounded-lg bg-accent/10 flex items-center justify-center shrink-0">
                    <Package size={14} class="text-accent" />
                  </div>
                  <p class="text-sm font-medium text-text-primary">{name}</p>
                </div>
              </div>
              <p class="text-xs text-text-muted mt-2">
                <Download size={10} class="inline" />
                {formatInstalls(plugin.unique_installs)} installs
              </p>
            </div>
            <div class="mt-3">
              {#if alreadyInstalled}
                <span class="text-xs text-success">Installed</span>
              {:else}
                <button
                  class="w-full py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
                  onclick={() => installPlugin(plugin.plugin)}
                  disabled={installing === plugin.plugin}
                >
                  {installing === plugin.plugin ? "Installing..." : "Install"}
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="bg-bg-secondary border border-border rounded-lg p-12 text-center">
        <Search size={32} class="mx-auto mb-3 opacity-20 text-text-muted" />
        <p class="text-sm text-text-muted">{search ? "No plugins match your search" : "No marketplace data available"}</p>
      </div>
    {/if}
  {/if}
</div>
