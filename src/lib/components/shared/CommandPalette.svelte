<script lang="ts">
  import { onMount } from "svelte";
  import { NAV_ITEMS, navigateTo, type Page } from "$lib/stores/navigation.svelte";
  import { toggleTheme, getTheme } from "$lib/stores/theme.svelte";
  import {
    BarChart3,
    Settings,
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
    History,
    Gauge,
    Sun,
    Moon,
    Search,
    Keyboard,
    Command as CommandIcon,
  } from "lucide-svelte";

  let open = $state(false);
  let query = $state("");
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | null>(null);

  const ICON_MAP: Record<string, typeof BarChart3> = {
    chart: BarChart3,
    gear: Settings,
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
    savings: Gauge,
    keybindings: Keyboard,
  };

  interface PaletteAction {
    id: string;
    label: string;
    description: string;
    icon: string;
    category: "navigate" | "action";
    handler: () => void;
  }

  const actions: PaletteAction[] = [
    ...NAV_ITEMS.map((item) => ({
      id: `nav-${item.id}`,
      label: item.label,
      description: `Go to ${item.label}`,
      icon: item.icon,
      category: "navigate" as const,
      handler: () => navigateTo(item.id),
    })),
    {
      id: "nav-keybindings",
      label: "Keybindings",
      description: "Go to Keybindings",
      icon: "keybindings",
      category: "navigate",
      handler: () => navigateTo("keybindings" as Page),
    },
    {
      id: "action-theme",
      label: "Toggle Theme",
      description: "Switch between dark and light mode",
      icon: "theme",
      category: "action",
      handler: toggleTheme,
    },
  ];

  const filtered = $derived(
    query.trim() === ""
      ? actions
      : actions.filter(
          (a) =>
            a.label.toLowerCase().includes(query.toLowerCase()) ||
            a.description.toLowerCase().includes(query.toLowerCase()),
        ),
  );

  $effect(() => {
    if (filtered.length > 0 && selectedIndex >= filtered.length) {
      selectedIndex = 0;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    // Cmd+K / Ctrl+K to open
    if ((e.metaKey || e.ctrlKey) && e.key === "k") {
      e.preventDefault();
      open = !open;
      if (open) {
        query = "";
        selectedIndex = 0;
        requestAnimationFrame(() => inputEl?.focus());
      }
      return;
    }

    // Keyboard shortcuts for pages (Cmd+1-9)
    if ((e.metaKey || e.ctrlKey) && e.key >= "1" && e.key <= "9") {
      const index = parseInt(e.key) - 1;
      if (index < NAV_ITEMS.length) {
        e.preventDefault();
        navigateTo(NAV_ITEMS[index].id);
      }
      return;
    }

    if (!open) return;

    if (e.key === "Escape") {
      e.preventDefault();
      open = false;
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % filtered.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + filtered.length) % filtered.length;
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (filtered[selectedIndex]) {
        filtered[selectedIndex].handler();
        open = false;
      }
    }
  }

  function handleSelect(action: PaletteAction) {
    action.handler();
    open = false;
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  });

  const currentTheme = $derived(getTheme());
</script>

{#if open}
  <!-- Backdrop -->
  <button
    class="fixed inset-0 bg-black/60 backdrop-blur-sm z-[100]"
    onclick={() => (open = false)}
    aria-label="Close command palette"
  ></button>

  <!-- Palette -->
  <div class="fixed top-[20%] left-1/2 -translate-x-1/2 w-[560px] bg-bg-secondary border border-border rounded-xl shadow-2xl z-[101] overflow-hidden">
    <!-- Search input -->
    <div class="flex items-center gap-3 px-4 py-3 border-b border-border">
      <Search size={18} class="text-text-muted shrink-0" />
      <input
        bind:this={inputEl}
        bind:value={query}
        type="text"
        placeholder="Search pages, actions..."
        class="flex-1 bg-transparent text-sm text-text-primary placeholder:text-text-muted focus:outline-none"
      />
      <kbd class="px-1.5 py-0.5 text-[10px] bg-bg-tertiary text-text-muted rounded border border-border font-mono">ESC</kbd>
    </div>

    <!-- Results -->
    <div class="max-h-[320px] overflow-y-auto py-1">
      {#if filtered.length === 0}
        <div class="px-4 py-8 text-center text-sm text-text-muted">
          No results for "{query}"
        </div>
      {:else}
        {#each filtered as action, i}
          {@const IconComponent = action.icon === "theme"
            ? (currentTheme === "dark" ? Sun : Moon)
            : ICON_MAP[action.icon]}
          <button
            class="w-full flex items-center gap-3 px-4 py-2.5 text-left transition-colors
              {i === selectedIndex ? 'bg-accent/15 text-accent' : 'text-text-secondary hover:bg-bg-hover'}"
            onclick={() => handleSelect(action)}
            onmouseenter={() => (selectedIndex = i)}
          >
            <span class="w-8 h-8 flex items-center justify-center rounded-lg {i === selectedIndex ? 'bg-accent/20' : 'bg-bg-tertiary'}">
              {#if IconComponent}
                <IconComponent size={16} />
              {/if}
            </span>
            <div class="flex-1 min-w-0">
              <p class="text-sm font-medium truncate">{action.label}</p>
              <p class="text-xs text-text-muted truncate">{action.description}</p>
            </div>
            <span class="text-[10px] px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted uppercase">
              {action.category === "navigate" ? "page" : "action"}
            </span>
          </button>
        {/each}
      {/if}
    </div>

    <!-- Footer -->
    <div class="flex items-center justify-between px-4 py-2 border-t border-border text-[10px] text-text-muted">
      <div class="flex items-center gap-3">
        <span class="flex items-center gap-1">
          <kbd class="px-1 py-0.5 bg-bg-tertiary rounded border border-border font-mono">↑↓</kbd>
          navigate
        </span>
        <span class="flex items-center gap-1">
          <kbd class="px-1 py-0.5 bg-bg-tertiary rounded border border-border font-mono">↵</kbd>
          select
        </span>
      </div>
      <span class="flex items-center gap-1">
        <CommandIcon size={10} />
        <span>+K to toggle</span>
      </span>
    </div>
  </div>
{/if}
