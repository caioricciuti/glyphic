<script lang="ts">
  import { formatNumber } from "$lib/utils/format";

  interface Props {
    totalSessions: number;
    totalMessages: number;
    totalToolCalls: number;
    daysActive: number;
  }

  const { totalSessions, totalMessages, totalToolCalls, daysActive }: Props = $props();

  const cards = $derived([
    { label: "Total Sessions", value: formatNumber(totalSessions), color: "text-accent" },
    { label: "Messages Sent", value: formatNumber(totalMessages), color: "text-success" },
    { label: "Tool Calls", value: formatNumber(totalToolCalls), color: "text-warning" },
    { label: "Days Active", value: formatNumber(daysActive), color: "text-info" },
  ]);
</script>

<div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
  {#each cards as card}
    <div class="bg-bg-secondary border border-border rounded-lg p-4">
      <p class="text-xs text-text-muted uppercase tracking-wider">{card.label}</p>
      <p class="text-2xl font-bold mt-1 {card.color}">{card.value}</p>
    </div>
  {/each}
</div>
