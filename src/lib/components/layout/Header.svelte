<script lang="ts">
  import { getCurrentPage, type Page } from "$lib/stores/navigation.svelte";

  const PAGE_TITLES: Record<Page, string> = {
    dashboard: "Dashboard",
    settings: "Settings",
    hooks: "Hooks",
    instructions: "Instructions",
    memory: "Memory",
    mcp: "MCP Servers",
    skills: "Skills & Agents",
    rules: "Rules",
    plugins: "Plugins",
    git: "Git",
    terminal: "Terminal",
    analytics: "Analytics",
    pipelines: "Pipelines",
    sessions: "Sessions",
    templates: "Templates",
  };

  const PAGE_DESCRIPTIONS: Record<Page, string> = {
    dashboard: "Your Claude Code overview and stats",
    settings: "Global and project configuration",
    hooks: "Event-driven automation hooks",
    instructions: "CLAUDE.md instruction files",
    memory: "Project memory and context",
    mcp: "Model Context Protocol servers",
    skills: "Custom skills and agent definitions",
    rules: "Contextual rules for Claude",
    plugins: "Browse and manage plugins",
    git: "Version control for your projects",
    terminal: "Interactive Claude Code session",
    analytics: "Token usage, costs, and session traces",
    pipelines: "Visual workflow builder with Claude Code",
    sessions: "Replay and review past Claude Code sessions",
    templates: "Browse and add pre-built configurations",
  };

  const currentPage = $derived(getCurrentPage());
</script>

<header class="flex items-center justify-between px-6 py-[7px] border-b border-border bg-bg-secondary">
  <div>
    <h2 class="text-lg font-semibold text-text-primary">{PAGE_TITLES[currentPage]}</h2>
    <p class="text-sm text-text-muted">{PAGE_DESCRIPTIONS[currentPage]}</p>
  </div>
</header>
