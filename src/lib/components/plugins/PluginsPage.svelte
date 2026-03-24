<script lang="ts">
  let activeTab = $state<"installed" | "marketplace">("installed");
</script>

<div class="p-6 overflow-y-auto h-full">
  <div class="flex gap-1 mb-6 bg-bg-tertiary rounded-lg p-1 w-fit">
    {#each [
      { id: "installed" as const, label: "Installed" },
      { id: "marketplace" as const, label: "Marketplace" },
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

  <div class="bg-bg-secondary border border-border rounded-lg p-4">
    <p class="text-sm text-text-muted">Connect to Rust backend to load {activeTab} plugins</p>
  </div>
</div>
