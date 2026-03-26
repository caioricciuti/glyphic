<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import { formatNumber } from "$lib/utils/format";
  import { calculateStreak, getDaysActive } from "$lib/utils/streaks";
  import { calculateXP, evaluateAchievements } from "$lib/utils/achievements";
  import { navigateTo } from "$lib/stores/navigation.svelte";
  import type { StatsCache, Settings, DailyActivity } from "$lib/types";
  import StatsOverview from "./StatsOverview.svelte";
  import StreakCard from "./StreakCard.svelte";
  import ConfigCompletenessRing from "./ConfigCompletenessRing.svelte";
  import ActivityHeatmap from "./ActivityHeatmap.svelte";
  import AchievementGrid from "./AchievementGrid.svelte";
  import SessionMonitor from "$lib/components/sessions/SessionMonitor.svelte";
  import type { CostSummary } from "$lib/tauri/commands";
  import { DollarSign, TrendingUp } from "lucide-svelte";

  let stats = $state<StatsCache | null>(null);
  let settings = $state<Settings | null>(null);
  let costSummary = $state<CostSummary | null>(null);
  let loading = $state(true);
  const totalMessages = $derived(stats?.totalMessages ?? 0);
  const totalSessions = $derived(stats?.totalSessions ?? 0);
  const totalToolCalls = $derived(
    stats?.dailyActivity?.reduce((sum: number, d: DailyActivity) => sum + d.toolCallCount, 0) ?? 0,
  );
  const daysActive = $derived(getDaysActive(stats?.dailyActivity ?? []));
  const streak = $derived(calculateStreak(stats?.dailyActivity ?? []));
  const xp = $derived(calculateXP(stats, settings));
  const achievements = $derived(evaluateAchievements(stats, settings, streak.current));

  onMount(async () => {
    try {
      const [s, set, cost] = await Promise.all([
        api.stats.computeLive(),
        api.settings.read("global"),
        api.budget.getCostSummary(),
      ]);
      stats = s as StatsCache;
      settings = set;
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
    <!-- Level Banner -->
    <div class="bg-gradient-to-r from-accent/20 to-accent/5 border border-accent/30 rounded-lg p-4 flex items-center justify-between">
      <div>
        <p class="text-xs text-text-muted uppercase tracking-wider">Level {xp.level}</p>
        <p class="text-xl font-bold text-accent">{xp.levelName}</p>
      </div>
      <div class="text-right">
        <p class="text-sm text-text-secondary">{formatNumber(xp.currentXP)} / {formatNumber(xp.nextLevelXP)} XP</p>
        <div class="w-48 h-2 bg-bg-tertiary rounded-full mt-1 overflow-hidden">
          <div
            class="h-full bg-accent rounded-full transition-all duration-1000"
            style="width: {Math.min((xp.currentXP / xp.nextLevelXP) * 100, 100)}%"
          ></div>
        </div>
      </div>
    </div>

    <!-- Stats Row -->
    <StatsOverview
      {totalSessions}
      {totalMessages}
      {totalToolCalls}
      {daysActive}
    />

    <div class="grid grid-cols-3 gap-4">
      <StreakCard current={streak.current} longest={streak.longest} lastActiveDate={streak.lastActiveDate} />
      <ConfigCompletenessRing {settings} />

      <!-- Cost Overview -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <h3 class="text-sm font-medium text-text-secondary mb-3 flex items-center gap-1.5">
          <DollarSign size={14} />
          Cost Overview
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
          </div>
        {:else}
          <p class="text-xs text-text-muted">Loading...</p>
        {/if}
      </div>
    </div>

    <!-- Activity Heatmap -->
    <ActivityHeatmap dailyActivity={stats?.dailyActivity ?? []} />

    <!-- Achievements -->
    <!-- Session Monitor + Achievements -->
    <div class="grid grid-cols-2 gap-4">
      <SessionMonitor />
      <AchievementGrid {achievements} />
    </div>
  {/if}
</div>
