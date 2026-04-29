<script lang="ts">
  import { onMount } from "svelte";
  import { api, type ContextEngineStatus, type RecentToolResult } from "$lib/tauri/commands";
  import {
    Network,
    Power,
    Database,
    Layers,
    RefreshCw,
    ToggleLeft,
    ToggleRight,
    Copy,
    Sparkles,
    Trash2,
    Check,
    ChevronDown,
    ChevronUp,
    HardDrive,
    Cpu,
    Info,
  } from "lucide-svelte";

  let status = $state<ContextEngineStatus | null>(null);
  let recent = $state<RecentToolResult[]>([]);
  let loading = $state(true);
  let toggling = $state(false);
  let refreshing = $state(false);
  let reindexing = $state(false);
  let reindexProgress = $state<{ processed: number; remaining: number } | null>(null);
  let purging = $state(false);
  let lastPurgeDeleted = $state<number | null>(null);
  let error = $state<string | null>(null);
  let copiedId = $state<string | null>(null);
  let showHowItWorks = $state(false);
  let showAllResults = $state(false);

  const enabled = $derived(status?.enabled ?? false);
  const totalRows = $derived((status?.toolResults ?? 0) + (status?.turns ?? 0));
  const embeddedRows = $derived(
    (status?.embeddedToolResults ?? 0) + (status?.embeddedTurns ?? 0),
  );
  const coveragePct = $derived(
    totalRows > 0 ? Math.round((embeddedRows / totalRows) * 100) : 0,
  );
  const rowsToReindex = $derived(Math.max(0, totalRows - embeddedRows));

  const visibleResults = $derived(showAllResults ? recent : recent.slice(0, 15));

  async function load() {
    try {
      status = await api.contextEngine.status();
      recent = await api.contextEngine.recentToolResults(undefined, 50);
      error = null;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function toggle() {
    toggling = true;
    try {
      if (enabled) await api.contextEngine.disable();
      else await api.contextEngine.enable();
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      toggling = false;
    }
  }

  async function refresh() {
    refreshing = true;
    await load();
  }

  async function reindex() {
    reindexing = true;
    reindexProgress = { processed: 0, remaining: rowsToReindex };
    try {
      let guard = 0;
      while (guard++ < 200) {
        const report = await api.contextEngine.reindex(64);
        reindexProgress = {
          processed: (reindexProgress?.processed ?? 0) + report.processed,
          remaining: report.remaining,
        };
        if (report.remaining <= 0 || report.processed === 0) break;
      }
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      reindexing = false;
    }
  }

  async function purgeLegacy() {
    purging = true;
    try {
      const report = await api.contextEngine.purgeLegacy();
      lastPurgeDeleted = report.deleted;
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      purging = false;
    }
  }

  function copyRef(id: string) {
    navigator.clipboard.writeText(`glyphic-ctx expand ${id}`);
    copiedId = id;
    setTimeout(() => (copiedId = null), 1500);
  }

  function formatBytes(n: number): string {
    if (n >= 1024 * 1024) return `${(n / 1024 / 1024).toFixed(1)} MB`;
    if (n >= 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${n} B`;
  }

  function formatTime(ts: number): string {
    const diff = Date.now() / 1000 - ts;
    if (diff < 60) return `${Math.floor(diff)}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
  }

  function toolColor(tool: string): string {
    if (tool === "Bash") return "bg-accent/15 text-accent";
    if (tool === "Read") return "bg-success/15 text-success";
    if (tool === "Grep") return "bg-warning/15 text-warning";
    return "bg-bg-tertiary text-text-secondary";
  }

  onMount(load);
</script>

<div class="flex flex-col h-full overflow-hidden">
  <header class="px-8 pt-8 pb-6 border-b border-border">
    <div class="flex items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2.5 rounded-xl bg-accent/10 text-accent">
          <Network size={22} />
        </div>
        <div>
          <h1 class="text-xl font-semibold">Context Engine</h1>
          <p class="text-sm text-text-secondary">
            Structured retrieval and tool-output virtualization via Claude Code hooks
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="flex items-center gap-2 px-3 py-2 rounded-lg border border-border text-sm text-text-secondary hover:bg-bg-hover hover:text-text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          onclick={purgeLegacy}
          disabled={purging || reindexing || refreshing || loading}
          title="Remove stored tool results from tools now on the skip list"
        >
          <Trash2 size={14} class={purging ? "animate-pulse" : ""} />
          {#if purging}
            Cleaning…
          {:else if lastPurgeDeleted !== null}
            Cleaned {lastPurgeDeleted.toLocaleString()}
          {:else}
            Clean legacy
          {/if}
        </button>
        <button
          type="button"
          class="flex items-center gap-2 px-3 py-2 rounded-lg border border-border text-sm text-text-secondary hover:bg-bg-hover hover:text-text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          onclick={reindex}
          disabled={reindexing || refreshing || loading || rowsToReindex === 0}
          title={rowsToReindex === 0
            ? "All rows already embedded"
            : `Embed ${rowsToReindex.toLocaleString()} remaining row${rowsToReindex === 1 ? "" : "s"}`}
        >
          <Sparkles size={14} class={reindexing ? "animate-pulse" : ""} />
          {#if reindexing && reindexProgress}
            Reindexing… {reindexProgress.processed.toLocaleString()}
          {:else}
            Reindex
          {/if}
        </button>
        <button
          type="button"
          class="flex items-center justify-center w-9 h-9 rounded-lg border border-border text-text-secondary hover:bg-bg-hover hover:text-text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          onclick={refresh}
          disabled={refreshing || loading}
          aria-label="Refresh"
        >
          <RefreshCw size={16} class={refreshing ? "animate-spin" : ""} />
        </button>
        <button
          type="button"
          class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed {enabled
            ? 'bg-success/10 text-success hover:bg-success/20'
            : 'bg-accent text-white hover:bg-accent/90'}"
          onclick={toggle}
          disabled={toggling || loading}
        >
          {#if toggling}
            <RefreshCw size={14} class="animate-spin" />
          {:else if enabled}
            <ToggleRight size={16} />
          {:else}
            <ToggleLeft size={16} />
          {/if}
          {toggling ? "..." : enabled ? "Enabled" : "Enable"}
        </button>
      </div>
    </div>
  </header>

  <div class="flex-1 overflow-y-auto px-8 py-6 space-y-6">
    {#if error}
      <div class="p-4 rounded-lg border border-error/40 bg-error/10 text-error text-sm">
        {error}
      </div>
    {/if}

    {#if loading}
      <div class="flex items-center gap-3 p-8">
        <RefreshCw size={16} class="animate-spin text-text-secondary" />
        <span class="text-sm text-text-secondary">Loading context engine status…</span>
      </div>
    {:else if status && !enabled && totalRows === 0}
      <!-- Empty state when not enabled -->
      <div class="bg-bg-secondary border border-border rounded-xl p-10 text-center">
        <div class="w-16 h-16 rounded-2xl bg-accent/10 flex items-center justify-center mx-auto mb-5">
          <Network size={28} class="text-accent" />
        </div>
        <h3 class="text-lg font-semibold text-text-primary mb-2">
          Virtualize and retrieve tool outputs
        </h3>
        <p class="text-sm text-text-secondary max-w-lg mx-auto mb-8">
          The Context Engine intercepts tool results via Claude Code hooks, stores them in a local SQLite database with FTS5 indexing, and replaces oversized outputs with compact summaries. Semantic reranking finds related results across sessions.
        </p>
        <div class="grid grid-cols-3 gap-4 max-w-md mx-auto mb-8">
          {#each [
            { icon: HardDrive, label: "FTS5 index", desc: "Full-text search" },
            { icon: Cpu, label: "BGE embeddings", desc: "Semantic rerank" },
            { icon: Layers, label: "Virtualization", desc: "2KB+ compressed" },
          ] as feature}
            {@const Icon = feature.icon}
            <div class="bg-bg-hover rounded-lg p-4">
              <Icon size={18} class="text-accent mx-auto mb-2" />
              <p class="text-xs font-medium text-text-primary">{feature.label}</p>
              <p class="text-[10px] text-text-secondary mt-0.5">{feature.desc}</p>
            </div>
          {/each}
        </div>
        <button
          type="button"
          onclick={toggle}
          disabled={toggling}
          class="px-6 py-2.5 bg-accent text-white rounded-lg text-sm font-medium hover:bg-accent/90 transition-colors disabled:opacity-50"
        >
          {toggling ? "Enabling..." : "Enable Context Engine"}
        </button>
      </div>
    {:else if status}
      <!-- Stats grid -->
      <section class="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <div class="p-4 rounded-xl border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center {enabled ? 'bg-success/15' : 'bg-bg-hover'}">
              <Power size={13} class={enabled ? "text-success" : "text-text-secondary"} />
            </div>
            <span class="text-[10px] text-text-secondary uppercase tracking-wider">Status</span>
          </div>
          <p class="text-2xl font-bold {enabled ? 'text-success' : 'text-text-secondary'}">
            {enabled ? "Active" : "Inactive"}
          </p>
          <p class="text-xs text-text-secondary mt-1">
            sidecar {status.sidecarInstalled ? "installed" : "missing"} · hooks {status.hookInstalled ? "registered" : "missing"}
          </p>
        </div>

        <div class="p-4 rounded-xl border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center bg-accent/15">
              <Layers size={13} class="text-accent" />
            </div>
            <span class="text-[10px] text-text-secondary uppercase tracking-wider">Stored</span>
          </div>
          <p class="text-2xl font-bold text-text-primary">{status.toolResults.toLocaleString()}</p>
          <p class="text-xs text-text-secondary mt-1">
            tool results · {status.turns.toLocaleString()} turns indexed
          </p>
        </div>

        <div class="p-4 rounded-xl border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center bg-info/15">
              <Database size={13} class="text-info" />
            </div>
            <span class="text-[10px] text-text-secondary uppercase tracking-wider">Database</span>
          </div>
          <p class="text-2xl font-bold text-text-primary">{formatBytes(status.bytesStored)}</p>
          <p class="text-xs text-text-secondary mt-1 truncate" title={status.dbPath}>
            {status.dbPath.split("/").pop()}
          </p>
        </div>

        <div class="p-4 rounded-xl border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-7 h-7 rounded-lg flex items-center justify-center {coveragePct >= 80 ? 'bg-success/15' : 'bg-warning/15'}">
              <Sparkles size={13} class={coveragePct >= 80 ? "text-success" : "text-warning"} />
            </div>
            <span class="text-[10px] text-text-secondary uppercase tracking-wider">Embeddings</span>
          </div>
          <p class="text-2xl font-bold {coveragePct >= 80 ? 'text-success' : coveragePct > 0 ? 'text-text-primary' : 'text-text-secondary'}">
            {coveragePct}%
          </p>
          <div class="mt-2">
            <div class="w-full h-1.5 bg-bg-hover rounded-full overflow-hidden">
              <div
                class="h-full rounded-full transition-all duration-500 {coveragePct >= 80 ? 'bg-success' : coveragePct >= 40 ? 'bg-warning' : 'bg-accent'}"
                style="width: {Math.min(coveragePct, 100)}%"
              ></div>
            </div>
            <p class="text-[10px] text-text-secondary mt-1">
              {embeddedRows.toLocaleString()} / {totalRows.toLocaleString()} embedded
              {#if !status.embeddingReady}· model warming{/if}
            </p>
          </div>
        </div>
      </section>

      {#if totalRows > 0 && coveragePct < 80}
        <div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-warning/5 border border-warning/20">
          <Info size={13} class="text-warning shrink-0" />
          <p class="text-xs text-text-secondary">
            Semantic rerank improves as more rows get embedded. <button type="button" class="text-accent hover:underline" onclick={reindex}>Reindex now</button> to backfill {rowsToReindex.toLocaleString()} remaining rows.
          </p>
        </div>
      {/if}

      <!-- Recent stored tool results -->
      <section>
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-semibold text-text-primary">
            Recent stored results
          </h2>
          <span class="text-xs text-text-secondary">
            {recent.length} total
          </span>
        </div>

        {#if recent.length === 0}
          <div class="p-8 rounded-xl border border-dashed border-border text-center">
            <Layers size={24} class="text-text-secondary mx-auto mb-3 opacity-50" />
            <p class="text-sm text-text-secondary">
              No tool results stored yet.
            </p>
            <p class="text-xs text-text-secondary mt-1">
              Enable the engine and run some Bash, Read, or MCP calls — oversized outputs get virtualized here.
            </p>
          </div>
        {:else}
          <div class="bg-bg-secondary border border-border rounded-xl overflow-hidden">
            <div class="overflow-x-auto">
              <table class="w-full text-sm">
                <thead>
                  <tr class="border-b border-border text-text-secondary">
                    <th class="text-left px-4 py-2.5 text-xs font-medium">ID</th>
                    <th class="text-left px-4 py-2.5 text-xs font-medium">Tool</th>
                    <th class="text-left px-4 py-2.5 text-xs font-medium">Summary</th>
                    <th class="text-right px-4 py-2.5 text-xs font-medium">Size</th>
                    <th class="text-right px-4 py-2.5 text-xs font-medium">When</th>
                    <th class="text-center px-4 py-2.5 text-xs font-medium w-10"></th>
                  </tr>
                </thead>
                <tbody>
                  {#each visibleResults as r (r.id)}
                    <tr class="border-b border-border/50 hover:bg-bg-hover transition-colors group">
                      <td class="px-4 py-2.5">
                        <span class="font-mono text-xs px-1.5 py-0.5 rounded bg-bg-tertiary text-accent">
                          {r.id}
                        </span>
                      </td>
                      <td class="px-4 py-2.5">
                        <span class="text-xs font-medium px-2 py-0.5 rounded-full {toolColor(r.tool)}">
                          {r.tool}
                        </span>
                      </td>
                      <td class="px-4 py-2.5 max-w-xs">
                        {#if r.summary}
                          <p class="text-xs text-text-secondary truncate" title={r.summary}>{r.summary}</p>
                        {:else}
                          <span class="text-xs text-text-secondary opacity-50">—</span>
                        {/if}
                        {#if r.project}
                          <p class="text-[10px] text-text-secondary opacity-60 truncate mt-0.5" title={r.project}>{r.project}</p>
                        {/if}
                      </td>
                      <td class="px-4 py-2.5 text-right">
                        <span class="text-xs text-text-secondary font-mono">
                          {r.lineCount.toLocaleString()}L
                        </span>
                        <span class="text-[10px] text-text-secondary ml-1">
                          {formatBytes(r.sizeBytes)}
                        </span>
                      </td>
                      <td class="px-4 py-2.5 text-right text-xs text-text-secondary">
                        {formatTime(r.ts)}
                      </td>
                      <td class="px-4 py-2.5 text-center">
                        <button
                          type="button"
                          class="flex items-center justify-center w-6 h-6 rounded text-text-secondary opacity-0 group-hover:opacity-100 hover:bg-bg-hover hover:text-text-primary transition-all"
                          title="Copy expand command"
                          onclick={() => copyRef(r.id)}
                        >
                          {#if copiedId === r.id}
                            <Check size={12} class="text-success" />
                          {:else}
                            <Copy size={12} />
                          {/if}
                        </button>
                      </td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
            {#if recent.length > 15}
              <div class="px-4 py-2.5 border-t border-border">
                <button
                  type="button"
                  onclick={() => (showAllResults = !showAllResults)}
                  class="flex items-center gap-1 text-xs text-accent hover:underline"
                >
                  {#if showAllResults}
                    <ChevronUp size={12} />
                    Show less
                  {:else}
                    <ChevronDown size={12} />
                    Show all {recent.length} results
                  {/if}
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </section>

      <!-- How it works -->
      <section>
        <button
          type="button"
          class="flex items-center gap-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
          onclick={() => (showHowItWorks = !showHowItWorks)}
        >
          {#if showHowItWorks}
            <ChevronUp size={14} />
          {:else}
            <ChevronDown size={14} />
          {/if}
          How it works
        </button>

        {#if showHowItWorks}
          <div class="mt-3 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
            {#each [
              {
                hook: "PostToolUse",
                title: "Store & Index",
                desc: "Every tool result is stored in SQLite with FTS5 indexing. Outputs over 2 KB are virtualized — Claude sees a summary + expand pointer.",
                color: "bg-accent/10 text-accent border-accent/20",
              },
              {
                hook: "UserPromptSubmit",
                title: "Hybrid Recall",
                desc: "Runs BM25 keyword recall + local BGE-Small-EN-v1.5 embedding rerank (384-dim, CPU). \"login broken\" matches prior \"auth failing\".",
                color: "bg-success/10 text-success border-success/20",
              },
              {
                hook: "PreToolUse",
                title: "Expand Refs",
                desc: "Short-circuits `glyphic-ctx expand <id>` Bash calls, serving the full result directly from the store without re-running the command.",
                color: "bg-warning/10 text-warning border-warning/20",
              },
              {
                hook: "Kill Switch",
                title: "Emergency Off",
                desc: "Set GLYPHIC_CTX_DISABLED=1 in your shell environment to bypass all hooks instantly without modifying settings.",
                color: "bg-error/10 text-error border-error/20",
              },
            ] as step}
              <div class="p-4 rounded-xl border {step.color}">
                <span class="text-[10px] font-mono font-bold uppercase tracking-wider">{step.hook}</span>
                <h4 class="text-sm font-medium text-text-primary mt-1.5">{step.title}</h4>
                <p class="text-xs text-text-secondary mt-1 leading-relaxed">{step.desc}</p>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    {/if}
  </div>
</div>
