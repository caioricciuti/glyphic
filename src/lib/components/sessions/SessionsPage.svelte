<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { SessionSummary, SessionEvent, SearchResult, SessionTags, LiveSession } from "$lib/tauri/commands";
  import { marked } from "marked";
  import {
    History, Search, Play, Pause, ChevronDown, ChevronRight,
    User, Bot, Terminal, Pencil, Eye, Folder, Code,
    Wrench, Globe, Zap, Brain, Clock, MessageSquare, Hash,
    Tag, FileDown, Radio, Check, X,
  } from "lucide-svelte";

  let sessions = $state<SessionSummary[]>([]);
  let loading = $state(true);
  let loadingMore = $state(false);
  let hasMore = $state(false);
  let totalCount = $state(0);
  let searchQuery = $state("");
  let selectedSession = $state<SessionSummary | null>(null);
  let events = $state<SessionEvent[]>([]);
  let loadingEvents = $state(false);
  let loadingMoreEvents = $state(false);
  let hasMoreEvents = $state(false);
  let totalEvents = $state(0);

  // Search
  let searchResults = $state<SearchResult[]>([]);
  let searching = $state(false);
  let searchMode = $state(false);

  // Tags
  let sessionTags = $state<SessionTags>({ tags: {}, notes: {} });
  let showTagEditor = $state(false);
  let editTagInput = $state("");

  // Export
  let exportedMd = $state<string | null>(null);
  let copiedExport = $state(false);

  // Live sessions
  let liveSessions = $state<LiveSession[]>([]);

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
        input: JSON.stringify(item.input ?? {}),
      }));
  }

  let expandedToolCalls = $state<Set<string>>(new Set());

  function toggleToolCall(key: string) {
    const next = new Set(expandedToolCalls);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    expandedToolCalls = next;
  }

  function toggleExpand(idx: number) {
    const next = new Set(expandedEvents);
    if (next.has(idx)) next.delete(idx);
    else next.add(idx);
    expandedEvents = next;
  }

  async function selectSession(session: SessionSummary) {
    // Immediately clear and show loading
    selectedSession = session;
    events = [];
    loadingEvents = true;
    playing = false;
    visibleCount = 0;
    expandedEvents = new Set();
    expandedToolCalls = new Set();
    if (playInterval) clearInterval(playInterval);
    try {
      const result = await api.sessions.load(session.path, 50, 0);
      events = result.events;
      hasMoreEvents = result.has_more;
      totalEvents = result.total;
    } catch (e) {
      console.error("Failed to load session:", e);
      events = [];
    } finally {
      loadingEvents = false;
    }
  }

  async function loadMoreEvents() {
    if (!selectedSession || loadingMoreEvents) return;
    loadingMoreEvents = true;
    try {
      const result = await api.sessions.load(selectedSession.path, 50, events.length);
      events = [...events, ...result.events];
      hasMoreEvents = result.has_more;
      totalEvents = result.total;
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loadingMoreEvents = false;
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

  async function doSearch() {
    if (!searchQuery.trim()) { searchMode = false; searchResults = []; return; }
    searching = true;
    searchMode = true;
    try {
      searchResults = await api.sessions.search(searchQuery, 20);
    } catch (e) { console.error("Search failed:", e); }
    finally { searching = false; }
  }

  async function loadTags() {
    try { sessionTags = await api.sessions.getTags(); } catch { /* silent */ }
  }

  async function addTag(tag: string) {
    if (!selectedSession || !tag.trim()) return;
    const current = sessionTags.tags[selectedSession.id] ?? [];
    if (!current.includes(tag.trim())) {
      await api.sessions.setTag(selectedSession.id, [...current, tag.trim()]);
      await loadTags();
    }
    editTagInput = "";
  }

  async function removeTag(tag: string) {
    if (!selectedSession) return;
    const current = (sessionTags.tags[selectedSession.id] ?? []).filter((t) => t !== tag);
    await api.sessions.setTag(selectedSession.id, current);
    await loadTags();
  }

  async function exportSession() {
    if (!selectedSession) return;
    try {
      exportedMd = await api.sessions.exportMarkdown(selectedSession.path);
      await navigator.clipboard.writeText(exportedMd);
      copiedExport = true;
      setTimeout(() => (copiedExport = false), 2000);
    } catch (e) { console.error("Export failed:", e); }
  }

  async function detectLive() {
    try { liveSessions = await api.sessions.detectLive(); } catch { /* silent */ }
  }

  function isLive(path: string): boolean {
    return liveSessions.some((l) => l.path === path);
  }

  async function loadMore() {
    loadingMore = true;
    try {
      const result = await api.sessions.list(10, sessions.length);
      sessions = [...sessions, ...result.sessions];
      hasMore = result.has_more;
      totalCount = result.total;
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loadingMore = false;
    }
  }

  let liveInterval: ReturnType<typeof setInterval> | null = null;

  onMount(() => {
    (async () => {
      try {
        const [result] = await Promise.all([
          api.sessions.list(10, 0),
          loadTags(),
          detectLive(),
        ]);
        sessions = result.sessions;
        hasMore = result.has_more;
        totalCount = result.total;
      } catch (e) {
        console.error("Failed:", e);
      } finally {
        loading = false;
      }
    })();
    liveInterval = setInterval(detectLive, 30000);
    return () => { if (liveInterval) clearInterval(liveInterval); };
  });
</script>

<div class="flex h-full">
  <!-- Session list sidebar -->
  <div class="w-80 shrink-0 border-r border-border flex flex-col bg-bg-secondary">
    <div class="p-3 border-b border-border">
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2 text-text-muted" />
        <input type="text" class="w-full pl-8 pr-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search all sessions..." bind:value={searchQuery} onkeydown={(e) => e.key === "Enter" && doSearch()} />
        {#if searchQuery.trim()}
          <button class="absolute right-2 top-1.5 text-text-muted hover:text-accent" onclick={doSearch} aria-label="Search">
            <Search size={12} />
          </button>
        {/if}
      </div>
    </div>

    <div class="flex-1 overflow-y-auto">
      <!-- Live sessions -->
      {#if liveSessions.length > 0 && !searchMode}
        <div class="px-3 py-2 border-b border-border">
          <p class="text-[10px] text-success uppercase tracking-wider flex items-center gap-1 mb-1">
            <Radio size={10} class="animate-pulse" /> Live ({liveSessions.length})
          </p>
          {#each liveSessions as live}
            <div class="flex items-center gap-2 text-xs text-text-secondary py-0.5">
              <span class="w-1.5 h-1.5 rounded-full bg-success animate-pulse"></span>
              <span class="truncate">{live.project_path.split("/").pop()}</span>
              <span class="text-[10px] text-text-muted ml-auto">{live.modified_secs_ago}s ago</span>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Search results -->
      {#if searchMode}
        <div class="px-3 py-2 border-b border-border flex items-center justify-between">
          <span class="text-xs text-text-muted">{searching ? "Searching..." : `${searchResults.length} results`}</span>
          <button class="text-xs text-accent" onclick={() => { searchMode = false; searchQuery = ""; }}>Clear</button>
        </div>
        {#each searchResults as result}
          <button
            class="w-full text-left px-3 py-2 border-b border-border/50 hover:bg-bg-hover"
            onclick={() => {
              const found = sessions.find((s) => s.id === result.session_id);
              if (found) selectSession(found);
            }}
          >
            <div class="flex items-center gap-2">
              <Search size={10} class="text-accent shrink-0" />
              <span class="text-xs text-text-primary truncate">{result.project_path.split("/").pop()}</span>
            </div>
            <p class="text-[10px] text-text-secondary mt-0.5 ml-[18px] line-clamp-2">{result.snippet}</p>
          </button>
        {/each}
      {:else if loading}
        <p class="text-xs text-text-muted text-center py-4">Loading sessions...</p>
      {:else if filteredSessions.length === 0}
        <p class="text-xs text-text-muted text-center py-4">No sessions found</p>
      {:else}
        {#each filteredSessions as session}
          {@const tags = sessionTags.tags[session.id] ?? []}
          <button
            class="w-full text-left px-3 py-3 border-b border-border/50 transition-colors
              {selectedSession?.id === session.id ? 'bg-accent/10' : 'hover:bg-bg-hover'}"
            onclick={() => selectSession(session)}
          >
            <div class="flex items-center gap-2 mb-1">
              {#if isLive(session.path)}
                <span class="w-2 h-2 rounded-full bg-success animate-pulse shrink-0"></span>
              {:else}
                <History size={12} class="text-accent shrink-0" />
              {/if}
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
            {#if tags.length > 0}
              <div class="flex gap-1 mt-1 ml-[20px]">
                {#each tags as tag}
                  <span class="text-[9px] px-1.5 py-0.5 rounded-full bg-accent/10 text-accent">{tag}</span>
                {/each}
              </div>
            {/if}
            <div class="flex gap-3 mt-1 ml-[20px] text-[10px] text-text-muted">
              <span>{session.user_messages} msgs</span>
              <span>{session.tool_calls} tools</span>
              <span>{session.entry_count} events</span>
              <span>{formatDuration(session.first_timestamp, session.last_timestamp)}</span>
            </div>
          </button>
        {/each}

        <!-- Load more -->
        {#if hasMore}
          <div class="p-3">
            <button
              class="w-full py-2 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 transition-colors disabled:opacity-50"
              onclick={loadMore}
              disabled={loadingMore}
            >
              {loadingMore ? "Loading..." : `Load more (${sessions.length}/${totalCount})`}
            </button>
          </div>
        {/if}
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
          <!-- Export -->
          <button
            class="flex items-center gap-1 px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 transition-colors"
            onclick={exportSession}
            title="Export as Markdown + copy to clipboard"
          >
            {#if copiedExport}
              <Check size={12} class="text-success" />
            {:else}
              <FileDown size={12} />
            {/if}
            Export
          </button>
          <!-- Tags -->
          <button
            class="flex items-center gap-1 px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 transition-colors"
            onclick={() => (showTagEditor = !showTagEditor)}
          >
            <Tag size={12} />
            Tag
          </button>
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

      <!-- Tag editor bar -->
      {#if showTagEditor && selectedSession}
        {@const currentTags = sessionTags.tags[selectedSession.id] ?? []}
        <div class="flex items-center gap-2 px-6 py-2 border-b border-border bg-bg-secondary/50">
          {#each currentTags as tag}
            <span class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-accent/10 text-accent">
              {tag}
              <button class="hover:text-danger" onclick={() => removeTag(tag)} aria-label="Remove tag"><X size={10} /></button>
            </span>
          {/each}
          <div class="flex items-center gap-1">
            <input
              type="text"
              class="w-24 px-2 py-0.5 text-xs bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent"
              placeholder="add tag..."
              bind:value={editTagInput}
              onkeydown={(e) => e.key === "Enter" && addTag(editTagInput)}
            />
            {#each ["bug-fix", "feature", "refactor"] as preset}
              {#if !currentTags.includes(preset)}
                <button
                  class="text-[10px] px-1.5 py-0.5 rounded bg-bg-tertiary text-text-muted hover:text-accent"
                  onclick={() => addTag(preset)}
                >{preset}</button>
              {/if}
            {/each}
          </div>
        </div>
      {/if}

      <!-- Event stream -->
      <div class="flex-1 overflow-y-auto px-6 py-4 space-y-3">
        {#each visibleEvents as event, idx}
          {#if event.type === "user"}
            <!-- User message -->
            {@const userText = getEventText(event)}
            <div class="flex gap-3">
              <div class="w-7 h-7 rounded-full bg-info/10 flex items-center justify-center shrink-0 mt-0.5">
                <User size={14} class="text-info" />
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-[10px] text-text-muted mb-1">You · {formatTime(event.timestamp)}</p>
                <div class="bg-info/5 border border-info/20 rounded-lg px-4 py-2">
                  {#if userText.length > 300 && !expandedEvents.has(idx)}
                    <p class="text-sm text-text-primary whitespace-pre-wrap">{userText.slice(0, 300)}…</p>
                    <button class="text-xs text-accent mt-1" onclick={() => toggleExpand(idx)}>
                      Show more ({userText.length} chars)
                    </button>
                  {:else}
                    <p class="text-sm text-text-primary whitespace-pre-wrap">{userText}</p>
                    {#if userText.length > 300}
                      <button class="text-xs text-accent mt-1" onclick={() => toggleExpand(idx)}>
                        Show less
                      </button>
                    {/if}
                  {/if}
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
                {#each toolCalls as call, callIdx}
                  {@const ToolIcon = TOOL_ICONS[call.name] ?? Code}
                  {@const tcKey = `${idx}-${callIdx}`}
                  {@const tcExpanded = expandedToolCalls.has(tcKey)}
                  <div class="bg-bg-tertiary rounded-md overflow-hidden">
                    <button
                      type="button"
                      class="flex items-center gap-2 px-3 py-1.5 w-full text-left hover:bg-bg-hover transition-colors"
                      onclick={() => toggleToolCall(tcKey)}
                    >
                      <ToolIcon size={12} class="text-warning shrink-0" />
                      <span class="text-xs font-medium text-warning">{call.name}</span>
                      {#if !tcExpanded}
                        <span class="text-[10px] text-text-muted font-mono truncate">{call.input.length > 120 ? call.input.slice(0, 120) + "…" : call.input}</span>
                      {/if}
                      {#if call.input.length > 120}
                        <span class="text-[10px] text-text-muted ml-auto shrink-0">
                          {tcExpanded ? "▾" : "▸"}
                        </span>
                      {/if}
                    </button>
                    {#if tcExpanded}
                      <pre class="px-3 py-2 text-[10px] text-text-secondary font-mono whitespace-pre-wrap break-all border-t border-border/30 max-h-64 overflow-y-auto">{call.input}</pre>
                    {/if}
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

        {#if hasMoreEvents && !playing}
          <div class="py-4">
            <button
              class="w-full py-2 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 transition-colors disabled:opacity-50"
              onclick={loadMoreEvents}
              disabled={loadingMoreEvents}
            >
              {loadingMoreEvents ? "Loading..." : `Load more events (${events.length}/${totalEvents})`}
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
