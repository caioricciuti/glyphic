<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type {
    OptimizerStatus,
    SavingsData,
    DiscoverResult,
    FilterRules,
  } from "$lib/types";
  import {
    Gauge,
    Power,
    TrendingUp,
    Zap,
    Hash,
    Search,
    FileCode,
    ToggleLeft,
    ToggleRight,
    RefreshCw,
    Save,
    Check,
    AlertTriangle,
    ChevronDown,
    ChevronUp,
  } from "lucide-svelte";

  // ── State ──────────────────────────────────────────────────────────
  let status = $state<OptimizerStatus | null>(null);
  let savings = $state<SavingsData | null>(null);
  let discover = $state<DiscoverResult | null>(null);
  let filters = $state<FilterRules | null>(null);

  let loading = $state(true);
  let discoverLoading = $state(false);
  let toggling = $state(false);
  let savingFilters = $state(false);
  let filtersSaved = $state(false);
  let filterContent = $state("");
  let activeTab = $state<"overview" | "breakdown" | "discover" | "filters">(
    "overview",
  );
  let period = $state<"daily" | "weekly" | "monthly">("daily");

  // Chart tooltip
  let chartTooltip = $state<{
    x: number;
    y: number;
    label: string;
    value: string;
  } | null>(null);

  // Discover expanded state
  let showAllOpportunities = $state(false);

  // ── Derived ────────────────────────────────────────────────────────

  const isEnabled = $derived(status?.enabled ?? false);

  const maxDailySaved = $derived(() => {
    if (!savings?.daily?.length) return 1;
    return Math.max(...savings.daily.map((d) => d.savedTokens), 1);
  });

  const chartData = $derived(() => {
    if (!savings?.daily?.length) return [];
    return savings.daily.slice(-30);
  });

  const visibleOpportunities = $derived(() => {
    if (!discover?.opportunities?.length) return [];
    return showAllOpportunities
      ? discover.opportunities
      : discover.opportunities.slice(0, 10);
  });

  // ── Helpers ────────────────────────────────────────────────────────

  function formatTokens(n: number): string {
    if (n >= 1e9) return `${(n / 1e9).toFixed(1)}B`;
    if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
    if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
    return n.toString();
  }

  function savingsColor(pct: number): string {
    if (pct >= 70) return "text-success";
    if (pct >= 40) return "text-warning";
    return "text-error";
  }

  function savingsBg(pct: number): string {
    if (pct >= 70) return "bg-success/20";
    if (pct >= 40) return "bg-warning/20";
    return "bg-error/20";
  }

  function showTooltip(e: MouseEvent, label: string, value: string) {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    chartTooltip = {
      x: rect.left + rect.width / 2,
      y: rect.top - 8,
      label,
      value,
    };
  }

  // ── Actions ────────────────────────────────────────────────────────

  async function toggleOptimizer() {
    toggling = true;
    try {
      if (isEnabled) {
        await api.tokenSavings.disable();
      } else {
        await api.tokenSavings.enable();
      }
      status = await api.tokenSavings.status();
    } catch (e) {
      console.error("Toggle failed:", e);
    } finally {
      toggling = false;
    }
  }

  async function refreshData() {
    loading = true;
    try {
      const [s, sv, f] = await Promise.all([
        api.tokenSavings.status(),
        api.tokenSavings.savings(period),
        api.tokenSavings.getFilters(),
      ]);
      status = s;
      savings = sv;
      filters = f;
      filterContent = f.rawContent;
    } catch (e) {
      console.error("Failed to load token savings data:", e);
    } finally {
      loading = false;
    }
  }

  async function loadDiscover() {
    if (discover) return; // Already loaded
    discoverLoading = true;
    try {
      discover = await api.tokenSavings.discover();
    } catch (e) {
      console.error("Failed to load discover data:", e);
    } finally {
      discoverLoading = false;
    }
  }

  async function handleSaveFilters() {
    savingFilters = true;
    filtersSaved = false;
    try {
      await api.tokenSavings.saveFilters(filterContent);
      filtersSaved = true;
      setTimeout(() => (filtersSaved = false), 2000);
    } catch (e) {
      console.error("Failed to save filters:", e);
    } finally {
      savingFilters = false;
    }
  }

  $effect(() => {
    // Re-fetch savings when period changes
    if (!loading && status) {
      api.tokenSavings.savings(period).then((sv) => (savings = sv));
    }
  });

  onMount(() => {
    refreshData();
  });
</script>

<div class="p-6 overflow-y-auto h-full space-y-4">
  {#if loading}
    <p class="text-sm text-text-muted">Loading token savings data...</p>
  {:else}
    <!-- Status Banner -->
    <div
      class="flex items-center justify-between bg-bg-secondary border border-border rounded-lg p-4"
    >
      <div class="flex items-center gap-3">
        <div
          class="w-10 h-10 rounded-lg flex items-center justify-center {isEnabled ? 'bg-success/20' : 'bg-bg-hover'}"
        >
          <Gauge
            size={20}
            class={isEnabled ? "text-success" : "text-text-muted"}
          />
        </div>
        <div>
          <h2 class="text-sm font-semibold text-text-primary">
            Token Optimizer
          </h2>
          <p class="text-xs text-text-muted">
            {#if isEnabled}
              Active — filtering command output to save tokens
            {:else}
              Disabled — enable to start saving tokens automatically
            {/if}
          </p>
        </div>
      </div>
      <div class="flex items-center gap-3">
        <button
          onclick={refreshData}
          class="p-2 rounded-md hover:bg-bg-hover text-text-muted transition-colors"
          title="Refresh"
        >
          <RefreshCw size={14} />
        </button>
        <button
          onclick={toggleOptimizer}
          disabled={toggling}
          class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors {isEnabled
            ? 'bg-error/10 text-error hover:bg-error/20'
            : 'bg-success/10 text-success hover:bg-success/20'}"
        >
          {#if toggling}
            <RefreshCw size={14} class="animate-spin" />
          {:else if isEnabled}
            <ToggleRight size={14} />
          {:else}
            <ToggleLeft size={14} />
          {/if}
          {toggling ? "..." : isEnabled ? "Disable" : "Enable"}
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex gap-1 bg-bg-secondary border border-border rounded-lg p-1">
      {#each [
        { id: "overview" as const, label: "Overview", icon: TrendingUp },
        { id: "breakdown" as const, label: "Breakdown", icon: Hash },
        { id: "discover" as const, label: "Discover", icon: Search },
        { id: "filters" as const, label: "Filters", icon: FileCode },
      ] as tab}
        {@const Icon = tab.icon}
        <button
          onclick={() => {
            activeTab = tab.id;
            if (tab.id === "discover") loadDiscover();
          }}
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors flex-1 justify-center
            {activeTab === tab.id ? 'bg-bg-hover text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
        >
          <Icon size={12} />
          {tab.label}
        </button>
      {/each}
    </div>

    <!-- Tab Content -->
    {#if activeTab === "overview"}
      <!-- ── Overview Tab ─────────────────────────────────── -->
      {#if !isEnabled && (!savings || savings.summary.totalCommands === 0)}
        <!-- Empty state when not enabled -->
        <div
          class="bg-bg-secondary border border-border rounded-lg p-8 text-center"
        >
          <div
            class="w-16 h-16 rounded-full bg-accent/10 flex items-center justify-center mx-auto mb-4"
          >
            <Zap size={28} class="text-accent" />
          </div>
          <h3 class="text-lg font-semibold text-text-primary mb-2">
            Save 60-90% of tokens on command output
          </h3>
          <p class="text-sm text-text-muted max-w-md mx-auto mb-6">
            The Token Optimizer intercepts Claude Code shell commands and
            compresses their output using intelligent filters. Less noise, same
            information, massive token savings.
          </p>
          <div class="grid grid-cols-3 gap-4 max-w-sm mx-auto mb-6">
            {#each [
              { label: "git status", savings: "~81%" },
              { label: "cargo test", savings: "~85%" },
              { label: "npm install", savings: "~90%" },
            ] as example}
              <div class="bg-bg-hover rounded-lg p-3">
                <p class="text-xs text-text-muted">{example.label}</p>
                <p class="text-lg font-bold text-success">{example.savings}</p>
              </div>
            {/each}
          </div>
          <button
            onclick={toggleOptimizer}
            disabled={toggling}
            class="px-6 py-2 bg-accent text-white rounded-lg text-sm font-medium hover:bg-accent/90 transition-colors"
          >
            {toggling ? "Enabling..." : "Enable Token Optimizer"}
          </button>
        </div>
      {:else}
        <!-- Hero Stats -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
          {#each [
            {
              label: "Tokens Saved",
              value: formatTokens(savings?.summary.totalSaved ?? 0),
              sub: `of ${formatTokens(savings?.summary.totalInputTokens ?? 0)} input`,
              icon: Zap,
              color: "text-accent",
            },
            {
              label: "Commands Filtered",
              value: (savings?.summary.totalCommands ?? 0).toLocaleString(),
              sub: `${savings?.topCommands.length ?? 0} unique commands`,
              icon: Hash,
              color: "text-info",
            },
            {
              label: "Avg Savings",
              value: `${(savings?.summary.avgSavingsPct ?? 0).toFixed(1)}%`,
              sub: savings?.summary.avgSavingsPct
                ? savings.summary.avgSavingsPct >= 70
                  ? "Excellent efficiency"
                  : savings.summary.avgSavingsPct >= 40
                    ? "Good efficiency"
                    : "Improving"
                : "",
              icon: TrendingUp,
              color: savingsColor(savings?.summary.avgSavingsPct ?? 0),
            },
            {
              label: "Status",
              value: isEnabled ? "Active" : "Paused",
              sub: status?.sidecarVersion ?? "",
              icon: Power,
              color: isEnabled ? "text-success" : "text-text-muted",
            },
          ] as card}
            {@const Icon = card.icon}
            <div class="bg-bg-secondary border border-border rounded-lg p-4">
              <div class="flex items-center gap-2 mb-2">
                <Icon size={14} class="text-text-muted" />
                <p class="text-[10px] text-text-muted uppercase tracking-wider">
                  {card.label}
                </p>
              </div>
              <p class="text-2xl font-bold {card.color}">{card.value}</p>
              {#if card.sub}
                <p class="text-xs text-text-muted mt-1">{card.sub}</p>
              {/if}
            </div>
          {/each}
        </div>

        <!-- Savings Gauge -->
        {#if savings && savings.summary.totalCommands > 0}
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <div class="flex items-center justify-between mb-3">
              <div class="flex items-center gap-2">
                <Gauge size={14} class="text-text-muted" />
                <h3 class="text-sm font-medium text-text-secondary">
                  Efficiency Meter
                </h3>
              </div>
              <span
                class="text-xs font-mono {savingsColor(savings.summary.avgSavingsPct)}"
              >
                {savings.summary.avgSavingsPct.toFixed(1)}%
              </span>
            </div>
            <div class="w-full h-3 bg-bg-hover rounded-full overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-500 {savings.summary.avgSavingsPct >= 70
                  ? 'bg-success'
                  : savings.summary.avgSavingsPct >= 40
                    ? 'bg-warning'
                    : 'bg-error'}"
                style="width: {Math.min(savings.summary.avgSavingsPct, 100)}%"
              ></div>
            </div>
            <div class="flex justify-between mt-1">
              <span class="text-[10px] text-text-muted">0%</span>
              <span class="text-[10px] text-text-muted"
                >{formatTokens(savings.summary.totalInputTokens)} input → {formatTokens(savings.summary.totalOutputTokens)} output</span
              >
              <span class="text-[10px] text-text-muted">100%</span>
            </div>
          </div>
        {/if}

        <!-- Daily Savings Chart -->
        {#if chartData().length > 0}
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <div class="flex items-center gap-2 mb-4">
              <TrendingUp size={14} class="text-text-muted" />
              <h3 class="text-sm font-medium text-text-secondary">
                Daily Token Savings (Last 30 Days)
              </h3>
            </div>
            <div
              class="flex items-end gap-[2px] h-32"
              onmouseleave={() => (chartTooltip = null)}
            >
              {#each chartData() as day}
                {@const pct = (day.savedTokens / maxDailySaved()) * 100}
                <div
                  class="flex-1 rounded-t transition-all cursor-pointer hover:opacity-80 {day.savingsPct >= 70 ? 'bg-success/60' : day.savingsPct >= 40 ? 'bg-warning/60' : 'bg-accent/40'}"
                  style="height: {Math.max(pct, 2)}%"
                  onmouseenter={(e) =>
                    showTooltip(
                      e,
                      day.label,
                      `${formatTokens(day.savedTokens)} saved (${day.savingsPct.toFixed(0)}%) · ${day.commands} cmds`,
                    )}
                  role="presentation"
                ></div>
              {/each}
            </div>
            <div class="flex justify-between mt-1">
              <span class="text-[10px] text-text-muted"
                >{chartData()[0]?.label ?? ""}</span
              >
              <span class="text-[10px] text-text-muted"
                >{chartData()[chartData().length - 1]?.label ?? ""}</span
              >
            </div>
          </div>
        {/if}
      {/if}
    {:else if activeTab === "breakdown"}
      <!-- ── Breakdown Tab ────────────────────────────────── -->
      <!-- Period Toggle -->
      <div class="flex gap-1 bg-bg-secondary border border-border rounded-lg p-1 w-fit">
        {#each [
          { id: "daily" as const, label: "Daily" },
          { id: "weekly" as const, label: "Weekly" },
          { id: "monthly" as const, label: "Monthly" },
        ] as p}
          <button
            onclick={() => (period = p.id)}
            class="px-3 py-1 rounded-md text-xs font-medium transition-colors
              {period === p.id ? 'bg-bg-hover text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
          >
            {p.label}
          </button>
        {/each}
      </div>

      <!-- Top Commands Table -->
      {#if savings && savings.topCommands.length > 0}
        <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
          <div class="px-4 py-3 border-b border-border">
            <h3 class="text-sm font-medium text-text-secondary">
              Top Commands by Savings
            </h3>
          </div>
          <div class="overflow-x-auto">
            <table class="w-full text-sm">
              <thead>
                <tr class="border-b border-border text-text-muted">
                  <th class="text-left px-4 py-2 text-xs font-medium">Command</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Count</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Avg Savings</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Total Saved</th>
                </tr>
              </thead>
              <tbody>
                {#each savings.topCommands as cmd}
                  <tr class="border-b border-border/50 hover:bg-bg-hover transition-colors">
                    <td class="px-4 py-2 font-mono text-xs text-text-primary">
                      {cmd.command}
                    </td>
                    <td class="px-4 py-2 text-right text-text-muted">
                      {cmd.count.toLocaleString()}
                    </td>
                    <td class="px-4 py-2 text-right">
                      <span
                        class="inline-block px-2 py-0.5 rounded text-[10px] font-medium {savingsColor(cmd.avgSavingsPct)} {savingsBg(cmd.avgSavingsPct)}"
                      >
                        {cmd.avgSavingsPct.toFixed(0)}%
                      </span>
                    </td>
                    <td class="px-4 py-2 text-right font-mono text-text-primary">
                      {formatTokens(cmd.totalSaved)}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {:else}
        <div class="bg-bg-secondary border border-border rounded-lg p-8 text-center">
          <p class="text-sm text-text-muted">
            No savings data yet. {isEnabled
              ? "Run some Claude Code commands to see breakdowns."
              : "Enable the optimizer to start tracking."}
          </p>
        </div>
      {/if}

      <!-- Time Period Table -->
      {#if savings && savings.daily.length > 0}
        <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
          <div class="px-4 py-3 border-b border-border">
            <h3 class="text-sm font-medium text-text-secondary">
              {period === "daily"
                ? "Daily"
                : period === "weekly"
                  ? "Weekly"
                  : "Monthly"} Breakdown
            </h3>
          </div>
          <div class="overflow-x-auto max-h-64 overflow-y-auto">
            <table class="w-full text-sm">
              <thead class="sticky top-0 bg-bg-secondary">
                <tr class="border-b border-border text-text-muted">
                  <th class="text-left px-4 py-2 text-xs font-medium">Period</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Commands</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Input</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Output</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">Saved</th>
                  <th class="text-right px-4 py-2 text-xs font-medium">%</th>
                </tr>
              </thead>
              <tbody>
                {#each [...savings.daily].reverse() as bucket}
                  <tr class="border-b border-border/50 hover:bg-bg-hover transition-colors">
                    <td class="px-4 py-2 text-xs text-text-muted font-mono">
                      {bucket.label}
                    </td>
                    <td class="px-4 py-2 text-right text-text-muted">
                      {bucket.commands}
                    </td>
                    <td class="px-4 py-2 text-right text-text-muted font-mono">
                      {formatTokens(bucket.inputTokens)}
                    </td>
                    <td class="px-4 py-2 text-right text-text-muted font-mono">
                      {formatTokens(bucket.outputTokens)}
                    </td>
                    <td class="px-4 py-2 text-right font-mono text-text-primary">
                      {formatTokens(bucket.savedTokens)}
                    </td>
                    <td class="px-4 py-2 text-right">
                      <span
                        class="inline-block px-2 py-0.5 rounded text-[10px] font-medium {savingsColor(bucket.savingsPct)} {savingsBg(bucket.savingsPct)}"
                      >
                        {bucket.savingsPct.toFixed(0)}%
                      </span>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}
    {:else if activeTab === "discover"}
      <!-- ── Discover Tab ─────────────────────────────────── -->
      {#if discoverLoading}
        <div class="bg-bg-secondary border border-border rounded-lg p-8 text-center">
          <RefreshCw size={20} class="animate-spin mx-auto mb-2 text-text-muted" />
          <p class="text-sm text-text-muted">Analyzing session history...</p>
        </div>
      {:else if !discover}
        <div class="bg-bg-secondary border border-border rounded-lg p-8 text-center">
          <Search size={20} class="mx-auto mb-2 text-text-muted" />
          <p class="text-sm text-text-muted">Click to analyze your Claude Code sessions for optimization opportunities.</p>
          <button
            onclick={loadDiscover}
            class="mt-3 px-4 py-2 bg-accent/10 text-accent rounded-lg text-sm font-medium hover:bg-accent/20 transition-colors"
          >
            Analyze Sessions
          </button>
        </div>
      {:else if discover}
        <!-- Summary -->
        <div class="grid grid-cols-3 gap-3">
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <p class="text-[10px] text-text-muted uppercase tracking-wider mb-1">
              Sessions Scanned
            </p>
            <p class="text-2xl font-bold text-text-primary">
              {discover.sessionsScanned.toLocaleString()}
            </p>
          </div>
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <p class="text-[10px] text-text-muted uppercase tracking-wider mb-1">
              Commands Found
            </p>
            <p class="text-2xl font-bold text-text-primary">
              {discover.totalCommands.toLocaleString()}
            </p>
          </div>
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <p class="text-[10px] text-text-muted uppercase tracking-wider mb-1">
              Potential Savings
            </p>
            <p class="text-2xl font-bold text-accent">
              {formatTokens(discover.totalPotentialSavings)}
            </p>
            <p class="text-[10px] text-text-muted">tokens</p>
          </div>
        </div>

        <!-- Opportunities Table -->
        {#if discover.opportunities.length > 0}
          <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
            <div class="px-4 py-3 border-b border-border flex items-center justify-between">
              <h3 class="text-sm font-medium text-text-secondary">
                Optimization Opportunities
              </h3>
              <span class="text-xs text-text-muted">
                {discover.opportunities.filter((o) => o.hasFilter).length} filterable
              </span>
            </div>
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="border-b border-border text-text-muted">
                    <th class="text-left px-4 py-2 text-xs font-medium">Command</th>
                    <th class="text-left px-4 py-2 text-xs font-medium">Category</th>
                    <th class="text-right px-4 py-2 text-xs font-medium">Count</th>
                    <th class="text-right px-4 py-2 text-xs font-medium">Est. Savings</th>
                    <th class="text-center px-4 py-2 text-xs font-medium">Filter</th>
                  </tr>
                </thead>
                <tbody>
                  {#each visibleOpportunities() as opp}
                    <tr class="border-b border-border/50 hover:bg-bg-hover transition-colors">
                      <td class="px-4 py-2 font-mono text-xs text-text-primary">
                        {opp.command}
                      </td>
                      <td class="px-4 py-2 text-xs text-text-muted">
                        {opp.category}
                      </td>
                      <td class="px-4 py-2 text-right text-text-muted">
                        {opp.count.toLocaleString()}
                      </td>
                      <td class="px-4 py-2 text-right font-mono text-accent">
                        {formatTokens(opp.estimatedSavingsTokens)}
                      </td>
                      <td class="px-4 py-2 text-center">
                        {#if opp.hasFilter}
                          <span class="inline-block px-2 py-0.5 rounded text-[10px] font-medium text-success bg-success/20">
                            Available
                          </span>
                        {:else}
                          <span class="inline-block px-2 py-0.5 rounded text-[10px] font-medium text-text-muted bg-bg-hover">
                            None
                          </span>
                        {/if}
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
            {#if discover.opportunities.length > 10}
              <div class="px-4 py-2 border-t border-border">
                <button
                  onclick={() => (showAllOpportunities = !showAllOpportunities)}
                  class="flex items-center gap-1 text-xs text-accent hover:underline"
                >
                  {#if showAllOpportunities}
                    <ChevronUp size={12} />
                    Show less
                  {:else}
                    <ChevronDown size={12} />
                    Show all {discover.opportunities.length} commands
                  {/if}
                </button>
              </div>
            {/if}
          </div>
        {:else}
          <div class="bg-bg-secondary border border-border rounded-lg p-8 text-center">
            <p class="text-sm text-text-muted">
              No session data found. Use Claude Code to generate session history,
              then check back for optimization opportunities.
            </p>
          </div>
        {/if}
      {/if}
    {:else if activeTab === "filters"}
      <!-- ── Filters Tab ──────────────────────────────────── -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <!-- Built-in Filters -->
        <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
          <div class="px-4 py-3 border-b border-border">
            <h3 class="text-sm font-medium text-text-secondary">
              Built-in Filters
            </h3>
            <p class="text-[10px] text-text-muted mt-0.5">
              {filters?.builtinCount ?? 0} active
            </p>
          </div>
          <div class="p-2 space-y-0.5 max-h-96 overflow-y-auto">
            {#each [
              "git status",
              "git log",
              "git diff",
              "git show",
              "ls",
              "tree",
              "find",
              "grep / rg",
              "npm test / bun test",
              "npm install",
              "cargo test",
              "cargo build",
              "curl",
              "docker ps",
              "cat",
            ] as name}
              <div
                class="flex items-center gap-2 px-3 py-1.5 rounded text-xs"
              >
                <Check size={10} class="text-success shrink-0" />
                <span class="text-text-secondary font-mono">{name}</span>
              </div>
            {/each}
          </div>
        </div>

        <!-- Custom Filters Editor -->
        <div class="lg:col-span-2 bg-bg-secondary border border-border rounded-lg overflow-hidden">
          <div class="px-4 py-3 border-b border-border flex items-center justify-between">
            <div>
              <h3 class="text-sm font-medium text-text-secondary">
                Custom Filters
              </h3>
              <p class="text-[10px] text-text-muted mt-0.5">
                {filters?.filterCount ?? 0} custom · {filters?.path ?? ""}
              </p>
            </div>
            <button
              onclick={handleSaveFilters}
              disabled={savingFilters}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-colors
                {filtersSaved ? 'bg-success/10 text-success' : 'bg-accent/10 text-accent hover:bg-accent/20'}"
            >
              {#if filtersSaved}
                <Check size={12} />
                Saved
              {:else}
                <Save size={12} />
                {savingFilters ? "Saving..." : "Save"}
              {/if}
            </button>
          </div>
          <div class="p-0">
            <textarea
              bind:value={filterContent}
              class="w-full h-80 bg-transparent text-text-primary font-mono text-xs p-4 resize-none focus:outline-none"
              spellcheck="false"
              placeholder="# Add custom TOML filter rules here..."
            ></textarea>
          </div>
          {#if !isEnabled}
            <div class="px-4 py-2 border-t border-border bg-warning/5">
              <div class="flex items-center gap-2">
                <AlertTriangle size={12} class="text-warning" />
                <p class="text-[10px] text-warning">
                  Optimizer is disabled. Enable it for filters to take effect.
                </p>
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<!-- Chart Tooltip -->
{#if chartTooltip}
  <div
    class="fixed z-50 pointer-events-none px-2 py-1 bg-bg-primary border border-border rounded shadow-lg text-xs"
    style="left: {chartTooltip.x}px; top: {chartTooltip.y}px; transform: translate(-50%, -100%)"
  >
    <p class="text-text-muted">{chartTooltip.label}</p>
    <p class="text-text-primary font-medium">{chartTooltip.value}</p>
  </div>
{/if}
