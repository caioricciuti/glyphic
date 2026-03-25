<script lang="ts">
  import type { Settings } from "$lib/types";

  interface Props {
    settings: Settings | null;
  }

  const { settings }: Props = $props();

  const completeness = $derived(() => {
    if (!settings) return 0;
    let score = 0;
    const total = 6;

    // Has non-default settings
    if (Object.keys(settings).length > 0) score++;
    // Has hooks
    if (settings.hooks && Object.keys(settings.hooks).length > 0) score++;
    // Has MCP servers
    if (settings.mcpServers && Object.keys(settings.mcpServers).length > 0) score++;
    // Has permissions configured
    if (settings.permissions) score++;
    // Has env vars
    if (settings.env && Object.keys(settings.env).length > 0) score++;
    // Has effort level set
    if (settings.effortLevel) score++;

    return Math.round((score / total) * 100);
  });

  const pct = $derived(completeness());
  const dashArray = $derived(`${pct}, 100`);
</script>

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
          stroke-dasharray={dashArray}
          class="transition-all duration-1000"
        />
      </svg>
      <span class="absolute inset-0 flex items-center justify-center text-lg font-bold text-text-primary">
        {pct}%
      </span>
    </div>
  </div>
</div>
