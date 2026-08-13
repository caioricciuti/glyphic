<script lang="ts">
  import { onMount } from "svelte";
  import { api, type McpTestResult } from "$lib/tauri/commands";
  import type { SettingsScope } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath, getProjects, isLoaded, loadProjects } from "$lib/stores/project-context.svelte";
  import { Server, Plus, Cloud, Terminal, Globe, Trash2, Edit3, X, LayoutGrid, ArrowRightLeft, Copy, FolderOpen, FlaskConical, Play } from "lucide-svelte";
  import TemplateGallery from "$lib/components/shared/TemplateGallery.svelte";

  interface ServerEntry {
    name: string;
    config: Record<string, unknown>;
  }

  let servers = $state<ServerEntry[]>([]);
  let cloudMcps = $state<string[]>([]);
  let scope = $state<SettingsScope>("desktop");
  let loading = $state(true);
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);

  // Editor sheet
  let editing = $state<ServerEntry | null>(null);
  let isNew = $state(false);
  let formName = $state("");
  let formType = $state<"stdio" | "sse">("stdio");
  let formCommand = $state("");
  let formArgs = $state("");
  let formUrl = $state("");

  // Delete
  let deletingServerName = $state<string | null>(null);
  let galleryOpen = $state(false);

  // Bulk selection
  let selectedNames = $state<Set<string>>(new Set());

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(scope === "project" || scope === "mcp-local");
  const scopeLabel: Record<string, string> = {
    "desktop": "Claude Desktop",
    "global": "Claude Code (user)",
    "mcp-local": "Project (.mcp.json)",
    "project": "Project (.claude/settings.json)",
  };

  // Move / copy to another place (one server or a bulk selection)
  let movingServers = $state<ServerEntry[]>([]);
  let destScope = $state<SettingsScope>("global");
  let destProjectPath = $state("");
  let destProjectSearch = $state("");
  let destShowCustomPath = $state(false);
  let transferring = $state(false);
  let transferError = $state<string | null>(null);

  const destNeedsProject = $derived(destScope === "project" || destScope === "mcp-local");
  const destProjects = $derived(
    getProjects().filter((p) => {
      if (!destProjectSearch) return true;
      return p.path.toLowerCase().includes(destProjectSearch.toLowerCase());
    }),
  );
  const isSameDestination = $derived(
    movingServers.length > 0 &&
      destScope === scope &&
      (destNeedsProject ? destProjectPath : "") === (needsProject ? (projectPath ?? "") : ""),
  );

  async function loadServers() {
    if (needsProject && !projectPath) { loading = false; servers = []; return; }
    loading = true;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      const [raw, cloud] = await Promise.all([
        api.mcp.list(scope, pp) as Promise<Record<string, Record<string, unknown>>>,
        api.mcp.getCloudMcps(),
      ]);
      servers = Object.entries(raw).map(([name, config]) => ({ name, config }));
      cloudMcps = cloud;
    } catch (e) {
      console.error("Failed:", e);
      servers = [];
    } finally {
      loading = false;
    }
  }

  function editServer(server: ServerEntry) {
    editing = server;
    isNew = false;
    formName = server.name;
    if ("url" in server.config) {
      formType = "sse";
      formUrl = (server.config.url as string) ?? "";
      formCommand = "";
      formArgs = "";
    } else {
      formType = "stdio";
      formCommand = (server.config.command as string) ?? "";
      formArgs = ((server.config.args as string[]) ?? []).join(" ");
      formUrl = "";
    }
  }

  function newServer(template?: { name: string; type: "stdio" | "sse"; command: string; args: string }) {
    editing = { name: "", config: {} };
    isNew = true;
    formName = template?.name ?? "";
    formType = template?.type ?? "stdio";
    formCommand = template?.command ?? "";
    formArgs = template?.args ?? "";
    formUrl = "";
  }

  async function saveServer() {
    if (!formName.trim()) return;
    saving = true;
    saveMessage = null;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      const config = formType === "stdio"
        ? { command: formCommand, args: formArgs.split(/\s+/).filter(Boolean) }
        : { url: formUrl };
      await api.mcp.upsert(scope, formName.trim(), config, pp);
      await loadServers();
      editing = null;
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function deleteServer(name: string) {
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      await api.mcp.delete(scope, name, pp);
      await loadServers();
      if (editing?.name === name) editing = null;
      if (selectedNames.has(name)) {
        const next = new Set(selectedNames);
        next.delete(name);
        selectedNames = next;
      }
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deletingServerName = null;
    }
  }

  function toggleSelectServer(name: string) {
    const next = new Set(selectedNames);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selectedNames = next;
  }

  function clearSelection() {
    selectedNames = new Set();
  }

  function openMoveDialogFor(entries: ServerEntry[]) {
    if (entries.length === 0) return;
    movingServers = entries;
    // Default destination to a different scope than the source so the
    // dialog doesn't open pre-aimed at a no-op transfer.
    destScope = scope === "global" ? "desktop" : "global";
    destProjectPath = "";
    destProjectSearch = "";
    destShowCustomPath = false;
    transferError = null;
    if (!isLoaded()) loadProjects();
  }

  function openMoveDialog(server: ServerEntry) {
    openMoveDialogFor([server]);
  }

  function openBulkMoveDialog() {
    openMoveDialogFor(servers.filter((s) => selectedNames.has(s.name)));
  }

  // Names that already exist at the destination, pending overwrite confirmation
  let overwriteNames = $state<string[]>([]);
  let pendingMode = $state<"move" | "copy">("copy");

  async function transferServer(mode: "move" | "copy", confirmedOverwrite = false) {
    if (movingServers.length === 0 || isSameDestination) return;
    if (destNeedsProject && !destProjectPath.trim()) {
      transferError = "Pick a destination project";
      return;
    }
    transferring = true;
    transferError = null;
    const destPP = destNeedsProject ? destProjectPath.trim() : undefined;
    const srcPP = needsProject ? projectPath ?? undefined : undefined;

    if (!confirmedOverwrite) {
      try {
        const existing = await api.mcp.list(destScope, destPP);
        const clashes = movingServers.filter((s) => s.name in existing).map((s) => s.name);
        if (clashes.length > 0) {
          overwriteNames = clashes;
          pendingMode = mode;
          transferring = false;
          return;
        }
      } catch {
        // Destination unreadable (e.g. file doesn't exist yet): nothing to overwrite
      }
    }

    const results = await Promise.allSettled(
      movingServers.map(async (s) => {
        await api.mcp.upsert(destScope, s.name, s.config, destPP);
        if (mode === "move") {
          await api.mcp.delete(scope, s.name, srcPP);
        }
      }),
    );

    const failedCount = results.filter((r) => r.status === "rejected").length;
    const transferredNames = new Set(
      movingServers.filter((_, i) => results[i].status === "fulfilled").map((s) => s.name),
    );

    await loadServers();
    selectedNames = new Set([...selectedNames].filter((n) => !transferredNames.has(n)));

    if (failedCount > 0) {
      transferError = `${failedCount} of ${movingServers.length} server${movingServers.length === 1 ? "" : "s"} failed to transfer`;
      movingServers = movingServers.filter((s) => !transferredNames.has(s.name));
    } else {
      movingServers = [];
    }
    transferring = false;
  }

  // Live testing: spawn the configured server, list its tools, call them
  let testingServer = $state<ServerEntry | null>(null);
  let testResult = $state<McpTestResult | null>(null);
  let testError = $state<string | null>(null);
  let testLoading = $state(false);
  let toolArgs = $state<Record<string, string>>({});
  let toolResults = $state<Record<string, string>>({});
  let toolRunning = $state<string | null>(null);

  async function openTest(server: ServerEntry) {
    testingServer = server;
    testResult = null;
    testError = null;
    toolArgs = {};
    toolResults = {};
    testLoading = true;
    try {
      testResult = await api.mcp.test(server.config);
    } catch (e) {
      testError = String(e);
    } finally {
      testLoading = false;
    }
  }

  async function runTool(name: string) {
    if (!testingServer || toolRunning) return;
    toolRunning = name;
    let args: unknown = {};
    const raw = toolArgs[name]?.trim();
    if (raw) {
      try {
        args = JSON.parse(raw);
      } catch {
        toolResults = { ...toolResults, [name]: "Arguments must be valid JSON" };
        toolRunning = null;
        return;
      }
    }
    try {
      const res = await api.mcp.callTool(testingServer.config, name, args);
      toolResults = { ...toolResults, [name]: JSON.stringify(res, null, 2) };
    } catch (e) {
      toolResults = { ...toolResults, [name]: `Error: ${e}` };
    } finally {
      toolRunning = null;
    }
  }

  onMount(loadServers);
</script>

{#if testingServer}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button class="absolute inset-0 bg-black/50" onclick={() => (testingServer = null)} aria-label="Close dialog"></button>
    <div class="relative bg-bg-secondary border border-border rounded-xl shadow-2xl w-[600px] max-h-[75vh] p-6 z-10 flex flex-col">
      <div class="flex items-center justify-between mb-1">
        <h3 class="text-base font-semibold text-text-primary flex items-center gap-2">
          <FlaskConical size={16} class="text-accent" />
          Test {testingServer.name}
        </h3>
        <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => (testingServer = null)} aria-label="Close">
          <X size={16} />
        </button>
      </div>

      {#if testLoading}
        <p class="text-sm text-text-muted py-6">Connecting to server...</p>
      {:else if testError}
        <p class="text-xs text-danger font-mono whitespace-pre-wrap py-4">{testError}</p>
      {:else if testResult}
        <p class="text-xs text-text-muted mb-3">
          Connected{#if testResult.serverInfo?.name}
            to <span class="text-text-secondary">{testResult.serverInfo.name} {testResult.serverInfo.version ?? ""}</span>{/if}
          {#if testResult.protocolVersion}(protocol {testResult.protocolVersion}){/if}
          · {testResult.tools.length} tool{testResult.tools.length === 1 ? "" : "s"}
        </p>
        <div class="overflow-y-auto space-y-2">
          {#each testResult.tools as tool (tool.name)}
            <details class="bg-bg-tertiary rounded-lg">
              <summary class="px-3 py-2 cursor-pointer">
                <span class="text-sm font-mono text-text-primary">{tool.name}</span>
                {#if tool.description}
                  <span class="text-xs text-text-muted ml-2">{tool.description.slice(0, 80)}</span>
                {/if}
              </summary>
              <div class="px-3 pb-3 space-y-2">
                <textarea
                  class="w-full h-16 px-2 py-1.5 text-xs bg-bg-secondary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent resize-y"
                  placeholder={'Arguments as JSON, e.g. {"query": "test"}'}
                  bind:value={toolArgs[tool.name]}
                ></textarea>
                <button
                  class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
                  onclick={() => runTool(tool.name)}
                  disabled={toolRunning !== null}
                >
                  <Play size={12} />
                  {toolRunning === tool.name ? "Running..." : "Run"}
                </button>
                {#if toolResults[tool.name]}
                  <pre class="text-xs text-text-secondary font-mono whitespace-pre-wrap bg-bg-secondary rounded-md p-2 max-h-48 overflow-y-auto">{toolResults[tool.name]}</pre>
                {/if}
              </div>
            </details>
          {:else}
            <p class="text-sm text-text-muted">Server exposes no tools.</p>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}

<ConfirmDialog
  open={overwriteNames.length > 0}
  title="Overwrite Existing Server{overwriteNames.length === 1 ? '' : 's'}"
  message="{overwriteNames.join(', ')} already exist{overwriteNames.length === 1 ? 's' : ''} at the destination and will be overwritten."
  confirmLabel="Overwrite"
  onconfirm={() => { overwriteNames = []; transferServer(pendingMode, true); }}
  oncancel={() => (overwriteNames = [])}
/>

<ConfirmDialog
  open={deletingServerName !== null}
  title="Delete Server"
  message="The server '{deletingServerName}' will be permanently removed."
  onconfirm={() => { if (deletingServerName) deleteServer(deletingServerName); }}
  oncancel={() => (deletingServerName = null)}
/>

{#if movingServers.length > 0}
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button
      class="absolute inset-0 bg-black/50"
      onclick={() => !transferring && (movingServers = [])}
      aria-label="Close dialog"
    ></button>

    <div class="relative bg-bg-secondary border border-border rounded-xl shadow-2xl w-[440px] p-6 space-y-4 z-10">
      <div class="flex items-start justify-between">
        <div>
          <h3 class="text-base font-semibold text-text-primary">Move / Copy {movingServers.length === 1 ? "Server" : `${movingServers.length} Servers`}</h3>
          <p class="text-sm text-text-muted mt-1">
            {#if movingServers.length === 1}
              <span class="font-mono text-text-secondary">{movingServers[0].name}</span>
            {:else}
              <span class="text-text-secondary">{movingServers.map((s) => s.name).join(", ")}</span>
            {/if}
            from <span class="text-text-secondary">{scopeLabel[scope] ?? scope}</span>{#if needsProject && projectPath}
              <span class="font-mono text-text-secondary"> ({projectPath.split("/").pop()})</span>
            {/if}
          </p>
        </div>
        <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => (movingServers = [])} aria-label="Close">
          <X size={16} />
        </button>
      </div>

      <div>
        <span class="text-xs text-text-muted">Destination</span>
        <div class="flex gap-1 mt-1 bg-bg-tertiary rounded-lg p-1">
          {#each [{ id: "desktop" as const, label: "Desktop" }, { id: "global" as const, label: "Global" }, { id: "mcp-local" as const, label: "Local" }, { id: "project" as const, label: "Project" }] as tab}
            <button
              class="flex-1 px-2 py-1.5 text-xs rounded-md transition-colors {destScope === tab.id ? 'bg-bg-secondary text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
              onclick={() => { destScope = tab.id; transferError = null; }}
            >{tab.label}</button>
          {/each}
        </div>
      </div>

      {#if destNeedsProject}
        <div>
          <span class="text-xs text-text-muted">Destination project</span>
          {#if destProjectPath}
            <div class="flex items-center justify-between mt-1 px-3 py-1.5 bg-bg-tertiary border border-border rounded-md">
              <span class="text-xs font-mono text-text-primary truncate">{destProjectPath}</span>
              <button class="text-xs text-accent shrink-0 ml-2" onclick={() => (destProjectPath = "")}>Change</button>
            </div>
          {:else}
            <input
              type="text"
              class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
              placeholder="Search projects..."
              bind:value={destProjectSearch}
            />
            <div class="max-h-40 overflow-y-auto mt-1 border border-border rounded-md">
              {#each destProjects as project}
                <button
                  class="w-full text-left px-3 py-1.5 text-xs hover:bg-bg-hover transition-colors font-mono text-text-secondary truncate block"
                  onclick={() => (destProjectPath = project.path)}
                >{project.path}</button>
              {:else}
                <p class="px-3 py-2 text-xs text-text-muted">No matching projects</p>
              {/each}
            </div>
            {#if destShowCustomPath}
              <div class="flex gap-2 mt-1">
                <input
                  type="text"
                  class="flex-1 px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent"
                  placeholder="/path/to/project"
                  bind:value={destProjectSearch}
                  onkeydown={(e) => { if (e.key === "Enter" && destProjectSearch.trim()) destProjectPath = destProjectSearch.trim(); }}
                />
                <button class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md" onclick={() => { if (destProjectSearch.trim()) destProjectPath = destProjectSearch.trim(); }}>Use</button>
              </div>
            {:else}
              <button class="flex items-center gap-1.5 mt-1 text-xs text-text-muted hover:text-text-secondary" onclick={() => (destShowCustomPath = true)}>
                <FolderOpen size={12} />
                Enter path manually...
              </button>
            {/if}
          {/if}
        </div>
      {/if}

      {#if isSameDestination}
        <p class="text-xs text-warning">Destination is the same as the source.</p>
      {/if}
      {#if transferError}
        <p class="text-xs text-danger">{transferError}</p>
      {/if}

      <div class="flex justify-end gap-2 pt-2">
        <button class="px-4 py-2 text-sm text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-lg transition-colors" onclick={() => (movingServers = [])} disabled={transferring}>
          Cancel
        </button>
        <button
          class="flex items-center gap-1.5 px-4 py-2 text-sm text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-lg transition-colors disabled:opacity-50"
          onclick={() => transferServer("copy")}
          disabled={transferring || isSameDestination || (destNeedsProject && !destProjectPath.trim())}
        >
          <Copy size={14} />
          {transferring ? "Working..." : "Copy"}
        </button>
        <button
          class="flex items-center gap-1.5 px-4 py-2 text-sm text-white bg-accent hover:bg-accent-hover rounded-lg transition-colors disabled:opacity-50"
          onclick={() => transferServer("move")}
          disabled={transferring || isSameDestination || (destNeedsProject && !destProjectPath.trim())}
        >
          <ArrowRightLeft size={14} />
          {transferring ? "Working..." : "Move"}
        </button>
      </div>
    </div>
  </div>
{/if}

<div class="flex h-full">
  <!-- Main content -->
  <div class="flex-1 overflow-y-auto p-6 space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
          {#each [{ id: "desktop" as const, label: "Desktop" }, { id: "global" as const, label: "Global" }, { id: "mcp-local" as const, label: "Local" }, { id: "project" as const, label: "Project" }] as tab}
            <button
              class="px-4 py-1.5 text-sm rounded-md transition-colors {scope === tab.id ? 'bg-bg-secondary text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
              onclick={() => { scope = tab.id; editing = null; selectedNames = new Set(); loadServers(); }}
            >{tab.label}</button>
          {/each}
        </div>
        {#if needsProject}
          <ProjectPicker onselect={() => { selectedNames = new Set(); loadServers(); }} />
        {/if}
      </div>
      <div class="flex items-center gap-3">
        {#if selectedNames.size > 0}
          <span class="text-xs text-text-secondary">{selectedNames.size} selected</span>
          <button class="text-xs text-text-muted hover:text-text-secondary" onclick={clearSelection}>Clear</button>
          <button
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 hover:text-accent transition-colors"
            onclick={openBulkMoveDialog}
          >
            <ArrowRightLeft size={14} />
            Move / Copy
          </button>
        {/if}
        {#if saveMessage}
          <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
        {/if}
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 hover:text-accent transition-colors"
          onclick={() => (galleryOpen = true)}
        >
          <LayoutGrid size={14} />
          Templates
        </button>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
          onclick={() => newServer()}
        >
          <Plus size={14} />
          Add Server
        </button>
      </div>
    </div>

    {#if needsProject && !projectPath}
      <div class="flex items-center justify-center h-48 text-sm text-text-muted">Select a project</div>
    {:else if loading}
      <p class="text-sm text-text-muted">Loading...</p>
    {:else}
      <!-- Cloud MCPs -->
      {#if cloudMcps.length > 0}
        <div>
          <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-2 flex items-center gap-1.5">
            <Cloud size={12} />
            Cloud (configured at claude.ai)
          </h3>
          <div class="grid grid-cols-2 gap-2">
            {#each cloudMcps as name}
              <div class="bg-bg-secondary border border-border rounded-lg p-3 flex items-center gap-3 opacity-70">
                <div class="w-8 h-8 rounded-lg bg-info/10 flex items-center justify-center">
                  <Cloud size={16} class="text-info" />
                </div>
                <div>
                  <p class="text-sm text-text-primary">{name}</p>
                  <p class="text-xs text-text-muted">Managed via claude.ai</p>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      <!-- Local MCPs -->
      <div>
        <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-2 flex items-center gap-1.5">
          <Terminal size={12} />
          {scopeLabel[scope] ?? scope}
        </h3>

        {#if servers.length > 0}
          <div class="space-y-2">
            {#each servers as server}
              {@const isStdio = "command" in server.config}
              <div
                class="bg-bg-secondary border rounded-lg p-4 flex items-center gap-4 group cursor-pointer transition-colors hover:border-accent/30
                  {editing?.name === server.name ? 'border-accent/50' : 'border-border'}"
                role="button" tabindex="0"
                onclick={() => editServer(server)}
                onkeydown={(e) => e.key === "Enter" && editServer(server)}
              >
                <label class="shrink-0 cursor-pointer">
                  <input
                    type="checkbox"
                    class="accent-accent"
                    checked={selectedNames.has(server.name)}
                    onclick={(e) => e.stopPropagation()}
                    onchange={() => toggleSelectServer(server.name)}
                    aria-label="Select server"
                  />
                </label>
                <div class="w-10 h-10 rounded-lg flex items-center justify-center shrink-0
                  {isStdio ? 'bg-success/10' : 'bg-accent/10'}">
                  {#if isStdio}
                    <Terminal size={18} class="text-success" />
                  {:else}
                    <Globe size={18} class="text-accent" />
                  {/if}
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-text-primary">{server.name}</p>
                  <p class="text-xs text-text-muted font-mono truncate">
                    {isStdio
                      ? `${server.config.command} ${((server.config.args as string[]) ?? []).join(" ")}`
                      : server.config.url}
                  </p>
                </div>
                <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  {#if isStdio}
                    <button class="p-1.5 rounded hover:bg-bg-hover text-text-muted" onclick={(e) => { e.stopPropagation(); openTest(server); }} aria-label="Test server" title="Connect and try its tools">
                      <FlaskConical size={14} />
                    </button>
                  {/if}
                  <button class="p-1.5 rounded hover:bg-bg-hover text-text-muted" onclick={(e) => { e.stopPropagation(); openMoveDialog(server); }} aria-label="Move or copy to another place" title="Move / copy to another place">
                    <ArrowRightLeft size={14} />
                  </button>
                  <button class="p-1.5 rounded hover:bg-bg-hover text-text-muted" onclick={(e) => { e.stopPropagation(); editServer(server); }} aria-label="Edit">
                    <Edit3 size={14} />
                  </button>
                  <button class="p-1.5 rounded hover:bg-bg-hover text-text-muted hover:text-danger" onclick={(e) => { e.stopPropagation(); deletingServerName = server.name; }} aria-label="Delete">
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="bg-bg-secondary border border-border rounded-lg p-8 text-center">
            <Server size={24} class="mx-auto mb-3 opacity-20 text-text-muted" />
            <p class="text-sm text-text-muted mb-1">No MCP servers configured</p>
            <p class="text-xs text-text-muted">Add a server or browse templates</p>
          </div>
        {/if}
      </div>

    {/if}
  </div>

  <!-- Editor Sheet -->
  {#if editing}
    <div class="w-[400px] shrink-0 border-l border-border flex flex-col bg-bg-secondary">
      <div class="flex items-center justify-between px-4 py-3 border-b border-border">
        <span class="text-sm font-medium text-text-primary">{isNew ? "Add Server" : `Edit: ${editing.name}`}</span>
        <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => (editing = null)} aria-label="Close">
          <X size={16} />
        </button>
      </div>

      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <label class="block">
          <span class="text-xs text-text-muted">Server Name</span>
          <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" bind:value={formName} disabled={!isNew} />
        </label>

        <div>
          <span class="text-xs text-text-muted">Type</span>
          <div class="flex gap-1 mt-1" role="group" aria-label="Server type">
            {#each [{ id: "stdio" as const, label: "Command (stdio)", icon: Terminal }, { id: "sse" as const, label: "URL (SSE/HTTP)", icon: Globe }] as t}
              <button
                class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-md transition-colors flex-1
                  {formType === t.id ? 'bg-accent text-white' : 'bg-bg-tertiary text-text-muted'}"
                onclick={() => (formType = t.id)}
              >
                <t.icon size={12} />
                {t.label}
              </button>
            {/each}
          </div>
        </div>

        {#if formType === "stdio"}
          <label class="block">
            <span class="text-xs text-text-muted">Command</span>
            <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="npx, node, python..." bind:value={formCommand} />
          </label>
          <label class="block">
            <span class="text-xs text-text-muted">Arguments</span>
            <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="-y @modelcontextprotocol/server-..." bind:value={formArgs} />
            <p class="text-[10px] text-text-muted mt-1">Space-separated arguments</p>
          </label>
        {:else}
          <label class="block">
            <span class="text-xs text-text-muted">URL</span>
            <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="http://localhost:8000/sse" bind:value={formUrl} />
          </label>
        {/if}
      </div>

      <div class="px-4 py-3 border-t border-border flex justify-end gap-2">
        <button class="px-4 py-1.5 text-sm text-text-muted hover:text-text-secondary" onclick={() => (editing = null)}>Cancel</button>
        <button
          class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
          onclick={saveServer} disabled={saving || !formName.trim()}
        >{saving ? "Saving..." : "Save"}</button>
      </div>
    </div>
  {/if}
</div>

<TemplateGallery
  open={galleryOpen}
  defaultCategory="mcp"
  onselect={async (template) => {
    const name = template.name.toLowerCase().replace(/\s+/g, "-");
    if (template.mcpUrl) {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      await api.mcp.upsert(scope, name, { url: template.mcpUrl }, pp);
      await loadServers();
    } else {
      newServer({
        name,
        type: (template.mcpType ?? "stdio") as "stdio" | "sse",
        command: template.mcpCommand ?? "",
        args: template.mcpArgs ?? "",
      });
    }
  }}
  onclose={() => (galleryOpen = false)}
/>
