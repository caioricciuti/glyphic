<script lang="ts">
  import { onMount } from "svelte";
  import { navigateTo } from "$lib/stores/navigation.svelte";
  import {
    Rocket,
    Settings,
    BookOpen,
    Server,
    TerminalSquare,
    Sparkles,
    ArrowRight,
    Check,
    X,
  } from "lucide-svelte";
  import logoUrl from "$lib/assets/logo.png";

  let visible = $state(false);
  let step = $state(0);

  interface OnboardingStep {
    icon: typeof Rocket;
    title: string;
    description: string;
    action: string;
    page: string;
  }

  const steps: OnboardingStep[] = [
    {
      icon: Settings,
      title: "Configure Settings",
      description: "Set your preferred model, effort level, and permissions. Glyphic reads and writes the same settings.json Claude Code uses.",
      action: "Open Settings",
      page: "settings",
    },
    {
      icon: BookOpen,
      title: "Write Instructions",
      description: "Create CLAUDE.md files that guide Claude Code. Set global preferences and per-project rules.",
      action: "Open Instructions",
      page: "instructions",
    },
    {
      icon: Server,
      title: "Add MCP Servers",
      description: "Connect external tools via the Model Context Protocol. Add GitHub, Slack, databases, and more.",
      action: "Open MCP Servers",
      page: "mcp",
    },
    {
      icon: Sparkles,
      title: "Create Skills",
      description: "Build reusable slash commands that extend Claude Code with custom workflows and templates.",
      action: "Open Skills",
      page: "skills",
    },
    {
      icon: TerminalSquare,
      title: "Launch Terminal",
      description: "Run Claude Code directly inside Glyphic with a fully embedded terminal. Multi-tab, persistent sessions.",
      action: "Open Terminal",
      page: "terminal",
    },
  ];

  function dismiss() {
    visible = false;
    localStorage.setItem("glyphic-onboarded", "true");
  }

  function goTo(page: string) {
    dismiss();
    navigateTo(page as Parameters<typeof navigateTo>[0]);
  }

  onMount(() => {
    const onboarded = localStorage.getItem("glyphic-onboarded");
    if (!onboarded) {
      visible = true;
    }
  });
</script>

{#if visible}
  <!-- Backdrop -->
  <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-[200] flex items-center justify-center">
    <div class="w-[640px] max-h-[85vh] bg-bg-secondary border border-border rounded-2xl shadow-2xl overflow-hidden flex flex-col">
      <!-- Header -->
      <div class="relative px-8 pt-8 pb-6 text-center border-b border-border">
        <button
          class="absolute top-4 right-4 p-1.5 rounded-lg hover:bg-bg-hover text-text-muted hover:text-text-primary transition-colors"
          onclick={dismiss}
          aria-label="Close"
        >
          <X size={18} />
        </button>

        <img src={logoUrl} alt="Glyphic" class="w-16 h-16 rounded-2xl mx-auto mb-4" />
        <h1 class="text-2xl font-bold text-text-primary">Welcome to Glyphic</h1>
        <p class="text-sm text-text-muted mt-2 max-w-md mx-auto">
          The desktop app for managing Claude Code. Everything runs locally
          — no accounts, no servers, no telemetry.
        </p>
      </div>

      <!-- Steps -->
      <div class="flex-1 overflow-y-auto px-8 py-6">
        <p class="text-xs font-medium text-text-muted uppercase tracking-wider mb-4">Get started</p>
        <div class="space-y-3">
          {#each steps as s, i}
            {@const StepIcon = s.icon}
            <div
              class="group flex items-center gap-4 p-4 rounded-xl border transition-all cursor-pointer
                {step === i
                  ? 'border-accent/40 bg-accent/5'
                  : 'border-border hover:border-border-light hover:bg-bg-hover'}"
              onclick={() => (step = i)}
              onkeydown={(e) => e.key === "Enter" && (step = i)}
              role="button"
              tabindex="0"
            >
              <span class="w-10 h-10 rounded-xl flex items-center justify-center shrink-0
                {step === i ? 'bg-accent/20 text-accent' : 'bg-bg-tertiary text-text-muted group-hover:text-text-secondary'}">
                <StepIcon size={20} />
              </span>
              <div class="flex-1 min-w-0">
                <p class="text-sm font-medium text-text-primary">{s.title}</p>
                <p class="text-xs text-text-muted mt-0.5 line-clamp-2">{s.description}</p>
              </div>
              <button
                class="shrink-0 flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium rounded-lg transition-colors
                  {step === i
                    ? 'bg-accent text-white hover:bg-accent-hover'
                    : 'bg-bg-tertiary text-text-muted hover:text-text-secondary'}"
                onclick={(e: MouseEvent) => { e.stopPropagation(); goTo(s.page); }}
              >
                {s.action}
                <ArrowRight size={12} />
              </button>
            </div>
          {/each}
        </div>
      </div>

      <!-- Footer -->
      <div class="px-8 py-4 border-t border-border flex items-center justify-between">
        <p class="text-xs text-text-muted">
          You can revisit this anytime from the About dialog
        </p>
        <button
          class="flex items-center gap-2 px-4 py-2 text-sm font-medium bg-accent hover:bg-accent-hover text-white rounded-lg transition-colors"
          onclick={dismiss}
        >
          <Check size={14} />
          Get Started
        </button>
      </div>
    </div>
  </div>
{/if}
