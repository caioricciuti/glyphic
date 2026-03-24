<script lang="ts">
  let activeScope = $state<"global" | "project">("global");
</script>

<div class="p-6 overflow-y-auto h-full">
  <div class="flex gap-1 mb-6 bg-bg-tertiary rounded-lg p-1 w-fit">
    {#each [
      { id: "global" as const, label: "Global (~/.claude/CLAUDE.md)" },
      { id: "project" as const, label: "Project (./CLAUDE.md)" },
    ] as tab}
      <button
        class="px-4 py-1.5 text-sm rounded-md transition-colors
          {activeScope === tab.id
            ? 'bg-bg-secondary text-text-primary'
            : 'text-text-muted hover:text-text-secondary'}"
        onclick={() => (activeScope = tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="bg-bg-secondary border border-border rounded-lg p-4">
    <p class="text-sm text-text-muted">Connect to Rust backend to load CLAUDE.md</p>
  </div>
</div>
