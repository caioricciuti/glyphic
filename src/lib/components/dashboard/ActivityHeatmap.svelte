<script lang="ts">
  import type { DailyActivity } from "$lib/types";
  import { formatNumber } from "$lib/utils/format";

  interface Props {
    dailyActivity: DailyActivity[];
  }

  const { dailyActivity }: Props = $props();

  let tooltip = $state<{ x: number; y: number; date: string; count: number; sessions: number; tools: number } | null>(null);

  const activityMap = $derived(() => {
    const map = new Map<string, DailyActivity>();
    for (const d of dailyActivity) {
      map.set(d.date, d);
    }
    return map;
  });

  interface DayCell {
    date: string;
    count: number;
    dayOfWeek: number;
    empty: boolean;
  }

  const weeks = $derived(() => {
    const today = new Date();
    const map = activityMap();

    // Start from 52 weeks ago, aligned to Sunday
    const start = new Date(today);
    start.setDate(start.getDate() - 52 * 7 - start.getDay());

    const result: DayCell[][] = [];
    let currentWeek: DayCell[] = [];

    for (let i = 0; i <= 52 * 7 + today.getDay(); i++) {
      const d = new Date(start);
      d.setDate(start.getDate() + i);
      const dateStr = d.toISOString().split("T")[0];
      const dayOfWeek = d.getDay();

      if (dayOfWeek === 0 && currentWeek.length > 0) {
        // Pad to 7 days if needed
        while (currentWeek.length < 7) {
          currentWeek.push({ date: "", count: 0, dayOfWeek: currentWeek.length, empty: true });
        }
        result.push(currentWeek);
        currentWeek = [];
      }

      const activity = map.get(dateStr);
      currentWeek.push({
        date: dateStr,
        count: activity?.messageCount ?? 0,
        dayOfWeek,
        empty: false,
      });
    }

    // Pad final week
    while (currentWeek.length < 7) {
      currentWeek.push({ date: "", count: 0, dayOfWeek: currentWeek.length, empty: true });
    }
    if (currentWeek.length > 0) result.push(currentWeek);

    return result;
  });

  function getColor(count: number, empty: boolean): string {
    if (empty) return "bg-transparent";
    if (count === 0) return "bg-bg-tertiary";
    if (count < 100) return "bg-accent/20";
    if (count < 500) return "bg-accent/40";
    if (count < 1000) return "bg-accent/60";
    if (count < 3000) return "bg-accent/80";
    return "bg-accent";
  }

  function showTooltip(e: MouseEvent, day: DayCell) {
    if (day.empty) return;
    const map = activityMap();
    const activity = map.get(day.date);
    const rect = (e.target as HTMLElement).getBoundingClientRect();
    tooltip = {
      x: rect.left + rect.width / 2,
      y: rect.top - 8,
      date: day.date,
      count: day.count,
      sessions: activity?.sessionCount ?? 0,
      tools: activity?.toolCallCount ?? 0,
    };
  }

  const DAY_LABELS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4">
  <h3 class="text-sm font-medium text-text-secondary mb-3">Activity</h3>

  <div class="flex gap-0">
    <!-- Day labels -->
    <div class="flex flex-col gap-[2px] mr-1.5 shrink-0" style="width: 28px">
      {#each DAY_LABELS as label, i}
        <div class="h-[14px] text-[10px] text-text-muted leading-[14px]">
          {i % 2 === 1 ? label : ""}
        </div>
      {/each}
    </div>

    <!-- Grid -->
    <div class="flex gap-[2px] flex-1 min-w-0">
      {#each weeks() as week}
        <div class="flex flex-col gap-[2px] flex-1">
          {#each week as day}
            <div
              class="h-[14px] rounded-sm {getColor(day.count, day.empty)} {day.empty ? '' : 'cursor-pointer hover:ring-1 hover:ring-accent/50'}"
              role={day.empty ? undefined : "button"}
              tabindex={day.empty ? undefined : -1}
              onmouseenter={(e) => !day.empty && showTooltip(e, day)}
              onmouseleave={() => (tooltip = null)}
            ></div>
          {/each}
        </div>
      {/each}
    </div>
  </div>

  <!-- Legend -->
  <div class="flex items-center gap-2 mt-3 text-xs text-text-muted">
    <span>Less</span>
    <div class="flex gap-1">
      <div class="w-[11px] h-[11px] rounded-sm bg-bg-tertiary"></div>
      <div class="w-[11px] h-[11px] rounded-sm bg-accent/20"></div>
      <div class="w-[11px] h-[11px] rounded-sm bg-accent/40"></div>
      <div class="w-[11px] h-[11px] rounded-sm bg-accent/60"></div>
      <div class="w-[11px] h-[11px] rounded-sm bg-accent"></div>
    </div>
    <span>More</span>
    <span class="ml-auto">{dailyActivity.filter((d) => d.messageCount > 0).length} days tracked</span>
  </div>
</div>

<!-- Tooltip -->
{#if tooltip}
  <div
    class="fixed z-50 px-3 py-2 bg-bg-primary border border-border rounded-lg shadow-xl text-xs pointer-events-none"
    style="left: {tooltip.x}px; top: {tooltip.y}px; transform: translate(-50%, -100%)"
  >
    <p class="text-text-primary font-medium">
      {new Date(tooltip.date).toLocaleDateString("en", { weekday: "short", month: "short", day: "numeric", year: "numeric" })}
    </p>
    <p class="text-text-secondary">
      {formatNumber(tooltip.count)} messages · {tooltip.sessions} sessions · {formatNumber(tooltip.tools)} tools
    </p>
  </div>
{/if}
