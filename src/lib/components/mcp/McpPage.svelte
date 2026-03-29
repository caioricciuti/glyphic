<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { SettingsScope } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { Server, Plus, Cloud, Terminal, Globe, Trash2, Edit3, X, LayoutGrid } from "lucide-svelte";
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

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(scope === "project" || scope === "mcp-local");
  const scopeLabel: Record<string, string> = {
    "desktop": "Claude Desktop",
    "global": "Claude Code (user)",
    "mcp-local": "Project (.mcp.json)",
    "project": "Project (.claude/settings.json)",
  };

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
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deletingServerName = null;
    }
  }

  onMount(loadServers);
</script>

<ConfirmDialog
  open={deletingServerName !== null}
  title="Delete Server"
  message="The server '{deletingServerName}' will be permanently removed."
  onconfirm={() => { if (deletingServerName) deleteServer(deletingServerName); }}
  oncancel={() => (deletingServerName = null)}
/>

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
              onclick={() => { scope = tab.id; editing = null; loadServers(); }}
            >{tab.label}</button>
          {/each}
        </div>
        {#if needsProject}
          <ProjectPicker onselect={loadServers} />
        {/if}
      </div>
      <div class="flex items-center gap-3">
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
