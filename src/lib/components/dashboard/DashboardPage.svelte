<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import { formatNumber } from "$lib/utils/format";
  import { navigateTo } from "$lib/stores/navigation.svelte";
  import type { StatsCache, DailyActivity } from "$lib/types";
  import type { CostSummary, ProjectCost } from "$lib/tauri/commands";
  import StatsOverview from "./StatsOverview.svelte";
  import ActivityHeatmap from "./ActivityHeatmap.svelte";
  import QuickHealth from "./QuickHealth.svelte";
  import SessionMonitor from "$lib/components/sessions/SessionMonitor.svelte";
  import {
    DollarSign,
    TrendingUp,
    FolderOpen,
    BarChart3,
    Database,
  } from "lucide-svelte";

  let stats = $state<StatsCache | null>(null);
  let costSummary = $state<CostSummary | null>(null);
  let loading = $state(true);

  const totalMessages = $derived(stats?.totalMessages ?? 0);
  const totalSessions = $derived(stats?.totalSessions ?? 0);
  const totalToolCalls = $derived(
    stats?.dailyActivity?.reduce((sum: number, d: DailyActivity) => sum + d.toolCallCount, 0) ?? 0,
  );
  const daysActive = $derived(
    stats?.dailyActivity?.filter((d: DailyActivity) => d.messageCount > 0).length ?? 0,
  );

  const topProjects = $derived<ProjectCost[]>(
    (costSummary?.per_project_month ?? [])
      .filter((p) => p.cost > 0)
      .sort((a, b) => b.cost - a.cost)
      .slice(0, 5),
  );

  const last7Days = $derived(costSummary?.last_7_days ?? []);
  const weekTotal = $derived(last7Days.reduce((s, d) => s + d, 0));

  onMount(async () => {
    try {
      const [s, cost] = await Promise.all([
        api.stats.computeLive(),
        api.budget.getCostSummary(),
      ]);
      stats = s as StatsCache;
      costSummary = cost;
    } catch (e) {
      console.error("Failed to load dashboard data:", e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="p-6 space-y-6 overflow-y-auto h-full">
  {#if loading}
    <div class="flex items-center justify-center h-64">
      <p class="text-text-muted">Loading...</p>
    </div>
  {:else}
    <!-- Stats Row -->
    <StatsOverview
      {totalSessions}
      {totalMessages}
      {totalToolCalls}
      {daysActive}
    />

    <!-- Cost + Health + Spend Trend -->
    <div class="grid grid-cols-3 gap-4">
      <!-- Cost Overview -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <h3 class="text-sm font-medium text-text-secondary mb-3 flex items-center gap-1.5" title="Estimated API-equivalent cost based on token usage — not actual billing">
          <DollarSign size={14} />
          Cost Estimate
        </h3>
        {#if costSummary}
          <div class="space-y-3">
            <div>
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="text-text-muted">Today</span>
                <span class="font-medium {costSummary.daily_exceeded ? 'text-danger' : 'text-text-primary'}">${costSummary.today.toFixed(2)}</span>
              </div>
              {#if costSummary.daily_limit}
                <div class="h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
                  <div class="h-full {costSummary.daily_exceeded ? 'bg-danger' : 'bg-accent'} rounded-full" style="width: {Math.min((costSummary.today / costSummary.daily_limit) * 100, 100)}%"></div>
                </div>
              {/if}
            </div>
            <div>
              <div class="flex items-center justify-between text-xs mb-0.5">
                <span class="text-text-muted">This month</span>
                <span class="font-medium text-text-primary">${costSummary.this_month.toFixed(2)}</span>
              </div>
              {#if costSummary.monthly_limit}
                <div class="h-1.5 bg-bg-tertiary rounded-full overflow-hidden">
                  <div class="h-full {costSummary.monthly_exceeded ? 'bg-danger' : 'bg-accent'} rounded-full" style="width: {Math.min((costSummary.this_month / costSummary.monthly_limit) * 100, 100)}%"></div>
                </div>
              {/if}
            </div>
            <div class="flex items-center justify-between text-xs">
              <span class="text-text-muted flex items-center gap-1"><TrendingUp size={10} /> Projection</span>
              <span class="text-text-secondary">${costSummary.monthly_projection.toFixed(2)}/mo</span>
            </div>
            <button
              class="w-full text-xs text-accent hover:text-accent-hover py-1 transition-colors"
              onclick={() => navigateTo("analytics")}
            >
              View full analytics →
            </button>
            <p class="text-[10px] text-text-muted/60 leading-tight">Based on API token rates, not subscription billing</p>
          </div>
        {:else}
          <p class="text-xs text-text-muted">No cost data available</p>
        {/if}
      </div>

      <!-- Config Health -->
      <QuickHealth />

      <!-- 7-Day Spend -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <h3 class="text-sm font-medium text-text-secondary mb-3 flex items-center gap-1.5" title="Estimated API-equivalent cost — not actual billing">
          <BarChart3 size={14} />
          Last 7 Days
        </h3>
        {#if last7Days.length > 0}
          {@const maxDay = Math.max(...last7Days, 0.01)}
          <div class="flex items-end gap-1 h-16">
            {#each last7Days as day, i}
              {@const pct = (day / maxDay) * 100}
              {@const dayDate = new Date(Date.now() - (6 - i) * 86400000)}
              <div class="flex-1 flex flex-col items-center gap-0.5 group relative">
                <div
                  class="w-full bg-accent/70 rounded-sm transition-all hover:bg-accent min-h-[2px]"
                  style="height: {Math.max(pct, 3)}%"
                ></div>
                <span class="text-[9px] text-text-muted">
                  {dayDate.toLocaleDateString("en", { weekday: "narrow" })}
                </span>
                <div class="absolute -top-6 left-1/2 -translate-x-1/2 px-1.5 py-0.5 bg-bg-primary border border-border rounded text-[10px] text-text-primary opacity-0 group-hover:opacity-100 pointer-events-none whitespace-nowrap z-10">
                  ${day.toFixed(2)}
                </div>
              </div>
            {/each}
          </div>
          <p class="text-xs text-text-muted mt-2 text-right">
            Total: <span class="text-text-secondary font-medium">${weekTotal.toFixed(2)}</span>
          </p>
        {:else}
          <p class="text-xs text-text-muted">No data yet</p>
        {/if}
      </div>
    </div>

    <!-- Activity Heatmap -->
    <ActivityHeatmap dailyActivity={stats?.dailyActivity ?? []} />

    <!-- Sessions + Top Projects -->
    <div class="grid grid-cols-2 gap-4">
      <SessionMonitor />

      <!-- Top Projects by Cost -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <h3 class="text-sm font-medium text-text-secondary mb-3 flex items-center gap-1.5">
          <FolderOpen size={14} />
          Top Projects This Month
        </h3>
        {#if topProjects.length > 0}
          {@const maxCost = topProjects[0].cost}
          <div class="space-y-2">
            {#each topProjects as project}
              <div class="space-y-1">
                <div class="flex items-center justify-between text-xs">
                  <span class="text-text-primary truncate max-w-[60%]" title={project.project}>
                    {project.project.split("/").pop()}
                  </span>
                  <span class="text-text-secondary font-medium shrink-0">
                    ${project.cost.toFixed(2)}
                    <span class="text-text-muted font-normal ml-1">{formatNumber(project.messages)} msgs</span>
                  </span>
                </div>
                <div class="h-1 bg-bg-tertiary rounded-full overflow-hidden">
                  <div class="h-full bg-accent/60 rounded-full" style="width: {(project.cost / maxCost) * 100}%"></div>
                </div>
              </div>
            {/each}
          </div>
          <button
            class="w-full text-xs text-accent hover:text-accent-hover py-1 mt-2 transition-colors"
            onclick={() => navigateTo("analytics")}
          >
            View all in analytics →
          </button>
        {:else}
          <div class="flex flex-col items-center justify-center py-6 text-text-muted">
            <Database size={24} class="mb-2 opacity-50" />
            <p class="text-xs">No project cost data yet</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
