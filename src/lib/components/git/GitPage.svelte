<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { GitStatus, GitLogEntry } from "$lib/tauri/commands";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import {
    GitBranch, ArrowUp, ArrowDown, RefreshCw, Copy, Check,
    FileEdit, FilePlus, FileX, FileQuestion, ChevronDown,
    Terminal as TerminalIcon,
  } from "lucide-svelte";

  let status = $state<GitStatus | null>(null);
  let logEntries = $state<GitLogEntry[]>([]);
  let branches = $state<string[]>([]);
  let loading = $state(false);
  let commitMessage = $state("");
  let commitType = $state("");
  let actionMessage = $state<string | null>(null);
  let showBranches = $state(false);
  let pushing = $state(false);
  let pulling = $state(false);
  let committing = $state(false);
  let errorMessage = $state<string | null>(null);
  let copiedHash = $state<string | null>(null);
  let refreshInterval: ReturnType<typeof setInterval> | null = null;

  const projectPath = $derived(getSelectedProjectPath());

  // Group file changes
  const fileGroups = $derived(() => {
    if (!status?.files) return { modified: [], added: [], deleted: [], untracked: [] };
    const modified: typeof status.files = [];
    const added: typeof status.files = [];
    const deleted: typeof status.files = [];
    const untracked: typeof status.files = [];
    for (const f of status.files) {
      if (f.status === "??" || f.status === "?") untracked.push(f);
      else if (f.status.includes("D")) deleted.push(f);
      else if (f.status === "A" || f.status === "AM") added.push(f);
      else modified.push(f);
    }
    return { modified, added, deleted, untracked };
  });

  const COMMIT_TYPES = [
    { value: "feat", label: "feat", desc: "New feature" },
    { value: "fix", label: "fix", desc: "Bug fix" },
    { value: "refactor", label: "refactor", desc: "Code change" },
    { value: "docs", label: "docs", desc: "Documentation" },
    { value: "test", label: "test", desc: "Tests" },
    { value: "chore", label: "chore", desc: "Maintenance" },
  ];

  const fullCommitMessage = $derived(
    commitType ? `${commitType}: ${commitMessage}` : commitMessage,
  );

  async function refresh() {
    if (!projectPath) return;
    loading = true;
    errorMessage = null;
    try {
      const [s, l, b] = await Promise.all([
        api.git.status(projectPath),
        api.git.log(projectPath, 20).catch(() => []),
        api.git.branches(projectPath).catch(() => []),
      ]);
      status = s;
      logEntries = l;
      branches = b;
    } catch (e) {
      errorMessage = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function doCommit() {
    if (!projectPath || !fullCommitMessage.trim()) return;
    committing = true;
    try {
      await api.git.commit(projectPath, fullCommitMessage.trim());
      commitMessage = "";
      commitType = "";
      showAction("Committed!");
      await refresh();
    } catch (e) { showAction(`Error: ${e}`); }
    finally { committing = false; }
  }

  async function doPush() {
    if (!projectPath) return;
    pushing = true;
    try { await api.git.push(projectPath); showAction("Pushed!"); await refresh(); }
    catch (e) { showAction(`Error: ${e}`); }
    finally { pushing = false; }
  }

  async function doPull() {
    if (!projectPath) return;
    pulling = true;
    try { await api.git.pull(projectPath); showAction("Pulled!"); await refresh(); }
    catch (e) { showAction(`Error: ${e}`); }
    finally { pulling = false; }
  }

  async function doCheckout(branch: string) {
    if (!projectPath) return;
    try { await api.git.checkout(projectPath, branch); showBranches = false; showAction(`Switched to ${branch}`); await refresh(); }
    catch (e) { showAction(`Error: ${e}`); }
  }

  async function doInit() {
    if (!projectPath) return;
    try { await api.git.init(projectPath); showAction("Initialized!"); await refresh(); }
    catch (e) { showAction(`Error: ${e}`); }
  }

  function copyHash(hash: string) {
    navigator.clipboard.writeText(hash);
    copiedHash = hash;
    setTimeout(() => (copiedHash = null), 2000);
  }

  function showAction(msg: string) {
    actionMessage = msg;
    setTimeout(() => (actionMessage = null), 3000);
  }

  $effect(() => {
    if (projectPath) refresh();
  });

  onMount(() => {
    refreshInterval = setInterval(() => { if (projectPath && status?.is_repo) refresh(); }, 30000);
  });

  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
  });
</script>

<div class="flex flex-col h-full">
  <!-- Top bar -->
  <div class="flex items-center justify-between px-6 py-3 border-b border-border shrink-0">
    <div class="flex items-center gap-3">
      <ProjectPicker onselect={refresh} />
      {#if projectPath && status?.is_repo}
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
          onclick={() => projectPath && api.git.openInTerminal(projectPath)}
        >
          <TerminalIcon size={14} />
          Open in Claude Code
        </button>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if errorMessage}
        <span class="text-xs text-danger">{errorMessage}</span>
      {/if}
      {#if actionMessage}
        <span class="text-xs {actionMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{actionMessage}</span>
      {/if}
      {#if status?.is_repo}
        <button class="p-1.5 text-text-muted hover:text-text-primary rounded transition-colors" onclick={refresh} aria-label="Refresh">
          <RefreshCw size={14} class={loading ? "animate-spin" : ""} />
        </button>
      {/if}
    </div>
  </div>

  {#if !projectPath}
    <div class="flex-1 flex flex-col items-center justify-center text-text-muted">
      <GitBranch size={32} class="opacity-20 mb-3" />
      <p class="text-sm">Select a project to view git status</p>
    </div>
  {:else if loading && !status}
    <div class="flex-1 flex items-center justify-center text-sm text-text-muted">Loading...</div>
  {:else if !status?.is_repo}
    <div class="flex-1 flex flex-col items-center justify-center">
      <GitBranch size={32} class="opacity-20 text-text-muted mb-3" />
      <p class="text-sm text-text-muted mb-3">Not a git repository</p>
      <button class="px-4 py-2 text-sm bg-accent hover:bg-accent-hover text-white rounded-md" onclick={doInit}>Initialize Git</button>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto">
      <!-- Branch bar -->
      <div class="flex items-center gap-4 px-6 py-3 bg-bg-secondary border-b border-border">
        <!-- Branch selector -->
        <div class="relative">
          <button
            class="flex items-center gap-2 px-3 py-1.5 bg-bg-tertiary border border-border rounded-md text-sm hover:border-border-light transition-colors"
            onclick={() => (showBranches = !showBranches)}
          >
            <GitBranch size={14} class="text-accent" />
            <span class="font-mono text-text-primary">{status.branch || "HEAD"}</span>
            <ChevronDown size={12} class="text-text-muted" />
          </button>
          {#if showBranches}
            <button class="fixed inset-0 z-40" onclick={() => (showBranches = false)} aria-label="Close"></button>
            <div class="absolute top-full left-0 mt-1 w-64 bg-bg-secondary border border-border rounded-lg shadow-xl z-50 max-h-48 overflow-y-auto">
              {#each branches as branch}
                <button
                  class="w-full text-left px-3 py-1.5 text-sm font-mono hover:bg-bg-hover transition-colors {branch === status.branch ? 'text-accent bg-accent/5' : 'text-text-secondary'}"
                  onclick={() => doCheckout(branch)}
                >{branch}</button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Status badges -->
        <span class="px-2.5 py-1 text-xs rounded-full {status.clean ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning'}">
          {status.clean ? "Clean" : `${status.files.length} changed`}
        </span>

        {#if status.ahead > 0}
          <span class="flex items-center gap-1 text-xs text-info"><ArrowUp size={12} />{status.ahead} ahead</span>
        {/if}
        {#if status.behind > 0}
          <span class="flex items-center gap-1 text-xs text-warning"><ArrowDown size={12} />{status.behind} behind</span>
        {/if}

        <div class="flex-1"></div>

        <button class="px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-border-light disabled:opacity-50" onclick={doPull} disabled={pulling}>
          {pulling ? "Pulling..." : "Pull"}
        </button>
        <button class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50" onclick={doPush} disabled={pushing}>
          {pushing ? "Pushing..." : "Push"}
        </button>
      </div>

      <div class="p-6 space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <!-- Changes -->
          <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
            <div class="px-4 py-3 border-b border-border">
              <h3 class="text-sm font-medium text-text-secondary">Changes</h3>
            </div>
            <div class="p-3 max-h-80 overflow-y-auto">
              {#if status.clean}
                <p class="text-xs text-text-muted text-center py-4">Working tree clean</p>
              {:else}
                {#each [
                  { label: "Modified", files: fileGroups().modified, icon: FileEdit, color: "text-warning" },
                  { label: "Added", files: fileGroups().added, icon: FilePlus, color: "text-success" },
                  { label: "Deleted", files: fileGroups().deleted, icon: FileX, color: "text-danger" },
                  { label: "Untracked", files: fileGroups().untracked, icon: FileQuestion, color: "text-info" },
                ] as group}
                  {#if group.files.length > 0}
                    <div class="mb-3">
                      <p class="text-[10px] uppercase tracking-wider {group.color} mb-1 flex items-center gap-1">
                        <group.icon size={10} />
                        {group.label} ({group.files.length})
                      </p>
                      {#each group.files as file}
                        <div class="flex items-center gap-2 py-0.5 text-xs font-mono text-text-primary">
                          <span class="truncate">{file.path}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                {/each}
              {/if}
            </div>

            <!-- Commit form -->
            {#if !status.clean}
              <div class="px-4 py-3 border-t border-border space-y-2">
                <div class="flex gap-2">
                  <select
                    class="px-2 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent"
                    bind:value={commitType}
                    aria-label="Commit type"
                  >
                    <option value="">type...</option>
                    {#each COMMIT_TYPES as ct}
                      <option value={ct.value}>{ct.label} — {ct.desc}</option>
                    {/each}
                  </select>
                  <input
                    type="text"
                    aria-label="Commit message"
                    class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
                    placeholder="commit message..."
                    bind:value={commitMessage}
                    onkeydown={(e) => e.key === "Enter" && doCommit()}
                  />
                </div>
                {#if fullCommitMessage}
                  <p class="text-[10px] text-text-muted font-mono truncate">→ {fullCommitMessage}</p>
                {/if}
                <button
                  class="w-full py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50"
                  onclick={doCommit}
                  disabled={!fullCommitMessage.trim() || committing}
                >
                  {committing ? "Committing..." : "Commit All Changes"}
                </button>
              </div>
            {/if}
          </div>

          <!-- Recent Commits -->
          <div class="bg-bg-secondary border border-border rounded-lg overflow-hidden">
            <div class="px-4 py-3 border-b border-border">
              <h3 class="text-sm font-medium text-text-secondary">Recent Commits</h3>
            </div>
            <div class="max-h-96 overflow-y-auto">
              {#each logEntries as entry, i}
                <div class="px-4 py-2.5 border-b border-border/50 hover:bg-bg-hover/50 transition-colors {i === 0 ? '' : ''}">
                  <div class="flex items-start gap-3">
                    <!-- Timeline dot -->
                    <div class="flex flex-col items-center mt-1 shrink-0">
                      <div class="w-2 h-2 rounded-full {i === 0 ? 'bg-accent' : 'bg-border'}"></div>
                      {#if i < logEntries.length - 1}
                        <div class="w-px h-full bg-border/50 mt-1"></div>
                      {/if}
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="text-sm text-text-primary">{entry.message}</p>
                      <div class="flex items-center gap-2 mt-1">
                        <button
                          class="flex items-center gap-1 text-[10px] font-mono text-accent hover:text-accent-hover transition-colors"
                          onclick={() => copyHash(entry.hash)}
                          title="Copy full hash"
                        >
                          {#if copiedHash === entry.hash}
                            <Check size={10} class="text-success" />
                          {:else}
                            <Copy size={10} />
                          {/if}
                          {entry.hash.slice(0, 7)}
                        </button>
                        <span class="text-[10px] text-text-muted">{entry.author}</span>
                        <span class="text-[10px] text-text-muted">·</span>
                        <span class="text-[10px] text-text-muted">{entry.date}</span>
                      </div>
                    </div>
                  </div>
                </div>
              {/each}
              {#if logEntries.length === 0}
                <p class="text-xs text-text-muted text-center py-4">No commits yet</p>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
