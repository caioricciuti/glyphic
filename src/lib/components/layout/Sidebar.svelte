<script lang="ts">
  import { onMount } from "svelte";
  import { NAV_ITEMS, getCurrentPage, navigateTo } from "$lib/stores/navigation.svelte";
  import { api } from "$lib/tauri/commands";
  import { calculateXP } from "$lib/utils/achievements";
  import { formatNumber } from "$lib/utils/format";
  import type { StatsCache, Settings } from "$lib/types";
  import {
    BarChart3, Settings as SettingsIcon, Zap, BookOpen, Brain,
    Server, Sparkles, Shield, Puzzle, GitBranch, TerminalSquare, Activity,
  } from "lucide-svelte";

  const currentPage = $derived(getCurrentPage());

  let stats = $state<StatsCache | null>(null);
  let settings = $state<Settings | null>(null);

  const xp = $derived(calculateXP(stats, settings));
  const xpPct = $derived(Math.min((xp.currentXP / xp.nextLevelXP) * 100, 100));

  const ICON_MAP: Record<string, typeof BarChart3> = {
    chart: BarChart3,
    gear: SettingsIcon,
    bolt: Zap,
    book: BookOpen,
    brain: Brain,
    server: Server,
    sparkles: Sparkles,
    shield: Shield,
    puzzle: Puzzle,
    git: GitBranch,
    terminal: TerminalSquare,
    analytics: Activity,
  };

  onMount(async () => {
    try {
      const [s, set] = await Promise.all([
        api.stats.computeLive(),
        api.settings.read("global"),
      ]);
      stats = s as StatsCache;
      settings = set;
    } catch {
      // Silently fail — sidebar XP is non-critical
    }
  });
</script>

<aside class="flex flex-col h-full w-60 bg-bg-secondary border-r border-border shrink-0">
  <!-- Logo -->
  <div class="flex items-center gap-2 px-4 py-4 border-b border-border">
    <div class="w-8 h-8 rounded-lg bg-accent flex items-center justify-center text-white font-bold text-base">
      G
    </div>
    <div>
      <h1 class="text-sm font-semibold text-text-primary">Glyphic</h1>
      <p class="text-xs text-text-muted">AI Config Manager</p>
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 py-2 overflow-y-auto">
    {#each NAV_ITEMS as item}
      {@const IconComponent = ICON_MAP[item.icon]}
      <button
        class="w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors
          {currentPage === item.id
            ? 'bg-accent-dim text-accent border-r-2 border-accent'
            : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'}"
        onclick={() => navigateTo(item.id)}
      >
        <span class="w-5 h-5 flex items-center justify-center">
          {#if IconComponent}
            <IconComponent size={18} />
          {/if}
        </span>
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- XP Bar -->
  <div class="p-4 border-t border-border">
    <div class="flex items-center justify-between text-xs text-text-muted mb-1">
      <span>Level {xp.level} — {xp.levelName}</span>
      <span>{formatNumber(xp.currentXP)} XP</span>
    </div>
    <div class="w-full h-2 bg-bg-tertiary rounded-full overflow-hidden">
      <div class="h-full bg-accent rounded-full transition-all duration-500" style="width: {xpPct}%"></div>
    </div>
  </div>
</aside>
