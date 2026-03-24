<script lang="ts">
  let activeTab = $state<"global" | "project" | "local">("global");
</script>

<div class="p-6 overflow-y-auto h-full">
  <!-- Scope Tabs -->
  <div class="flex gap-1 mb-6 bg-bg-tertiary rounded-lg p-1 w-fit">
    {#each [
      { id: "global" as const, label: "Global" },
      { id: "project" as const, label: "Project" },
      { id: "local" as const, label: "Local" },
    ] as tab}
      <button
        class="px-4 py-1.5 text-sm rounded-md transition-colors
          {activeTab === tab.id
            ? 'bg-bg-secondary text-text-primary'
            : 'text-text-muted hover:text-text-secondary'}"
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="space-y-4">
    <p class="text-sm text-text-muted">
      {activeTab === "global" ? "~/.claude/settings.json" : activeTab === "project" ? ".claude/settings.json" : ".claude/settings.local.json"}
    </p>
    <p class="text-sm text-text-secondary">Connect to Rust backend to load settings</p>
  </div>
</div>
