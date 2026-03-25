<script lang="ts">
  import { Flame } from "lucide-svelte";
  import { timeAgo } from "$lib/utils/format";

  interface Props {
    current: number;
    longest: number;
    lastActiveDate: string;
  }

  const { current, longest, lastActiveDate }: Props = $props();
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4">
  <h3 class="text-sm font-medium text-text-secondary mb-3">Streak</h3>
  <div class="flex items-baseline gap-2">
    {#if current > 0}
      <Flame size={24} class="text-accent" />
    {/if}
    <p class="text-4xl font-bold text-accent">{current}</p>
    <p class="text-sm text-text-muted">day{current !== 1 ? "s" : ""}</p>
  </div>
  <div class="mt-2 space-y-1">
    <p class="text-xs text-text-muted">
      Longest: <span class="text-text-secondary">{longest} day{longest !== 1 ? "s" : ""}</span>
    </p>
    {#if lastActiveDate}
      <p class="text-xs text-text-muted">
        Last active: <span class="text-text-secondary">{timeAgo(lastActiveDate)}</span>
      </p>
    {/if}
  </div>
</div>
