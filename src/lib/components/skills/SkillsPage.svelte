<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { SkillInfo } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { marked } from "marked";
  import {
    Sparkles, Bot, Plus, Search, X, Trash2,
    Zap, Server, Brain, Shield, Wrench, BookOpen,
    Terminal, Eye, EyeOff, GitFork, Database, LayoutGrid,
  } from "lucide-svelte";
  import TemplateGallery from "$lib/components/shared/TemplateGallery.svelte";

  let activeTab = $state<"skills" | "agents">("skills");
  let scope = $state<"global" | "project">("global");
  let items = $state<SkillInfo[]>([]);
  let selected = $state<SkillInfo | null>(null);
  let loading = $state(true);
  let searchQuery = $state("");

  // Editor
  let editing = $state(false);
  let isNew = $state(false);
  let editName = $state("");
  let editContent = $state(""); // full raw content including frontmatter
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let previewMode = $state(false);
  let galleryOpen = $state(false);

  // Delete
  let deleteDialogOpen = $state(false);

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(scope === "project");
  const isAgent = $derived(activeTab === "agents");

  const filteredItems = $derived(
    items.filter((i) => !searchQuery || i.name.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  // Parse frontmatter from content for display
  function parseFrontmatter(raw: string): { meta: Record<string, string | string[] | boolean>; body: string } {
    if (!raw.startsWith("---")) return { meta: {}, body: raw };
    const end = raw.indexOf("---", 3);
    if (end === -1) return { meta: {}, body: raw };
    const front = raw.slice(3, end);
    const body = raw.slice(end + 3).trim();
    const meta: Record<string, string | string[] | boolean> = {};
    let currentKey = "";
    for (const line of front.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed) continue;
      if (trimmed.startsWith("- ") && currentKey) {
        const existing = meta[currentKey];
        if (Array.isArray(existing)) {
          existing.push(trimmed.slice(2).trim());
        } else {
          meta[currentKey] = [trimmed.slice(2).trim()];
        }
      } else {
        const colonIdx = trimmed.indexOf(":");
        if (colonIdx > 0) {
          const key = trimmed.slice(0, colonIdx).trim();
          const val = trimmed.slice(colonIdx + 1).trim();
          currentKey = key;
          if (val === "true") meta[key] = true;
          else if (val === "false") meta[key] = false;
          else if (val) meta[key] = val;
        }
      }
    }
    return { meta, body };
  }

  const selectedParsed = $derived(selected ? parseFrontmatter(selected.content) : null);

  async function loadItems() {
    if (needsProject && !projectPath) { loading = false; items = []; return; }
    loading = true;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      items = activeTab === "skills"
        ? await api.skills.list(scope, pp)
        : await api.agents.list(scope, pp);
    } catch (e) { console.error("Failed:", e); items = []; }
    finally { loading = false; }
  }

  function selectItem(item: SkillInfo) {
    selected = item;
    editing = false;
    isNew = false;
  }

  function startEdit() {
    if (!selected) return;
    editContent = selected.content;
    editName = selected.name;
    editing = true;
    isNew = false;
    previewMode = false;
  }

  function startCreate() {
    const type = activeTab === "skills" ? "skill" : "agent";
    const template = type === "skill"
      ? `---\nname: my-${type}\ndescription: What this ${type} does\nuser-invocable: true\n---\n\n# Instructions\n\nDescribe what Claude should do when this ${type} is invoked.\n`
      : `---\nname: my-${type}\ndescription: When to delegate to this agent\nmodel: sonnet\ntools: Read, Glob, Grep\n---\n\n# System Prompt\n\nYou are a specialized assistant that...\n`;
    editContent = template;
    editName = "";
    editing = true;
    isNew = true;
    selected = null;
    previewMode = false;
  }

  async function save() {
    const name = isNew ? editName.trim() : selected?.name;
    if (!name) return;
    saving = true;
    saveMessage = null;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      if (activeTab === "skills") await api.skills.write(scope, name, editContent, pp);
      else await api.agents.write(scope, name, editContent, pp);
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
      await loadItems();
      if (isNew) {
        selected = items.find((i) => i.name === name) ?? null;
        editing = false;
        isNew = false;
      }
    } catch (e) { saveMessage = `Error: ${e}`; }
    finally { saving = false; }
  }

  async function deleteItem() {
    if (!selected) return;
    try {
      if (activeTab === "skills") await api.skills.delete(scope, selected.name);
      else await api.agents.delete(scope, selected.name);
      selected = null;
      editing = false;
      await loadItems();
    } catch (e) { console.error("Failed:", e); }
    finally { deleteDialogOpen = false; }
  }

  function handleTabChange(tab: typeof activeTab) {
    activeTab = tab;
    selected = null;
    editing = false;
    loadItems();
  }

  onMount(loadItems);
</script>

<ConfirmDialog
  open={deleteDialogOpen}
  title="Delete {isAgent ? 'Agent' : 'Skill'}"
  message="This {isAgent ? 'agent' : 'skill'} and all supporting files will be permanently deleted."
  onconfirm={deleteItem}
  oncancel={() => (deleteDialogOpen = false)}
/>

<div class="flex h-full">
  <!-- Sidebar: item list -->
  <div class="w-64 shrink-0 border-r border-border flex flex-col bg-bg-secondary">
    <!-- Tabs -->
    <div class="p-3 border-b border-border space-y-2">
      <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
        <button class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs rounded-md transition-colors {activeTab === 'skills' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => handleTabChange("skills")}>
          <Sparkles size={12} /> Skills
        </button>
        <button class="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 text-xs rounded-md transition-colors {activeTab === 'agents' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => handleTabChange("agents")}>
          <Bot size={12} /> Agents
        </button>
      </div>
      <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
        <button class="flex-1 px-2 py-1 text-[10px] rounded transition-colors {scope === 'global' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { scope = "global"; loadItems(); }}>Global</button>
        <button class="flex-1 px-2 py-1 text-[10px] rounded transition-colors {scope === 'project' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { scope = "project"; loadItems(); }}>Project</button>
      </div>
      {#if needsProject}
        <ProjectPicker onselect={loadItems} />
      {/if}
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2 text-text-muted" />
        <input type="text" class="w-full pl-8 pr-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search..." bind:value={searchQuery} />
      </div>
    </div>

    <!-- Item list -->
    <div class="flex-1 overflow-y-auto py-1">
      {#if loading}
        <p class="text-xs text-text-muted px-3 py-4 text-center">Loading...</p>
      {:else if needsProject && !projectPath}
        <p class="text-xs text-text-muted px-3 py-4 text-center">Select a project</p>
      {:else}
        {#each filteredItems as item}
          {@const parsed = parseFrontmatter(item.content)}
          <button
            class="w-full text-left px-3 py-2.5 transition-colors border-b border-border/50
              {selected?.name === item.name ? 'bg-accent/10 text-accent' : 'text-text-secondary hover:bg-bg-hover'}"
            onclick={() => selectItem(item)}
          >
            <div class="flex items-center gap-2">
              {#if isAgent}
                <Bot size={14} class="shrink-0 text-accent" />
              {:else}
                <Sparkles size={14} class="shrink-0 text-warning" />
              {/if}
              <span class="text-sm font-medium truncate">{item.name}</span>
            </div>
            {#if parsed.meta.description}
              <p class="text-[10px] text-text-muted mt-0.5 truncate ml-[22px]">{parsed.meta.description}</p>
            {/if}
            <!-- Feature badges -->
            <div class="flex gap-1 mt-1 ml-[22px]">
              {#if parsed.meta.model}
                <span class="text-[9px] px-1 py-0.5 rounded bg-info/10 text-info">{parsed.meta.model}</span>
              {/if}
              {#if parsed.meta.hooks}
                <span class="text-[9px] px-1 py-0.5 rounded bg-warning/10 text-warning">hooks</span>
              {/if}
              {#if parsed.meta.memory}
                <span class="text-[9px] px-1 py-0.5 rounded bg-success/10 text-success">memory</span>
              {/if}
              {#if parsed.meta.mcp}
                <span class="text-[9px] px-1 py-0.5 rounded bg-accent/10 text-accent">mcp</span>
              {/if}
            </div>
          </button>
        {/each}

      {/if}
    </div>

    <!-- Create + Templates buttons -->
    <div class="p-3 border-t border-border flex gap-2">
      <button
        class="flex-1 flex items-center justify-center gap-1.5 py-2 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
        onclick={startCreate}
      >
        <Plus size={14} />
        Create
      </button>
      <button
        class="flex items-center justify-center gap-1.5 px-3 py-2 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 hover:text-accent transition-colors"
        onclick={() => (galleryOpen = true)}
      >
        <LayoutGrid size={14} />
        Templates
      </button>
    </div>
  </div>

  <!-- Main content: detail + editor -->
  <div class="flex-1 flex min-w-0">
    {#if editing || isNew}
      <!-- Editor -->
      <div class="flex-1 flex flex-col">
        <div class="flex items-center justify-between px-6 py-3 border-b border-border shrink-0">
          <div class="flex items-center gap-3">
            {#if isNew}
              <label class="block">
                <span class="text-xs text-text-muted">Name</span>
                <input type="text" class="ml-2 px-3 py-1 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="my-{activeTab === 'skills' ? 'skill' : 'agent'}" bind:value={editName} />
              </label>
            {:else}
              <span class="text-sm font-medium text-text-primary">{selected?.name}</span>
            {/if}
          </div>
          <div class="flex items-center gap-2">
            {#if saveMessage}
              <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
            {/if}
            <div class="flex gap-1 bg-bg-tertiary rounded-lg p-0.5">
              <button class="px-2 py-1 text-[10px] rounded {!previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => (previewMode = false)}>Edit</button>
              <button class="px-2 py-1 text-[10px] rounded {previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => (previewMode = true)}>Preview</button>
            </div>
            <button class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50" onclick={save} disabled={saving || (isNew && !editName.trim())}>{saving ? "..." : "Save"}</button>
            <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => { editing = false; isNew = false; }} aria-label="Close">
              <X size={16} />
            </button>
          </div>
        </div>

        <!-- Help bar for skills -->
        {#if activeTab === "skills" && !previewMode}
          <div class="px-6 py-2 bg-bg-tertiary/50 border-b border-border text-[10px] text-text-muted flex gap-4">
            <span><code class="text-accent">$ARGUMENTS</code> = user input</span>
            <span><code class="text-accent">$0</code>, <code class="text-accent">$1</code> = positional args</span>
            <span><code class="text-accent">!`cmd`</code> = run before prompt</span>
            <span><code class="text-accent">${"${CLAUDE_SKILL_DIR}"}</code> = skill directory</span>
          </div>
        {/if}

        <div class="flex-1 overflow-hidden p-4">
          {#if previewMode}
            <div class="h-full overflow-y-auto md-preview px-4">
              {@html marked(editContent.replace(/^---[\s\S]*?---\n*/m, "")) as string}
            </div>
          {:else}
            <textarea
              class="w-full h-full px-4 py-3 text-sm bg-bg-secondary border border-border rounded-lg text-text-primary font-mono resize-none focus:outline-none focus:border-accent"
              placeholder="---\nname: my-{activeTab === 'skills' ? 'skill' : 'agent'}\ndescription: ...\n---\n\n# Instructions..."
              bind:value={editContent}
            ></textarea>
          {/if}
        </div>
      </div>
    {:else if selected && selectedParsed}
      <!-- Detail view -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        <!-- Header -->
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg flex items-center justify-center {isAgent ? 'bg-accent/10' : 'bg-warning/10'}">
              {#if isAgent}
                <Bot size={20} class="text-accent" />
              {:else}
                <Sparkles size={20} class="text-warning" />
              {/if}
            </div>
            <div>
              <h2 class="text-lg font-semibold text-text-primary">{selected.name}</h2>
              {#if selectedParsed.meta.description}
                <p class="text-sm text-text-muted">{selectedParsed.meta.description}</p>
              {/if}
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button class="px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30" onclick={startEdit}>
              Edit
            </button>
            <button class="p-1.5 text-text-muted hover:text-danger rounded" onclick={() => (deleteDialogOpen = true)} aria-label="Delete">
              <Trash2 size={14} />
            </button>
          </div>
        </div>

        <!-- Config cards grid -->
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
          <!-- Model -->
          {#if selectedParsed.meta.model}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Terminal size={12} />
                <span>Model</span>
              </div>
              <p class="text-sm font-medium text-info">{selectedParsed.meta.model}</p>
            </div>
          {/if}

          <!-- Effort -->
          {#if selectedParsed.meta.effort}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Zap size={12} />
                <span>Effort</span>
              </div>
              <p class="text-sm font-medium text-warning">{selectedParsed.meta.effort}</p>
            </div>
          {/if}

          <!-- Tools (agents) -->
          {#if selectedParsed.meta.tools}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Wrench size={12} />
                <span>Tools</span>
              </div>
              <div class="flex flex-wrap gap-1">
                {#each String(selectedParsed.meta.tools).split(",").map(t => t.trim()) as tool}
                  <span class="text-[10px] px-1.5 py-0.5 rounded bg-bg-tertiary text-text-secondary">{tool}</span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Permissions (agents) -->
          {#if selectedParsed.meta.permissions}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Shield size={12} />
                <span>Permissions</span>
              </div>
              <p class="text-sm font-medium text-accent">{selectedParsed.meta.permissions}</p>
            </div>
          {/if}

          <!-- Memory (agents) -->
          {#if selectedParsed.meta.memory}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Brain size={12} />
                <span>Memory</span>
              </div>
              <p class="text-sm font-medium text-success">{selectedParsed.meta.memory} scope</p>
            </div>
          {/if}

          <!-- Context (skills) -->
          {#if selectedParsed.meta.context}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <GitFork size={12} />
                <span>Context</span>
              </div>
              <p class="text-sm font-medium text-accent">{selectedParsed.meta.context}
                {#if selectedParsed.meta.agent}
                  → {selectedParsed.meta.agent}
                {/if}
              </p>
            </div>
          {/if}

          <!-- Invocation (skills) -->
          {#if activeTab === "skills"}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                {#if selectedParsed.meta["user-invocable"] === false}
                  <EyeOff size={12} />
                {:else}
                  <Eye size={12} />
                {/if}
                <span>Invocation</span>
              </div>
              <p class="text-sm text-text-secondary">
                {#if selectedParsed.meta["disable-model-invocation"]}
                  Manual only (/<span class="text-accent">{selected.name}</span>)
                {:else if selectedParsed.meta["user-invocable"] === false}
                  Auto only (Claude invokes)
                {:else}
                  /<span class="text-accent">{selected.name}</span>
                  {#if selectedParsed.meta["argument-hint"]}
                    <span class="text-text-muted"> {selectedParsed.meta["argument-hint"]}</span>
                  {/if}
                {/if}
              </p>
            </div>
          {/if}

          <!-- Skills preloaded (agents) -->
          {#if selectedParsed.meta.skills}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <BookOpen size={12} />
                <span>Preloaded Skills</span>
              </div>
              <div class="flex flex-wrap gap-1">
                {#each (Array.isArray(selectedParsed.meta.skills) ? selectedParsed.meta.skills : [String(selectedParsed.meta.skills)]) as skill}
                  <span class="text-[10px] px-1.5 py-0.5 rounded bg-warning/10 text-warning">{skill}</span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- MCP (agents) -->
          {#if selectedParsed.meta.mcp}
            <div class="bg-bg-secondary border border-border rounded-lg p-3">
              <div class="flex items-center gap-2 text-xs text-text-muted mb-1">
                <Server size={12} />
                <span>Inline MCP</span>
              </div>
              <p class="text-sm text-accent">Configured</p>
            </div>
          {/if}
        </div>

        <!-- Connections visualization -->
        {#if selectedParsed.meta.skills || selectedParsed.meta.mcp || selectedParsed.meta.memory || selectedParsed.meta.hooks}
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-3">Connections</h3>
            <div class="flex items-center gap-2 flex-wrap">
              <div class="px-3 py-1.5 rounded-lg border-2 {isAgent ? 'border-accent bg-accent/5' : 'border-warning bg-warning/5'}">
                <span class="text-sm font-medium {isAgent ? 'text-accent' : 'text-warning'}">{selected.name}</span>
              </div>
              {#if selectedParsed.meta.skills}
                <span class="text-text-muted">→</span>
                <div class="flex items-center gap-1">
                  <BookOpen size={12} class="text-warning" />
                  {#each (Array.isArray(selectedParsed.meta.skills) ? selectedParsed.meta.skills : [String(selectedParsed.meta.skills)]) as skill}
                    <span class="px-2 py-1 rounded bg-warning/10 text-warning text-xs">{skill}</span>
                  {/each}
                </div>
              {/if}
              {#if selectedParsed.meta.mcp}
                <span class="text-text-muted">→</span>
                <div class="flex items-center gap-1">
                  <Server size={12} class="text-accent" />
                  <span class="px-2 py-1 rounded bg-accent/10 text-accent text-xs">MCP</span>
                </div>
              {/if}
              {#if selectedParsed.meta.memory}
                <span class="text-text-muted">→</span>
                <div class="flex items-center gap-1">
                  <Database size={12} class="text-success" />
                  <span class="px-2 py-1 rounded bg-success/10 text-success text-xs">{selectedParsed.meta.memory} memory</span>
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Content preview -->
        <div class="bg-bg-secondary border border-border rounded-lg p-4">
          <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-3">Instructions</h3>
          <div class="md-preview text-sm">
            {@html marked(selectedParsed.body || "_No content_") as string}
          </div>
        </div>
      </div>
    {:else}
      <!-- Empty state -->
      <div class="flex-1 flex flex-col items-center justify-center text-text-muted">
        {#if isAgent}
          <Bot size={32} class="opacity-20 mb-3" />
        {:else}
          <Sparkles size={32} class="opacity-20 mb-3" />
        {/if}
        <p class="text-sm">Select a {isAgent ? "agent" : "skill"} or create a new one</p>
        <p class="text-xs mt-1">
          {isAgent
            ? "Agents are specialized AI assistants with isolated contexts"
            : "Skills extend Claude's capabilities with custom commands"}
        </p>
      </div>
    {/if}
  </div>
</div>

<TemplateGallery
  open={galleryOpen}
  defaultCategory={activeTab === "skills" ? "skill" : "agent"}
  onselect={async (template) => {
    const pp = needsProject ? projectPath ?? undefined : undefined;
    const name = template.name.toLowerCase().replace(/\s+/g, "-");
    const content = template.content ?? "";
    if (template.category === "agent") {
      await api.agents.write(scope, name, content, pp);
    } else {
      await api.skills.write(scope, name, content, pp);
    }
    await loadItems();
    const found = items.find((i) => i.name === name);
    if (found) selectItem(found);
  }}
  onclose={() => (galleryOpen = false)}
/>
