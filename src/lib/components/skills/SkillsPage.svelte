<script lang="ts">
  let activeTab = $state<"skills" | "agents">("skills");
</script>

<div class="p-6 overflow-y-auto h-full">
  <div class="flex items-center justify-between mb-4">
    <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
      {#each [
        { id: "skills" as const, label: "Skills" },
        { id: "agents" as const, label: "Agents" },
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
    <button class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors">
      Create {activeTab === "skills" ? "Skill" : "Agent"}
    </button>
  </div>

  <div class="bg-bg-secondary border border-border rounded-lg p-4">
    <p class="text-sm text-text-muted">Connect to Rust backend to load {activeTab}</p>
  </div>
</div>
