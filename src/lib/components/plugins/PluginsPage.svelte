<script lang="ts">
  import { onMount } from "svelte";
  import { api, type InstalledPlugin, type Marketplace, type PluginScope } from "$lib/tauri/commands";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import {
    Puzzle, Download, Search, Trash2, Package, Star, Clock, Shield,
    Power, RefreshCw, Info, Plus, Store, X, Eraser,
  } from "lucide-svelte";

  interface BlockedEntry {
    plugin: string;
    added_at: string;
    reason: string;
  }

  interface InstallCount {
    plugin: string;
    unique_installs: number;
  }

  interface MarketplaceData {
    name?: string;
    plugins?: Array<{ name: string; description?: string; category?: string }>;
  }

  interface MarketplacePlugin {
    name: string;
    description: string;
    category?: string;
    marketplace: string;
  }

  let activeTab = $state<"installed" | "marketplace" | "sources">("installed");
  let installed = $state<InstalledPlugin[]>([]);
  let blocked = $state<BlockedEntry[]>([]);
  let installCounts = $state<InstallCount[]>([]);
  let marketplacePlugins = $state<MarketplacePlugin[]>([]);
  let marketplaces = $state<Marketplace[]>([]);
  let loading = $state(true);
  let search = $state("");
  let actionMessage = $state<string | null>(null);
  // One in-flight CLI action at a time, keyed "action:id" for per-button spinners
  let busy = $state<string | null>(null);

  let installScope = $state<PluginScope>("user");
  const projectPath = $derived(getSelectedProjectPath());

  let uninstallingPlugin = $state<InstalledPlugin | null>(null);
  let removingMarketplace = $state<string | null>(null);
  let newMarketplaceSource = $state("");

  // Details modal
  let detailsFor = $state<string | null>(null);
  let detailsText = $state("");

  const installCountMap = $derived(new Map(installCounts.map((c) => [c.plugin, c.unique_installs])));

  const filteredMarketplace = $derived(
    marketplacePlugins
      .filter(
        (p) =>
          !search ||
          p.name.toLowerCase().includes(search.toLowerCase()) ||
          p.description.toLowerCase().includes(search.toLowerCase()),
      )
      .sort(
        (a, b) =>
          (installCountMap.get(`${b.name}@${b.marketplace}`) ?? 0) -
          (installCountMap.get(`${a.name}@${a.marketplace}`) ?? 0),
      ),
  );

  const filteredInstalled = $derived(
    installed.filter((p) => !search || p.id.toLowerCase().includes(search.toLowerCase())),
  );

  function bareName(id: string): string {
    return id.split("@")[0];
  }

  function marketplaceOf(id: string): string {
    return id.includes("@") ? id.split("@")[1] : "";
  }

  function isBlocked(id: string): boolean {
    const name = bareName(id);
    return blocked.some((b) => b.plugin === name || b.plugin.startsWith(name + "@"));
  }

  function isInstalled(fullName: string): boolean {
    return installed.some((p) => p.id === fullName || bareName(p.id) === bareName(fullName));
  }

  function flash(message: string, isError = false) {
    actionMessage = message;
    setTimeout(() => (actionMessage = null), isError ? 6000 : 3000);
  }

  async function loadData() {
    loading = true;
    try {
      const [list, block, counts, marketData, sources] = await Promise.all([
        api.plugins.list().catch(() => [] as InstalledPlugin[]),
        api.plugins.getBlocked(),
        api.plugins.getInstallCounts(),
        api.plugins.getMarketplace(),
        api.plugins.marketplaceList().catch(() => [] as Marketplace[]),
      ]);

      installed = list;
      marketplaces = sources;
      blocked = (block as BlockedEntry[]) ?? [];

      const countsData = counts as { counts?: InstallCount[] } | InstallCount[];
      installCounts = Array.isArray(countsData) ? countsData : (countsData?.counts ?? []);

      const markets = (marketData as MarketplaceData[]) ?? [];
      marketplacePlugins = markets.flatMap((m) =>
        (m.plugins ?? []).map((p) => ({
          name: p.name,
          description: p.description ?? "",
          category: p.category,
          marketplace: m.name ?? "unknown",
        })),
      );
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loading = false;
    }
  }

  async function runAction(key: string, fn: () => Promise<string>, successMsg: string) {
    if (busy) return;
    busy = key;
    try {
      await fn();
      flash(successMsg);
      await loadData();
    } catch (e) {
      flash(`Error: ${e}`, true);
    } finally {
      busy = null;
    }
  }

  function installPlugin(fullName: string) {
    const scope = installScope;
    const pp = scope === "user" ? undefined : (projectPath ?? undefined);
    if (scope !== "user" && !pp) {
      flash("Error: pick a project (in any project-scoped page) before installing to project/local scope", true);
      return;
    }
    runAction(`install:${fullName}`, () => api.plugins.install(fullName, scope, pp), `Installed ${bareName(fullName)}`);
  }

  function togglePlugin(plugin: InstalledPlugin) {
    const fn = plugin.enabled ? api.plugins.disable : api.plugins.enable;
    runAction(
      `toggle:${plugin.id}`,
      () => fn(plugin.id, plugin.scope, undefined),
      `${plugin.enabled ? "Disabled" : "Enabled"} ${bareName(plugin.id)}`,
    );
  }

  function updatePlugin(plugin: InstalledPlugin) {
    runAction(`update:${plugin.id}`, () => api.plugins.update(plugin.id), `Updated ${bareName(plugin.id)} (restart Claude Code to apply)`);
  }

  function prunePlugins() {
    runAction("prune", () => api.plugins.prune(), "Pruned unused dependencies");
  }

  async function confirmUninstall() {
    if (!uninstallingPlugin) return;
    const p = uninstallingPlugin;
    uninstallingPlugin = null;
    await runAction(`uninstall:${p.id}`, () => api.plugins.uninstall(p.id, p.scope, undefined), `Uninstalled ${bareName(p.id)}`);
  }

  async function showDetails(plugin: InstalledPlugin) {
    detailsFor = plugin.id;
    detailsText = "Loading...";
    try {
      detailsText = await api.plugins.details(bareName(plugin.id));
    } catch (e) {
      detailsText = plugin.enabled
        ? `Failed to load details: ${e}`
        : `Details are only available for enabled plugins.\n\n(${e})`;
    }
  }

  function addMarketplace() {
    const source = newMarketplaceSource.trim();
    if (!source) return;
    runAction(`market-add:${source}`, () => api.plugins.marketplaceAdd(source), `Added marketplace`).then(() => {
      newMarketplaceSource = "";
    });
  }

  function updateMarketplace(name?: string) {
    runAction(`market-update:${name ?? "all"}`, () => api.plugins.marketplaceUpdate(name), name ? `Updated ${name}` : "Updated all marketplaces");
  }

  async function confirmRemoveMarketplace() {
    if (!removingMarketplace) return;
    const name = removingMarketplace;
    removingMarketplace = null;
    await runAction(`market-remove:${name}`, () => api.plugins.marketplaceRemove(name), `Removed ${name}`);
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
  message="'{uninstallingPlugin?.id}' will be removed."
  confirmLabel="Uninstall"
  onconfirm={confirmUninstall}
  oncancel={() => (uninstallingPlugin = null)}
/>

<ConfirmDialog
  open={removingMarketplace !== null}
  title="Remove Marketplace"
  message="'{removingMarketplace}' will be removed. If this is the last scope using it, its installed plugins are uninstalled too."
  confirmLabel="Remove"
  onconfirm={confirmRemoveMarketplace}
  oncancel={() => (removingMarketplace = null)}
/>

{#if detailsFor}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button class="absolute inset-0 bg-black/50" onclick={() => (detailsFor = null)} aria-label="Close dialog"></button>
    <div class="relative bg-bg-secondary border border-border rounded-xl shadow-2xl w-[560px] max-h-[70vh] p-6 z-10 flex flex-col">
      <div class="flex items-center justify-between mb-3">
        <h3 class="text-base font-semibold text-text-primary">{detailsFor}</h3>
        <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => (detailsFor = null)} aria-label="Close">
          <X size={16} />
        </button>
      </div>
      <pre class="text-xs text-text-secondary font-mono whitespace-pre-wrap overflow-y-auto bg-bg-tertiary rounded-lg p-4">{detailsText}</pre>
    </div>
  </div>
{/if}

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
        {#if marketplacePlugins.length > 0}
          <span class="ml-1 text-xs text-text-muted">{marketplacePlugins.length}</span>
        {/if}
      </button>
      <button class="px-4 py-1.5 text-sm rounded-md transition-colors {activeTab === 'sources' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { activeTab = "sources"; search = ""; }}>
        Sources
        {#if marketplaces.length > 0}
          <span class="ml-1 text-xs text-text-muted">{marketplaces.length}</span>
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
      {#if activeTab === "installed"}
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-md transition-colors disabled:opacity-50"
          onclick={prunePlugins}
          disabled={busy !== null}
          title="Remove auto-installed dependencies that are no longer needed"
        >
          <Eraser size={12} />
          Prune
        </button>
      {/if}
    </div>
  </div>

  {#if activeTab !== "sources"}
    <!-- Search -->
    <div class="relative">
      <Search size={14} class="absolute left-3 top-2.5 text-text-muted" />
      <input type="text" class="w-full pl-9 pr-3 py-2 text-sm bg-bg-secondary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search {activeTab === 'installed' ? 'installed' : 'marketplace'} plugins..." bind:value={search} />
    </div>
  {/if}

  {#if loading}
    <p class="text-sm text-text-muted">Loading...</p>
  {:else if activeTab === "installed"}
    <!-- Installed plugins -->
    {#if filteredInstalled.length > 0}
      <div class="grid grid-cols-2 gap-3">
        {#each filteredInstalled as plugin (plugin.id)}
          <div class="bg-bg-secondary border border-border rounded-lg p-4 group {plugin.enabled ? '' : 'opacity-70'}">
            <div class="flex items-start justify-between">
              <div class="flex items-center gap-3">
                <div class="w-10 h-10 rounded-lg {plugin.enabled ? 'bg-accent/10' : 'bg-bg-tertiary'} flex items-center justify-center shrink-0">
                  <Package size={18} class={plugin.enabled ? "text-accent" : "text-text-muted"} />
                </div>
                <div class="min-w-0">
                  <p class="text-sm font-medium text-text-primary">{bareName(plugin.id)}</p>
                  <p class="text-xs text-text-muted">{marketplaceOf(plugin.id)}</p>
                </div>
              </div>
              <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  class="p-1.5 text-text-muted hover:text-text-primary"
                  onclick={() => showDetails(plugin)}
                  aria-label="Details"
                  title="Component inventory and token cost"
                >
                  <Info size={14} />
                </button>
                <button
                  class="p-1.5 text-text-muted hover:text-text-primary disabled:opacity-50"
                  onclick={() => updatePlugin(plugin)}
                  disabled={busy !== null}
                  aria-label="Update"
                  title="Update to latest version"
                >
                  <RefreshCw size={14} class={busy === `update:${plugin.id}` ? "animate-spin" : ""} />
                </button>
                <button
                  class="p-1.5 text-text-muted hover:text-danger disabled:opacity-50"
                  onclick={() => (uninstallingPlugin = plugin)}
                  disabled={busy !== null}
                  aria-label="Uninstall"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
            <div class="flex items-center justify-between mt-3">
              <div class="flex items-center gap-3 text-xs text-text-muted">
                <span class="flex items-center gap-1">
                  <Star size={10} />
                  v{plugin.version}
                </span>
                <span class="flex items-center gap-1">
                  <Clock size={10} />
                  {formatDate(plugin.installedAt)}
                </span>
                <span class="px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted">{plugin.scope}</span>
                {#if isBlocked(plugin.id)}
                  <span class="px-1.5 py-0.5 rounded bg-danger/10 text-danger">blocked</span>
                {/if}
              </div>
              <button
                class="flex items-center gap-1.5 px-2 py-1 text-xs rounded-md transition-colors disabled:opacity-50 {plugin.enabled ? 'text-success hover:bg-success/10' : 'text-text-muted hover:bg-bg-tertiary'}"
                onclick={() => togglePlugin(plugin)}
                disabled={busy !== null}
                title={plugin.enabled ? "Disable" : "Enable"}
              >
                <Power size={12} />
                {busy === `toggle:${plugin.id}` ? "..." : plugin.enabled ? "Enabled" : "Disabled"}
              </button>
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
  {:else if activeTab === "marketplace"}
    <!-- Install scope -->
    <div class="flex items-center gap-2 text-xs text-text-muted">
      <span>Install to:</span>
      <div class="flex gap-1 bg-bg-tertiary rounded-md p-0.5">
        {#each ["user", "project", "local"] as const as scope}
          <button
            class="px-2 py-1 rounded transition-colors {installScope === scope ? 'bg-bg-secondary text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
            onclick={() => (installScope = scope)}
          >{scope}</button>
        {/each}
      </div>
      {#if installScope !== "user"}
        <span class="font-mono truncate max-w-[300px]">{projectPath ?? "no project selected"}</span>
      {/if}
    </div>

    <!-- Marketplace -->
    {#if filteredMarketplace.length > 0}
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        {#each filteredMarketplace as plugin}
          {@const fullName = `${plugin.name}@${plugin.marketplace}`}
          {@const alreadyInstalled = isInstalled(fullName)}
          {@const installs = installCountMap.get(fullName)}
          <div class="bg-bg-secondary border border-border rounded-lg p-4 flex flex-col justify-between group hover:border-accent/30 transition-colors">
            <div>
              <div class="flex items-start justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-8 h-8 rounded-lg bg-accent/10 flex items-center justify-center shrink-0">
                    <Package size={14} class="text-accent" />
                  </div>
                  <div class="min-w-0">
                    <p class="text-sm font-medium text-text-primary">{plugin.name}</p>
                    <p class="text-xs text-text-muted">{plugin.marketplace}</p>
                  </div>
                </div>
              </div>
              {#if plugin.description}
                <p class="text-xs text-text-muted mt-2 line-clamp-2">{plugin.description}</p>
              {/if}
              <div class="flex items-center gap-2 mt-2">
                {#if plugin.category}
                  <span class="px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted text-xs">{plugin.category}</span>
                {/if}
                {#if installs}
                  <p class="text-xs text-text-muted">
                    <Download size={10} class="inline" />
                    {formatInstalls(installs)} installs
                  </p>
                {/if}
              </div>
            </div>
            <div class="mt-3">
              {#if alreadyInstalled}
                <span class="text-xs text-success">Installed</span>
              {:else}
                <button
                  class="w-full py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
                  onclick={() => installPlugin(fullName)}
                  disabled={busy !== null}
                >
                  {busy === `install:${fullName}` ? "Installing..." : "Install"}
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
  {:else}
    <!-- Marketplace sources -->
    <div class="flex gap-2">
      <input
        type="text"
        class="flex-1 px-3 py-2 text-sm bg-bg-secondary border border-border rounded-md text-text-primary font-mono placeholder:text-text-muted focus:outline-none focus:border-accent"
        placeholder="owner/repo, git URL, marketplace.json URL, or local path"
        bind:value={newMarketplaceSource}
        onkeydown={(e) => { if (e.key === "Enter") addMarketplace(); }}
      />
      <button
        class="flex items-center gap-1.5 px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
        onclick={addMarketplace}
        disabled={busy !== null || !newMarketplaceSource.trim()}
      >
        <Plus size={14} />
        Add
      </button>
      <button
        class="flex items-center gap-1.5 px-3 py-2 text-sm text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-md transition-colors disabled:opacity-50"
        onclick={() => updateMarketplace()}
        disabled={busy !== null || marketplaces.length === 0}
        title="Refresh all marketplaces"
      >
        <RefreshCw size={14} class={busy === "market-update:all" ? "animate-spin" : ""} />
        Update all
      </button>
    </div>

    {#if marketplaces.length > 0}
      <div class="space-y-2">
        {#each marketplaces as m (m.name)}
          <div class="bg-bg-secondary border border-border rounded-lg px-4 py-3 flex items-center justify-between group">
            <div class="flex items-center gap-3 min-w-0">
              <div class="w-8 h-8 rounded-lg bg-accent/10 flex items-center justify-center shrink-0">
                <Store size={14} class="text-accent" />
              </div>
              <div class="min-w-0">
                <p class="text-sm font-medium text-text-primary">{m.name}</p>
                <p class="text-xs text-text-muted font-mono truncate">{m.repo ?? m.installLocation} ({m.source})</p>
              </div>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
              <button
                class="p-1.5 text-text-muted hover:text-text-primary disabled:opacity-50"
                onclick={() => updateMarketplace(m.name)}
                disabled={busy !== null}
                aria-label="Update marketplace"
              >
                <RefreshCw size={14} class={busy === `market-update:${m.name}` ? "animate-spin" : ""} />
              </button>
              <button
                class="p-1.5 text-text-muted hover:text-danger disabled:opacity-50"
                onclick={() => (removingMarketplace = m.name)}
                disabled={busy !== null}
                aria-label="Remove marketplace"
              >
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <div class="bg-bg-secondary border border-border rounded-lg p-12 text-center">
        <Store size={32} class="mx-auto mb-3 opacity-20 text-text-muted" />
        <p class="text-sm text-text-muted">No marketplaces configured</p>
      </div>
    {/if}
  {/if}
</div>
