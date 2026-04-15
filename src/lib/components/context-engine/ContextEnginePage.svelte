<script lang="ts">
  import { onMount } from "svelte";
  import { api, type ContextEngineStatus, type RecentToolResult } from "$lib/tauri/commands";
  import {
    Network,
    Power,
    Database,
    Layers,
    MessageSquare,
    RefreshCw,
    ToggleLeft,
    ToggleRight,
    Copy,
    Sparkles,
    Trash2,
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

  const enabled = $derived(status?.enabled ?? false);
  const totalRows = $derived((status?.toolResults ?? 0) + (status?.turns ?? 0));
  const embeddedRows = $derived(
    (status?.embeddedToolResults ?? 0) + (status?.embeddedTurns ?? 0),
  );
  const coveragePct = $derived(
    totalRows > 0 ? Math.round((embeddedRows / totalRows) * 100) : 0,
  );
  const rowsToReindex = $derived(Math.max(0, totalRows - embeddedRows));

  async function load() {
    try {
      status = await api.contextEngine.status();
      recent = await api.contextEngine.recentToolResults(undefined, 30);
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

  onMount(load);
</script>

<div class="flex flex-col h-full overflow-hidden">
  <header class="px-8 pt-8 pb-6 border-b border-border">
    <div class="flex items-center justify-between gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2 rounded-lg bg-accent/10 text-accent">
          <Network size={22} />
        </div>
        <div>
          <h1 class="text-xl font-semibold">Context Engine</h1>
          <p class="text-sm text-text-secondary">
            Structured retrieval + tool-output virtualization via Claude Code hooks.
          </p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="flex items-center gap-2 px-3 py-2 rounded-lg border border-border text-sm text-text-secondary hover:bg-bg-hover hover:text-text-primary disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          onclick={purgeLegacy}
          disabled={purging || reindexing || refreshing || loading}
          title="Remove stored tool results from tools now on the skip list, and pre-extractor rows that still hold the raw JSON envelope. Idempotent."
        >
          <Trash2 size={14} class={purging ? "animate-pulse" : ""} />
          {#if purging}
            <span>Cleaning…</span>
          {:else if lastPurgeDeleted !== null}
            <span>Cleaned {lastPurgeDeleted.toLocaleString()}</span>
          {:else}
            <span>Clean legacy</span>
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
            <span>Reindexing… {reindexProgress.processed.toLocaleString()}</span>
          {:else}
            <span>Reindex</span>
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
            ? 'border border-border text-text-primary hover:bg-bg-hover'
            : 'bg-accent text-white hover:bg-accent-hover'}"
          onclick={toggle}
          disabled={toggling || loading}
        >
          {#if enabled}
            <ToggleRight size={16} />
            <span>Enabled</span>
          {:else}
            <ToggleLeft size={16} />
            <span>Enable</span>
          {/if}
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
      <div class="text-sm text-text-secondary">Loading…</div>
    {:else if status}
      <section class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-4">
        <div class="p-4 rounded-lg border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 text-text-secondary text-xs uppercase tracking-wide">
            <Power size={13} /> Status
          </div>
          <div class="mt-2 text-lg font-semibold {enabled ? 'text-success' : 'text-text-secondary'}">
            {enabled ? "Active" : "Inactive"}
          </div>
          <div class="mt-1 text-xs text-text-secondary">
            sidecar: {status.sidecarInstalled ? "✓" : "✗"} · hooks: {status.hookInstalled ? "✓" : "✗"}
          </div>
        </div>
        <div class="p-4 rounded-lg border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 text-text-secondary text-xs uppercase tracking-wide">
            <Layers size={13} /> Tool results
          </div>
          <div class="mt-2 text-lg font-semibold">{status.toolResults.toLocaleString()}</div>
          <div class="mt-1 text-xs text-text-secondary">stored for retrieval</div>
        </div>
        <div class="p-4 rounded-lg border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 text-text-secondary text-xs uppercase tracking-wide">
            <MessageSquare size={13} /> Prompts indexed
          </div>
          <div class="mt-2 text-lg font-semibold">{status.turns.toLocaleString()}</div>
          <div class="mt-1 text-xs text-text-secondary">user turns in FTS</div>
        </div>
        <div class="p-4 rounded-lg border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 text-text-secondary text-xs uppercase tracking-wide">
            <Database size={13} /> Store
          </div>
          <div class="mt-2 text-lg font-semibold">{formatBytes(status.bytesStored)}</div>
          <div class="mt-1 text-xs text-text-secondary truncate" title={status.dbPath}>
            {status.dbPath}
          </div>
        </div>
        <div class="p-4 rounded-lg border border-border bg-bg-secondary">
          <div class="flex items-center gap-2 text-text-secondary text-xs uppercase tracking-wide">
            <Sparkles size={13} /> Semantic coverage
          </div>
          <div class="mt-2 text-lg font-semibold {coveragePct >= 80 ? 'text-success' : coveragePct > 0 ? 'text-text-primary' : 'text-text-secondary'}">
            {coveragePct}%
          </div>
          <div class="mt-1 text-xs text-text-secondary">
            {embeddedRows.toLocaleString()} / {totalRows.toLocaleString()} embedded
            {#if !status.embeddingReady}
              · model warming
            {/if}
          </div>
        </div>
      </section>
      {#if totalRows > 0 && coveragePct < 80}
        <p class="-mt-2 text-xs text-text-secondary">
          Semantic rerank kicks in gradually as rows get embedded. Click Reindex to backfill now.
        </p>
      {/if}

      <section>
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-sm font-semibold uppercase tracking-wide text-text-secondary">
            Recent stored tool results
          </h2>
          <span class="text-xs text-text-secondary">
            {recent.length} most recent
          </span>
        </div>

        {#if recent.length === 0}
          <div class="p-6 rounded-lg border border-dashed border-border text-sm text-text-secondary">
            No tool results stored yet. Enable the engine and make a few Bash / Read / MCP calls — oversized outputs will be virtualized and indexed here.
          </div>
        {:else}
          <div class="space-y-2">
            {#each recent as r (r.id)}
              <article class="p-3 rounded-lg border border-border bg-bg-secondary hover:border-accent/40 transition-colors">
                <div class="flex items-center justify-between gap-3">
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-mono text-xs px-1.5 py-0.5 rounded bg-bg-tertiary text-accent">
                      {r.id}
                    </span>
                    <span class="text-xs font-medium">{r.tool}</span>
                    <span class="text-xs text-text-secondary">
                      {r.lineCount.toLocaleString()} lines · {formatBytes(r.sizeBytes)}
                    </span>
                  </div>
                  <div class="flex items-center gap-2 shrink-0">
                    <span class="text-xs text-text-secondary">{formatTime(r.ts)}</span>
                    <button
                      type="button"
                      class="flex items-center justify-center w-6 h-6 rounded text-text-secondary hover:bg-bg-hover hover:text-text-primary transition-colors"
                      title="Copy expand command"
                      onclick={() => copyRef(r.id)}
                    >
                      <Copy size={12} />
                    </button>
                  </div>
                </div>
                {#if r.summary}
                  <pre class="mt-2 text-xs text-text-secondary font-mono whitespace-pre-wrap line-clamp-4">{r.summary}</pre>
                {/if}
                {#if r.project}
                  <div class="mt-2 text-[10px] text-text-secondary truncate" title={r.project}>
                    {r.project}
                  </div>
                {/if}
              </article>
            {/each}
          </div>
        {/if}
      </section>

      <section class="p-4 rounded-lg border border-border bg-bg-secondary text-sm">
        <h3 class="font-medium mb-2">How it works</h3>
        <ul class="space-y-1 text-text-secondary text-xs">
          <li>• <b>PostToolUse</b> stores every tool result in SQLite, indexed with FTS5.</li>
          <li>• Outputs over 2 KB are virtualized — Claude sees a summary + a <code>glyphic-ctx expand &lt;id&gt;</code> pointer, not raw bytes.</li>
          <li>• <b>UserPromptSubmit</b> runs a hybrid query: BM25 recall + local embedding rerank (BGE-Small-EN-v1.5, 384-dim, runs on CPU) so "login broken" can match prior "auth failing".</li>
          <li>• <b>PreToolUse</b> on Bash short-circuits <code>glyphic-ctx expand</code> calls, serving directly from the store.</li>
          <li>• Kill switch: set <code>GLYPHIC_CTX_DISABLED=1</code> in your shell env.</li>
        </ul>
      </section>
    {/if}
  </div>
</div>
