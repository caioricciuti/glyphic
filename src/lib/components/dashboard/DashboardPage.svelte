<script lang="ts">
</script>

<div class="p-6 space-y-6 overflow-y-auto h-full">
  <!-- Stats Row -->
  <div class="grid grid-cols-4 gap-4">
    {#each [
      { label: "Total Sessions", value: "—", color: "accent" },
      { label: "Messages Sent", value: "—", color: "success" },
      { label: "Tool Calls", value: "—", color: "warning" },
      { label: "Days Active", value: "—", color: "info" },
    ] as stat}
      <div class="bg-bg-secondary border border-border rounded-lg p-4">
        <p class="text-xs text-text-muted uppercase tracking-wider">{stat.label}</p>
        <p class="text-2xl font-bold mt-1 text-text-primary">{stat.value}</p>
      </div>
    {/each}
  </div>

  <div class="grid grid-cols-3 gap-4">
    <!-- Streak -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <h3 class="text-sm font-medium text-text-secondary mb-3">Current Streak</h3>
      <p class="text-4xl font-bold text-accent">—</p>
      <p class="text-xs text-text-muted mt-1">days</p>
    </div>

    <!-- Config Completeness -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <h3 class="text-sm font-medium text-text-secondary mb-3">Config Completeness</h3>
      <div class="flex items-center justify-center">
        <div class="relative w-24 h-24">
          <svg class="w-24 h-24 -rotate-90" viewBox="0 0 36 36">
            <path
              d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
              fill="none"
              stroke="var(--color-bg-tertiary)"
              stroke-width="3"
            />
            <path
              d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
              fill="none"
              stroke="var(--color-accent)"
              stroke-width="3"
              stroke-dasharray="0, 100"
            />
          </svg>
          <span class="absolute inset-0 flex items-center justify-center text-lg font-bold text-text-primary">0%</span>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <h3 class="text-sm font-medium text-text-secondary mb-3">Quick Actions</h3>
      <div class="space-y-2 text-sm text-text-muted">
        <p>Connect to Rust backend to populate dashboard</p>
      </div>
    </div>
  </div>

  <!-- Activity Heatmap Placeholder -->
  <div class="bg-bg-secondary border border-border rounded-lg p-4">
    <h3 class="text-sm font-medium text-text-secondary mb-3">Activity</h3>
    <div class="flex gap-1 flex-wrap">
      {#each Array(52 * 7) as _}
        <div class="w-3 h-3 rounded-sm bg-bg-tertiary"></div>
      {/each}
    </div>
  </div>

  <!-- Achievements -->
  <div class="bg-bg-secondary border border-border rounded-lg p-4">
    <h3 class="text-sm font-medium text-text-secondary mb-3">Achievements</h3>
    <p class="text-sm text-text-muted">Unlock achievements by configuring Claude Code features</p>
  </div>
</div>
