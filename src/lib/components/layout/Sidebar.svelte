<script lang="ts">
  import { onMount } from "svelte";
  import {
    NAV_ITEMS,
    getCurrentPage,
    navigateTo,
  } from "$lib/stores/navigation.svelte";
  import { api } from "$lib/tauri/commands";
  import { calculateXP } from "$lib/utils/achievements";
  import { formatNumber } from "$lib/utils/format";
  import type { StatsCache, Settings } from "$lib/types";
  import {
    BarChart3,
    Settings as SettingsIcon,
    Zap,
    BookOpen,
    Brain,
    Server,
    Sparkles,
    Shield,
    Puzzle,
    GitBranch,
    TerminalSquare,
    Activity,
    LayoutGrid,
    Sun,
    Moon,
    History,
  } from "lucide-svelte";
  import { getTheme, toggleTheme } from "$lib/stores/theme.svelte";
  import type { CostSummary } from "$lib/tauri/commands";
  import { ExternalLink, GitBranch as GithubIcon, X as XIcon } from "lucide-svelte";
  import logoUrl from "$lib/assets/logo.png";

  let showAbout = $state(false);

  const currentPage = $derived(getCurrentPage());

  let stats = $state<StatsCache | null>(null);
  let settings = $state<Settings | null>(null);
  let costSummary = $state<CostSummary | null>(null);

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
    pipelines: Activity,
    sessions: History,
    templates: LayoutGrid,
    terminal: TerminalSquare,
    analytics: Activity,
  };

  const currentTheme = $derived(getTheme());

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
    } catch {
      // Silently fail — sidebar XP is non-critical
    }
  });
</script>

<aside
  class="flex flex-col h-full w-60 bg-bg-secondary border-r border-border shrink-0"
>
  <!-- Logo (clickable → About) -->
  <button
    class="flex items-center gap-2 px-4 py-[13.5px] border-b border-border w-full hover:bg-bg-hover transition-colors text-left"
    onclick={() => (showAbout = true)}
  >
    <img src={logoUrl} alt="Glyphic" class="w-8 h-8 rounded-lg" />
    <div>
      <h1 class="text-sm font-semibold text-text-primary">Glyphic</h1>
      <p class="text-xs text-text-muted">AI Config Manager</p>
    </div>
  </button>

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

  <!-- Cost Widget -->
  {#if costSummary}
    <div class="px-4 py-2 border-t border-border">
      <div class="flex items-center justify-between text-xs mb-1">
        <span class="text-text-muted">Today</span>
        <span
          class="font-medium {costSummary.daily_exceeded
            ? 'text-danger'
            : 'text-text-primary'}"
        >
          ${costSummary.today.toFixed(2)}
        </span>
      </div>
      <div class="flex items-center justify-between text-xs mb-1.5">
        <span class="text-text-muted">This month</span>
        <span class="font-medium text-text-primary"
          >${costSummary.this_month.toFixed(2)}</span
        >
      </div>
      <!-- Mini sparkline -->
      {#if costSummary.last_7_days.length > 0}
        {@const max = Math.max(...costSummary.last_7_days, 0.01)}
        <div class="flex items-end gap-px h-4">
          {#each costSummary.last_7_days as val}
            <div
              class="flex-1 bg-accent/40 rounded-t-sm"
              style="height: {Math.max((val / max) * 100, 5)}%"
            ></div>
          {/each}
        </div>
      {/if}
      {#if costSummary.daily_exceeded || costSummary.monthly_exceeded}
        <p class="text-[10px] text-danger mt-1">Budget exceeded!</p>
      {/if}
    </div>
  {/if}

  <!-- Theme toggle -->
  <div class="px-4 py-2 border-t border-border">
    <button
      class="w-full flex items-center gap-3 px-3 py-2 text-sm text-text-secondary hover:bg-bg-hover hover:text-text-primary rounded-md transition-colors"
      onclick={toggleTheme}
    >
      {#if currentTheme === "dark"}
        <Sun size={16} />
        <span>Light Mode</span>
      {:else}
        <Moon size={16} />
        <span>Dark Mode</span>
      {/if}
    </button>
  </div>

  <!-- XP Bar -->
  <div class="px-4 py-3 border-t border-border">
    <div class="flex items-center justify-between text-xs text-text-muted mb-1">
      <span>Level {xp.level} — {xp.levelName}</span>
      <span>{formatNumber(xp.currentXP)} XP</span>
    </div>
    <div class="w-full h-2 bg-bg-tertiary rounded-full overflow-hidden">
      <div
        class="h-full bg-accent rounded-full transition-all duration-500"
        style="width: {xpPct}%"
      ></div>
    </div>
  </div>
</aside>

<!-- About Dialog -->
{#if showAbout}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button class="absolute inset-0 bg-black/50" onclick={() => (showAbout = false)} aria-label="Close"></button>
    <div class="relative bg-bg-secondary border border-border rounded-2xl shadow-2xl w-96 p-8 text-center z-10">
      <button class="absolute top-3 right-3 p-1 text-text-muted hover:text-text-primary" onclick={() => (showAbout = false)} aria-label="Close">
        <XIcon size={16} />
      </button>

      <img src={logoUrl} alt="Glyphic" class="w-20 h-20 rounded-2xl mx-auto mb-4" />
      <h2 class="text-xl font-bold text-text-primary">Glyphic</h2>
      <p class="text-sm text-text-muted mt-1">AI Config Manager for Claude Code</p>
      <p class="text-xs text-text-muted mt-1">Version 0.4.0</p>

      <div class="mt-6 space-y-2">
        <a
          href="https://caioricciuti.com"
          target="_blank"
          rel="noopener"
          class="flex items-center justify-center gap-2 text-sm text-accent hover:text-accent-hover transition-colors"
        >
          <ExternalLink size={14} />
          caioricciuti.com
        </a>
        <a
          href="https://github.com/caioricciuti/glyphic"
          target="_blank"
          rel="noopener"
          class="flex items-center justify-center gap-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          <GithubIcon size={14} />
          github.com/caioricciuti/glyphic
        </a>
      </div>

      <div class="mt-6 pt-4 border-t border-border">
        <p class="text-xs text-text-muted">
          Built by <a href="https://caioricciuti.com" target="_blank" rel="noopener" class="text-accent hover:underline">Caio Ricciuti</a>
        </p>
        <p class="text-[10px] text-text-muted mt-1">AGPL-3.0 License</p>
      </div>
    </div>
  </div>
{/if}
