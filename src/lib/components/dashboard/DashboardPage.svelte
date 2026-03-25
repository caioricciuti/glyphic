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

  let stats = $state<StatsCache | null>(null);
  let settings = $state<Settings | null>(null);
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
      const [s, set] = await Promise.all([
        api.stats.computeLive(),
        api.settings.read("global"),
      ]);
      stats = s as StatsCache;
      settings = set;
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

      <!-- Quick Actions -->
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <h3 class="text-sm font-medium text-text-secondary mb-3">Quick Actions</h3>
        <div class="space-y-2">
          {#each [
            { label: "Configure Hooks", page: "hooks" as const },
            { label: "Add MCP Server", page: "mcp" as const },
            { label: "Edit Instructions", page: "instructions" as const },
            { label: "Create Skill", page: "skills" as const },
          ] as action}
            <button
              class="w-full text-left px-3 py-2 text-sm text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-md transition-colors"
              onclick={() => navigateTo(action.page)}
            >
              {action.label}
            </button>
          {/each}
        </div>
      </div>
    </div>

    <!-- Activity Heatmap -->
    <ActivityHeatmap dailyActivity={stats?.dailyActivity ?? []} />

    <!-- Achievements -->
    <AchievementGrid {achievements} />
  {/if}
</div>
