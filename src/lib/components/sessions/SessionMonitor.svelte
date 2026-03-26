<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { SessionSummary } from "$lib/tauri/commands";
  import { History, DollarSign, MessageSquare, Wrench, RefreshCw } from "lucide-svelte";
  import { navigateTo } from "$lib/stores/navigation.svelte";

  let activeSessions = $state<SessionSummary[]>([]);
  let loading = $state(true);
  let refreshTimer: ReturnType<typeof setInterval> | null = null;

  async function loadRecent() {
    try {
      const result = await api.sessions.list(5, 0);
      activeSessions = result.sessions;
    } catch {
      // silent
    } finally {
      loading = false;
    }
  }

  function formatTime(ts: string | null): string {
    if (!ts) return "";
    const diff = Date.now() - new Date(ts).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
  }

  onMount(() => {
    loadRecent();
    refreshTimer = setInterval(loadRecent, 60000); // refresh every minute
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
  });
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4">
  <div class="flex items-center justify-between mb-3">
    <h3 class="text-sm font-medium text-text-secondary flex items-center gap-2">
      <History size={14} />
      Recent Sessions
    </h3>
    <button class="p-1 text-text-muted hover:text-text-primary" onclick={loadRecent} aria-label="Refresh">
      <RefreshCw size={12} class={loading ? "animate-spin" : ""} />
    </button>
  </div>

  <div class="space-y-2">
    {#each activeSessions as session}
      <button
        class="w-full text-left px-3 py-2 bg-bg-tertiary rounded-md hover:bg-bg-hover transition-colors"
        onclick={() => navigateTo("sessions")}
      >
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium text-text-primary truncate">{session.project_path.split("/").pop()}</span>
          <span class="text-[10px] text-text-muted shrink-0">{formatTime(session.first_timestamp)}</span>
        </div>
        {#if session.first_message}
          <p class="text-[10px] text-text-muted truncate mt-0.5">"{session.first_message}"</p>
        {/if}
        <div class="flex gap-3 mt-1 text-[10px] text-text-muted">
          <span class="flex items-center gap-0.5"><MessageSquare size={8} />{session.user_messages}</span>
          <span class="flex items-center gap-0.5"><Wrench size={8} />{session.tool_calls}</span>
          <span class="flex items-center gap-0.5"><DollarSign size={8} />~${(session.entry_count * 0.01).toFixed(2)}</span>
        </div>
      </button>
    {/each}
  </div>
</div>
