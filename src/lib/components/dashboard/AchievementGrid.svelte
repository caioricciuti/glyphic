<script lang="ts">
  import type { Achievement } from "$lib/types";
  import {
    Settings, Zap, Server, Network, Shield, Variable,
    MessageSquare, MessageCircle, MessagesSquare, Rocket,
    Trophy, Wrench, Flame, CalendarDays, CalendarRange,
    CalendarCheck, Star, Award, Cog,
  } from "lucide-svelte";
  import type { ComponentType } from "svelte";

  interface Props {
    achievements: Achievement[];
  }

  const { achievements }: Props = $props();

  const unlockedCount = $derived(achievements.filter((a) => a.unlocked).length);

  const ICON_MAP: Record<string, ComponentType> = {
    Settings, Zap, Server, Network, Shield, Variable,
    MessageSquare, MessageCircle, MessagesSquare, Rocket,
    Trophy, Wrench, Flame, CalendarDays, CalendarRange,
    CalendarCheck, Star, Award, Cog,
  };

  function getIcon(name: string): ComponentType | null {
    return ICON_MAP[name] ?? null;
  }
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4">
  <h3 class="text-sm font-medium text-text-secondary mb-3">
    Achievements
    <span class="text-text-muted">({unlockedCount}/{achievements.length})</span>
  </h3>
  <div class="grid grid-cols-2 lg:grid-cols-3 gap-2">
    {#each achievements as achievement}
      {@const Icon = getIcon(achievement.icon)}
      <div
        class="flex items-center gap-3 p-3 rounded-lg border transition-colors
          {achievement.unlocked
            ? 'bg-accent/10 border-accent/30'
            : 'bg-bg-tertiary border-border opacity-40'}"
      >
        <div class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0
          {achievement.unlocked ? 'bg-accent/20 text-accent' : 'bg-bg-tertiary text-text-muted'}">
          {#if Icon}
            <Icon size={16} />
          {/if}
        </div>
        <div class="min-w-0">
          <p class="text-xs font-medium text-text-primary truncate">{achievement.name}</p>
          <p class="text-[10px] text-text-muted truncate">{achievement.description}</p>
        </div>
      </div>
    {/each}
  </div>
</div>
