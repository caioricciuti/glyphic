<script lang="ts">
  interface Props {
    used: number;
    max: number;
    label?: string;
  }

  const { used, max, label = "Context" }: Props = $props();

  const pct = $derived(Math.min((used / max) * 100, 100));
  const color = $derived(
    pct > 90 ? "bg-danger" : pct > 70 ? "bg-warning" : pct > 50 ? "bg-info" : "bg-success"
  );
  const textColor = $derived(
    pct > 90 ? "text-danger" : pct > 70 ? "text-warning" : "text-text-secondary"
  );

  function formatTokens(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(0)}K`;
    return n.toString();
  }
</script>

<div class="space-y-1">
  <div class="flex items-center justify-between text-[10px]">
    <span class="text-text-muted">{label}</span>
    <span class={textColor}>{formatTokens(used)} / {formatTokens(max)}</span>
  </div>
  <div class="h-2 bg-bg-tertiary rounded-full overflow-hidden">
    <div class="h-full {color} rounded-full transition-all duration-500" style="width: {pct}%"></div>
  </div>
  {#if pct > 80}
    <p class="text-[9px] {textColor}">{pct > 90 ? "Context nearly full — compaction imminent" : "Context filling up"}</p>
  {/if}
</div>
