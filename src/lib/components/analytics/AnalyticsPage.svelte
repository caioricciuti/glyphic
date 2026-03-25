<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import { formatNumber } from "$lib/utils/format";
  import { DollarSign, Coins, Clock, Calendar, TrendingUp, Cpu, Zap, Database } from "lucide-svelte";

  interface ModelUsage {
    inputTokens: number;
    outputTokens: number;
    cacheReadInputTokens: number;
    cacheCreationInputTokens: number;
    costUSD: number;
  }

  interface DailyModelTokens {
    date: string;
    tokensByModel: Record<string, number>;
  }

  interface FullStats {
    modelUsage?: Record<string, ModelUsage>;
    dailyModelTokens?: DailyModelTokens[];
    hourCounts?: Record<string, number>;
    totalSessions: number;
    totalMessages: number;
    longestSession?: { sessionId: string; duration: number; messageCount: number; timestamp: string };
    firstSessionDate?: string;
    dailyActivity?: { date: string; messageCount: number; sessionCount: number; toolCallCount: number }[];
  }

  let stats = $state<FullStats | null>(null);
  let loading = $state(true);

  // Tooltips
  let chartTooltip = $state<{ x: number; y: number; label: string; value: string } | null>(null);

  const PRICING: Record<string, { input: number; output: number; cacheRead: number; cacheWrite: number }> = {
    "claude-opus-4-6": { input: 15, output: 75, cacheRead: 1.5, cacheWrite: 18.75 },
    "claude-opus-4-5-20251101": { input: 15, output: 75, cacheRead: 1.5, cacheWrite: 18.75 },
    "claude-sonnet-4-6": { input: 3, output: 15, cacheRead: 0.3, cacheWrite: 3.75 },
  };
  const DEFAULT_PRICING = { input: 3, output: 15, cacheRead: 0.3, cacheWrite: 3.75 };

  function estimateCost(model: string, usage: ModelUsage): number {
    const p = PRICING[model] ?? DEFAULT_PRICING;
    return (usage.inputTokens / 1e6) * p.input + (usage.outputTokens / 1e6) * p.output +
      (usage.cacheReadInputTokens / 1e6) * p.cacheRead + (usage.cacheCreationInputTokens / 1e6) * p.cacheWrite;
  }

  function formatTokens(n: number): string {
    if (n >= 1e9) return `${(n / 1e9).toFixed(1)}B`;
    if (n >= 1e6) return `${(n / 1e6).toFixed(1)}M`;
    if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
    return n.toString();
  }

  function formatDuration(ms: number): string {
    const h = Math.floor(ms / 3600000);
    const m = Math.floor((ms % 3600000) / 60000);
    return h > 0 ? `${h}h ${m}m` : `${m}m`;
  }

  function showTooltip(e: MouseEvent, label: string, value: string) {
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    chartTooltip = { x: rect.left + rect.width / 2, y: rect.top - 8, label, value };
  }

  const totalCost = $derived(() => {
    if (!stats?.modelUsage) return 0;
    return Object.entries(stats.modelUsage).reduce((s, [m, u]) => s + estimateCost(m, u), 0);
  });

  const totalTokens = $derived(() => {
    if (!stats?.modelUsage) return 0;
    return Object.values(stats.modelUsage).reduce((s, u) => s + u.inputTokens + u.outputTokens + u.cacheReadInputTokens + u.cacheCreationInputTokens, 0);
  });

  const modelEntries = $derived(() => {
    if (!stats?.modelUsage) return [];
    return Object.entries(stats.modelUsage).map(([model, usage]) => ({
      model, ...usage,
      cost: estimateCost(model, usage),
      totalTokens: usage.inputTokens + usage.outputTokens + usage.cacheReadInputTokens + usage.cacheCreationInputTokens,
    })).sort((a, b) => b.cost - a.cost);
  });

  const hourlyData = $derived(() => {
    if (!stats?.hourCounts) return [];
    const max = Math.max(...Object.values(stats.hourCounts), 1);
    return Array.from({ length: 24 }, (_, h) => ({
      hour: h, count: stats!.hourCounts![h.toString()] ?? 0,
      pct: ((stats!.hourCounts![h.toString()] ?? 0) / max) * 100,
    }));
  });

  const peakHour = $derived(() => {
    const data = hourlyData();
    if (data.length === 0) return null;
    return data.reduce((max, d) => d.count > max.count ? d : max, data[0]);
  });

  const dailyTokenTrend = $derived(() => {
    if (!stats?.dailyModelTokens) return [];
    return stats.dailyModelTokens.slice(-30).map((d) => ({
      date: d.date,
      total: Object.values(d.tokensByModel).reduce((s, t) => s + t, 0),
    }));
  });

  const maxDailyTokens = $derived(() => Math.max(...dailyTokenTrend().map((d) => d.total), 1));

  const avgDailyTokens = $derived(() => {
    const data = dailyTokenTrend();
    if (data.length === 0) return 0;
    return data.reduce((s, d) => s + d.total, 0) / data.length;
  });

  // Cache stats
  const cacheTotalInput = $derived(modelEntries().reduce((s, e) => s + e.inputTokens, 0));
  const cacheTotalRead = $derived(modelEntries().reduce((s, e) => s + e.cacheReadInputTokens, 0));
  const cacheTotalWrite = $derived(modelEntries().reduce((s, e) => s + e.cacheCreationInputTokens, 0));
  const cacheTotal = $derived(cacheTotalInput + cacheTotalRead + cacheTotalWrite || 1);
  const cacheRatio = $derived((cacheTotalRead / cacheTotal) * 100);
  const cacheSavedCost = $derived(
    modelEntries().reduce((s, e) => {
      const p = PRICING[e.model] ?? DEFAULT_PRICING;
      return s + (e.cacheReadInputTokens / 1e6) * (p.input - p.cacheRead);
    }, 0),
  );

  // Cost per message
  const costPerMessage = $derived(() => {
    if (!stats?.totalMessages || stats.totalMessages === 0) return 0;
    return totalCost() / stats.totalMessages;
  });

  onMount(async () => {
    try { stats = (await api.stats.get()) as FullStats; }
    catch (e) { console.error("Failed:", e); }
    finally { loading = false; }
  });
</script>

<div class="p-6 overflow-y-auto h-full space-y-4">
  {#if loading}
    <p class="text-sm text-text-muted">Loading analytics...</p>
  {:else if !stats}
    <p class="text-sm text-text-muted">No analytics data available</p>
  {:else}
    <!-- Top Stats -->
    <div class="grid grid-cols-2 lg:grid-cols-4 gap-3">
      {#each [
        { label: "Estimated Cost", value: `$${totalCost().toFixed(2)}`, sub: `$${costPerMessage().toFixed(4)}/message`, icon: DollarSign, color: "text-accent" },
        { label: "Total Tokens", value: formatTokens(totalTokens()), sub: `avg ${formatTokens(avgDailyTokens())}/day`, icon: Coins, color: "text-info" },
        { label: "Longest Session", value: stats.longestSession ? formatDuration(stats.longestSession.duration) : "—", sub: stats.longestSession ? `${formatNumber(stats.longestSession.messageCount)} messages` : "", icon: Clock, color: "text-warning" },
        { label: "Active Since", value: stats.firstSessionDate ? new Date(stats.firstSessionDate).toLocaleDateString("en", { month: "short", year: "numeric" }) : "—", sub: stats.firstSessionDate ? `${Math.floor((Date.now() - new Date(stats.firstSessionDate).getTime()) / 86400000)} days ago` : "", icon: Calendar, color: "text-success" },
      ] as card}
        {@const Icon = card.icon}
        <div class="bg-bg-secondary border border-border rounded-lg p-4">
          <div class="flex items-center gap-2 mb-2">
            <Icon size={14} class="text-text-muted" />
            <p class="text-[10px] text-text-muted uppercase tracking-wider">{card.label}</p>
          </div>
          <p class="text-2xl font-bold {card.color}">{card.value}</p>
          {#if card.sub}
            <p class="text-xs text-text-muted mt-1">{card.sub}</p>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Model Usage -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 mb-4">
        <Cpu size={14} class="text-text-muted" />
        <h3 class="text-sm font-medium text-text-secondary">Model Usage</h3>
      </div>
      <div class="space-y-5">
        {#each modelEntries() as entry}
          {@const pct = (entry.totalTokens / (totalTokens() || 1)) * 100}
          <div>
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2">
                <span class="w-2.5 h-2.5 rounded-full {entry.model.includes('opus') ? 'bg-accent' : entry.model.includes('sonnet') ? 'bg-info' : 'bg-success'}"></span>
                <span class="text-sm font-mono text-text-primary">{entry.model}</span>
                <span class="text-xs text-text-muted">({pct.toFixed(1)}%)</span>
              </div>
              <span class="text-sm font-semibold text-accent">${entry.cost.toFixed(2)}</span>
            </div>

            <!-- Token breakdown -->
            <div class="grid grid-cols-4 gap-3">
              {#each [
                { label: "Input", value: entry.inputTokens, color: "bg-info" },
                { label: "Output", value: entry.outputTokens, color: "bg-success" },
                { label: "Cache Read", value: entry.cacheReadInputTokens, color: "bg-warning" },
                { label: "Cache Write", value: entry.cacheCreationInputTokens, color: "bg-accent" },
              ] as tok}
                <div>
                  <div class="flex justify-between text-[10px] text-text-muted mb-1">
                    <span>{tok.label}</span>
                    <span>{formatTokens(tok.value)}</span>
                  </div>
                  <div class="h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
                    <div class="h-full {tok.color} rounded-full transition-all" style="width: {Math.min((tok.value / (entry.totalTokens || 1)) * 100, 100)}%"></div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4">
      <!-- Daily Token Trend -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <TrendingUp size={14} class="text-text-muted" />
            <h3 class="text-sm font-medium text-text-secondary">Daily Tokens (30d)</h3>
          </div>
          <span class="text-xs text-text-muted">avg {formatTokens(avgDailyTokens())}/day</span>
        </div>
        <div class="flex items-end gap-[2px] h-36">
          {#each dailyTokenTrend() as day}
            {@const pct = (day.total / maxDailyTokens()) * 100}
            <div
              class="flex-1 bg-accent/40 rounded-t-sm hover:bg-accent transition-colors cursor-pointer min-w-[3px]"
              style="height: {Math.max(pct, 2)}%"
              role="button" tabindex="-1"
              onmouseenter={(e) => showTooltip(e, new Date(day.date).toLocaleDateString("en", { month: "short", day: "numeric" }), formatTokens(day.total) + " tokens")}
              onmouseleave={() => (chartTooltip = null)}
            ></div>
          {/each}
        </div>
        <div class="flex justify-between text-[10px] text-text-muted mt-1">
          <span>{dailyTokenTrend()[0]?.date?.slice(5) ?? ""}</span>
          <span>{dailyTokenTrend()[dailyTokenTrend().length - 1]?.date?.slice(5) ?? ""}</span>
        </div>
      </div>

      <!-- Hourly Activity -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <div class="flex items-center justify-between mb-3">
          <div class="flex items-center gap-2">
            <Zap size={14} class="text-text-muted" />
            <h3 class="text-sm font-medium text-text-secondary">Activity by Hour</h3>
          </div>
          {#if peakHour()}
            <span class="text-xs text-text-muted">Peak: {peakHour()!.hour}:00 ({peakHour()!.count} sessions)</span>
          {/if}
        </div>
        <div class="flex items-end gap-[2px] h-36">
          {#each hourlyData() as hour}
            <div
              class="flex-1 rounded-t-sm cursor-pointer min-w-[3px] transition-colors
                {hour.count > 0 ? 'bg-info/40 hover:bg-info' : 'bg-bg-tertiary hover:bg-bg-tertiary'}"
              style="height: {Math.max(hour.pct, 2)}%"
              role="button" tabindex="-1"
              onmouseenter={(e) => showTooltip(e, `${hour.hour}:00`, `${hour.count} sessions`)}
              onmouseleave={() => (chartTooltip = null)}
            ></div>
          {/each}
        </div>
        <div class="flex justify-between text-[10px] text-text-muted mt-1">
          <span>0:00</span>
          <span>6:00</span>
          <span>12:00</span>
          <span>18:00</span>
          <span>23:00</span>
        </div>
      </div>
    </div>

    <!-- Cache Efficiency -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <div class="flex items-center gap-2 mb-4">
        <Database size={14} class="text-text-muted" />
        <h3 class="text-sm font-medium text-text-secondary">Cache Efficiency</h3>
      </div>

      <div class="grid grid-cols-3 gap-6">
        <div>
          <p class="text-xs text-text-muted mb-1">Cache Hit Rate</p>
          <p class="text-2xl font-bold text-success">{cacheRatio.toFixed(1)}%</p>
          <p class="text-[10px] text-text-muted mt-1">{formatTokens(cacheTotalRead)} tokens from cache</p>
        </div>
        <div>
          <p class="text-xs text-text-muted mb-1">Total Cache Volume</p>
          <p class="text-2xl font-bold text-info">{formatTokens(cacheTotalRead + cacheTotalWrite)}</p>
          <p class="text-[10px] text-text-muted mt-1">{formatTokens(cacheTotalRead)} reads · {formatTokens(cacheTotalWrite)} writes</p>
        </div>
        <div>
          <p class="text-xs text-text-muted mb-1">Estimated Savings</p>
          <p class="text-2xl font-bold text-success">${cacheSavedCost.toFixed(2)}</p>
          <p class="text-[10px] text-text-muted mt-1">vs. full-price input tokens</p>
        </div>
      </div>

      <!-- Segmented bar -->
      <div class="mt-4">
        <div class="h-4 bg-bg-tertiary rounded-full overflow-hidden flex">
          <div
            class="bg-info h-full transition-all hover:brightness-110 cursor-pointer"
            style="width: {(cacheTotalInput / cacheTotal) * 100}%"
            role="button" tabindex="-1"
            onmouseenter={(e) => showTooltip(e, "Direct Input", formatTokens(cacheTotalInput))}
            onmouseleave={() => (chartTooltip = null)}
          ></div>
          <div
            class="bg-success h-full transition-all hover:brightness-110 cursor-pointer"
            style="width: {(cacheTotalRead / cacheTotal) * 100}%"
            role="button" tabindex="-1"
            onmouseenter={(e) => showTooltip(e, "Cache Reads", formatTokens(cacheTotalRead))}
            onmouseleave={() => (chartTooltip = null)}
          ></div>
          <div
            class="bg-accent h-full transition-all hover:brightness-110 cursor-pointer"
            style="width: {(cacheTotalWrite / cacheTotal) * 100}%"
            role="button" tabindex="-1"
            onmouseenter={(e) => showTooltip(e, "Cache Writes", formatTokens(cacheTotalWrite))}
            onmouseleave={() => (chartTooltip = null)}
          ></div>
        </div>
        <div class="flex gap-4 mt-2 text-xs text-text-muted">
          <span class="flex items-center gap-1.5"><span class="w-2.5 h-2.5 rounded-sm bg-info"></span> Input ({((cacheTotalInput / cacheTotal) * 100).toFixed(1)}%)</span>
          <span class="flex items-center gap-1.5"><span class="w-2.5 h-2.5 rounded-sm bg-success"></span> Cache Reads ({((cacheTotalRead / cacheTotal) * 100).toFixed(1)}%)</span>
          <span class="flex items-center gap-1.5"><span class="w-2.5 h-2.5 rounded-sm bg-accent"></span> Cache Writes ({((cacheTotalWrite / cacheTotal) * 100).toFixed(1)}%)</span>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Chart tooltip -->
{#if chartTooltip}
  <div
    class="fixed z-50 px-3 py-2 bg-bg-primary border border-border rounded-lg shadow-xl text-xs pointer-events-none"
    style="left: {chartTooltip.x}px; top: {chartTooltip.y}px; transform: translate(-50%, -100%)"
  >
    <p class="text-text-primary font-medium">{chartTooltip.label}</p>
    <p class="text-text-secondary">{chartTooltip.value}</p>
  </div>
{/if}
