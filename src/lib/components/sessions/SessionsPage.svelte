<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { SessionSummary, SessionEvent } from "$lib/tauri/commands";
  import { marked } from "marked";
  import {
    History, Search, Play, Pause, ChevronDown, ChevronRight,
    User, Bot, Terminal, Pencil, Eye, Folder, Code,
    Wrench, Globe, Zap, Brain, Clock, MessageSquare, Hash,
  } from "lucide-svelte";

  let sessions = $state<SessionSummary[]>([]);
  let loading = $state(true);
  let searchQuery = $state("");
  let selectedSession = $state<SessionSummary | null>(null);
  let events = $state<SessionEvent[]>([]);
  let loadingEvents = $state(false);

  // Replay controls
  let playing = $state(false);
  let visibleCount = $state(0);
  let playInterval: ReturnType<typeof setInterval> | null = null;
  let expandedEvents = $state<Set<number>>(new Set());

  const filteredSessions = $derived(
    sessions.filter((s) => {
      if (!searchQuery) return true;
      const q = searchQuery.toLowerCase();
      return (s.first_message?.toLowerCase().includes(q) ?? false)
        || s.project_path.toLowerCase().includes(q)
        || s.id.toLowerCase().includes(q);
    }),
  );

  // Meaningful events only (skip progress, tool-result-as-user-msg, etc.)
  const displayEvents = $derived(
    events.filter((e) => {
      if (!["user", "assistant", "tool_result"].includes(e.type)) return false;
      // Skip user messages that are just tool_result wrappers (API-level, not real user input)
      if (e.type === "user") {
        const msg = (e.content.message ?? {}) as Record<string, unknown>;
        const content = msg.content;
        if (Array.isArray(content)) {
          const hasText = (content as Array<Record<string, unknown>>).some((item) => item.type === "text");
          if (!hasText) return false; // Only tool_result items, no actual user text
        }
      }
      return true;
    }),
  );

  const visibleEvents = $derived(
    playing ? displayEvents.slice(0, visibleCount) : displayEvents,
  );

  const TOOL_ICONS: Record<string, typeof Terminal> = {
    Bash: Terminal, Read: Eye, Write: Pencil, Edit: Pencil,
    Glob: Folder, Grep: Search, Agent: Bot, WebFetch: Globe,
    WebSearch: Globe, ToolSearch: Wrench, TaskCreate: Zap,
    TaskUpdate: Zap, EnterPlanMode: Brain, ExitPlanMode: Brain,
  };

  function formatTime(ts: string | null): string {
    if (!ts) return "";
    return new Date(ts).toLocaleString("en", {
      month: "short", day: "numeric", hour: "2-digit", minute: "2-digit",
    });
  }

  function formatDuration(start: string | null, end: string | null): string {
    if (!start || !end) return "";
    const ms = new Date(end).getTime() - new Date(start).getTime();
    const mins = Math.floor(ms / 60000);
    if (mins < 60) return `${mins}m`;
    return `${Math.floor(mins / 60)}h ${mins % 60}m`;
  }

  function getEventText(event: SessionEvent): string {
    const content = event.content;
    const msg = (content.message ?? {}) as Record<string, unknown>;
    const msgContent = msg.content;

    if (event.type === "user") {
      if (typeof msgContent === "string") return msgContent;
      if (Array.isArray(msgContent)) {
        for (const item of msgContent) {
          if ((item as Record<string, unknown>).type === "text") {
            return (item as Record<string, unknown>).text as string;
          }
        }
      }
      return "";
    }

    if (event.type === "assistant") {
      const parts: string[] = [];
      if (Array.isArray(msgContent)) {
        for (const item of msgContent as Array<Record<string, unknown>>) {
          if (item.type === "text") parts.push(item.text as string);
        }
      }
      return parts.join("\n\n");
    }

    if (event.type === "tool_result") {
      if (typeof msgContent === "string") return msgContent;
      if (Array.isArray(msgContent)) {
        for (const item of msgContent as Array<Record<string, unknown>>) {
          if (item.type === "text") return (item.text as string).slice(0, 500);
        }
      }
      return "";
    }

    return "";
  }

  function getToolCalls(event: SessionEvent): Array<{ name: string; input: string }> {
    if (event.type !== "assistant") return [];
    const msg = (event.content.message ?? {}) as Record<string, unknown>;
    const content = msg.content;
    if (!Array.isArray(content)) return [];
    return (content as Array<Record<string, unknown>>)
      .filter((item) => item.type === "tool_use")
      .map((item) => ({
        name: item.name as string,
        input: JSON.stringify(item.input ?? {}).slice(0, 200),
      }));
  }

  function toggleExpand(idx: number) {
    const next = new Set(expandedEvents);
    if (next.has(idx)) next.delete(idx);
    else next.add(idx);
    expandedEvents = next;
  }

  async function selectSession(session: SessionSummary) {
    selectedSession = session;
    loadingEvents = true;
    playing = false;
    visibleCount = 0;
    expandedEvents = new Set();
    if (playInterval) clearInterval(playInterval);
    try {
      events = await api.sessions.load(session.path);
    } catch (e) {
      console.error("Failed to load session:", e);
      events = [];
    } finally {
      loadingEvents = false;
    }
  }

  function startReplay() {
    playing = true;
    visibleCount = 0;
    playInterval = setInterval(() => {
      if (visibleCount >= displayEvents.length) {
        playing = false;
        if (playInterval) clearInterval(playInterval);
        return;
      }
      visibleCount++;
    }, 300);
  }

  function stopReplay() {
    playing = false;
    if (playInterval) clearInterval(playInterval);
    visibleCount = displayEvents.length;
  }

  onMount(async () => {
    try {
      sessions = await api.sessions.list();
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="flex h-full">
  <!-- Session list sidebar -->
  <div class="w-80 shrink-0 border-r border-border flex flex-col bg-bg-secondary">
    <div class="p-3 border-b border-border">
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2 text-text-muted" />
        <input type="text" class="w-full pl-8 pr-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search sessions..." bind:value={searchQuery} />
      </div>
    </div>

    <div class="flex-1 overflow-y-auto">
      {#if loading}
        <p class="text-xs text-text-muted text-center py-4">Loading sessions...</p>
      {:else if filteredSessions.length === 0}
        <p class="text-xs text-text-muted text-center py-4">No sessions found</p>
      {:else}
        {#each filteredSessions as session}
          <button
            class="w-full text-left px-3 py-3 border-b border-border/50 transition-colors
              {selectedSession?.id === session.id ? 'bg-accent/10' : 'hover:bg-bg-hover'}"
            onclick={() => selectSession(session)}
          >
            <div class="flex items-center gap-2 mb-1">
              <History size={12} class="text-accent shrink-0" />
              <span class="text-xs font-medium text-text-primary truncate">
                {session.project_path.split("/").pop()}
              </span>
              <span class="text-[10px] text-text-muted ml-auto shrink-0">
                {formatTime(session.first_timestamp)}
              </span>
            </div>
            {#if session.first_message}
              <p class="text-xs text-text-secondary truncate ml-[20px]">"{session.first_message}"</p>
            {/if}
            <div class="flex gap-3 mt-1 ml-[20px] text-[10px] text-text-muted">
              <span>{session.user_messages} msgs</span>
              <span>{session.tool_calls} tools</span>
              <span>{session.entry_count} events</span>
              <span>{formatDuration(session.first_timestamp, session.last_timestamp)}</span>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Replay area -->
  <div class="flex-1 flex flex-col min-w-0">
    {#if !selectedSession}
      <div class="flex-1 flex flex-col items-center justify-center text-text-muted">
        <History size={32} class="opacity-20 mb-3" />
        <p class="text-sm">Select a session to replay</p>
        <p class="text-xs mt-1">Browse past Claude Code sessions step by step</p>
      </div>
    {:else if loadingEvents}
      <div class="flex-1 flex items-center justify-center text-sm text-text-muted">Loading session...</div>
    {:else}
      <!-- Session header -->
      <div class="flex items-center justify-between px-6 py-3 border-b border-border shrink-0 bg-bg-secondary">
        <div>
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-medium text-text-primary">
              {selectedSession.project_path.split("/").pop()}
            </h3>
            <span class="text-xs text-text-muted">{formatTime(selectedSession.first_timestamp)}</span>
          </div>
          {#if selectedSession.first_message}
            <p class="text-xs text-text-muted mt-0.5">"{selectedSession.first_message}"</p>
          {/if}
        </div>
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-3 text-xs text-text-muted">
            <span class="flex items-center gap-1"><MessageSquare size={10} />{selectedSession.user_messages}</span>
            <span class="flex items-center gap-1"><Wrench size={10} />{selectedSession.tool_calls}</span>
            <span class="flex items-center gap-1"><Clock size={10} />{formatDuration(selectedSession.first_timestamp, selectedSession.last_timestamp)}</span>
            <span class="flex items-center gap-1"><Hash size={10} />{displayEvents.length} events</span>
          </div>
          {#if !playing}
            <button
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
              onclick={startReplay}
            >
              <Play size={12} />
              Replay
            </button>
          {:else}
            <button
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary"
              onclick={stopReplay}
            >
              <Pause size={12} />
              Stop ({visibleCount}/{displayEvents.length})
            </button>
          {/if}
        </div>
      </div>

      <!-- Event stream -->
      <div class="flex-1 overflow-y-auto px-6 py-4 space-y-3">
        {#each visibleEvents as event, idx}
          {#if event.type === "user"}
            <!-- User message -->
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-info/10 flex items-center justify-center shrink-0 mt-0.5">
                <User size={14} class="text-info" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[10px] text-text-muted mb-1">You · {formatTime(event.timestamp)}</p>
                <div class="bg-info/5 border border-info/20 rounded-lg px-4 py-2">
                  <p class="text-sm text-text-primary">{getEventText(event)}</p>
                </div>
              </div>
            </div>

          {:else if event.type === "assistant"}
            <!-- Assistant response -->
            {@const toolCalls = getToolCalls(event)}
            {@const text = getEventText(event)}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-accent/10 flex items-center justify-center shrink-0 mt-0.5">
                <Bot size={14} class="text-accent" />
              </div>
              <div class="flex-1 min-w-0 space-y-2">
                <p class="text-[10px] text-text-muted">Claude · {formatTime(event.timestamp)}</p>

                <!-- Tool calls -->
                {#each toolCalls as call}
                  {@const ToolIcon = TOOL_ICONS[call.name] ?? Code}
                  <div class="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary rounded-md">
                    <ToolIcon size={12} class="text-warning shrink-0" />
                    <span class="text-xs font-medium text-warning">{call.name}</span>
                    <span class="text-[10px] text-text-muted font-mono truncate">{call.input}</span>
                  </div>
                {/each}

                <!-- Text response -->
                {#if text}
                  <div class="bg-bg-secondary border border-border rounded-lg px-4 py-2">
                    {#if text.length > 300 && !expandedEvents.has(idx)}
                      <div class="text-sm text-text-primary md-preview">
                        {@html marked(text.slice(0, 300) + "...") as string}
                      </div>
                      <button class="text-xs text-accent mt-1" onclick={() => toggleExpand(idx)}>
                        Show more
                      </button>
                    {:else}
                      <div class="text-sm text-text-primary md-preview">
                        {@html marked(text) as string}
                      </div>
                      {#if text.length > 300}
                        <button class="text-xs text-accent mt-1" onclick={() => toggleExpand(idx)}>
                          Show less
                        </button>
                      {/if}
                    {/if}
                  </div>
                {/if}
              </div>
            </div>

          {:else if event.type === "tool_result"}
            <!-- Tool result -->
            {@const resultText = getEventText(event)}
            {#if resultText}
              <div class="ml-10">
                <button
                  class="flex items-center gap-1.5 text-[10px] text-text-muted hover:text-text-secondary"
                  onclick={() => toggleExpand(idx)}
                >
                  {#if expandedEvents.has(idx)}
                    <ChevronDown size={10} />
                  {:else}
                    <ChevronRight size={10} />
                  {/if}
                  Tool output ({resultText.length} chars)
                </button>
                {#if expandedEvents.has(idx)}
                  <pre class="mt-1 px-3 py-2 bg-bg-tertiary rounded-md text-xs text-text-secondary font-mono overflow-x-auto max-h-48 overflow-y-auto">{resultText}</pre>
                {/if}
              </div>
            {/if}
          {/if}
        {/each}

        {#if playing && visibleCount < displayEvents.length}
          <div class="flex items-center justify-center py-4">
            <div class="w-2 h-2 rounded-full bg-accent animate-pulse"></div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
