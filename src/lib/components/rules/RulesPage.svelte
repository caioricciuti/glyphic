<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { RuleFile } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { marked } from "marked";
  import { Shield, Plus, Search, X, Trash2, FileText, Filter, LayoutGrid } from "lucide-svelte";
  import TemplateGallery from "$lib/components/shared/TemplateGallery.svelte";

  let scope = $state<"global" | "project">("global");
  let rules = $state<RuleFile[]>([]);
  let selected = $state<RuleFile | null>(null);
  let loading = $state(true);
  let searchQuery = $state("");

  // Editor
  let editing = $state(false);
  let isNew = $state(false);
  let editContent = $state("");
  let editPaths = $state("");
  let editFilename = $state("");
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let previewMode = $state(false);
  let galleryOpen = $state(false);

  // Delete
  let deleteDialogOpen = $state(false);

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(scope === "project");

  const filteredRules = $derived(
    rules.filter((r) => !searchQuery || r.name.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  async function loadRules() {
    if (needsProject && !projectPath) { loading = false; rules = []; return; }
    loading = true;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      rules = await api.rules.list(scope, pp);
    } catch (e) { console.error("Failed:", e); rules = []; }
    finally { loading = false; }
  }

  function selectRule(rule: RuleFile) {
    selected = rule;
    editing = false;
    isNew = false;
  }

  function startEdit() {
    if (!selected) return;
    editContent = selected.content;
    editPaths = selected.paths_filter.join("\n");
    editFilename = selected.name + ".md";
    editing = true;
    isNew = false;
    previewMode = false;
  }

  function startCreate() {
    editContent = "";
    editPaths = "";
    editFilename = "";
    editing = true;
    isNew = true;
    selected = null;
    previewMode = false;
  }

  async function save() {
    const filename = isNew ? (editFilename.endsWith(".md") ? editFilename : editFilename + ".md") : selected?.name + ".md";
    if (!filename?.trim()) return;
    saving = true;
    saveMessage = null;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      const pathsFilter = editPaths.split("\n").map((p) => p.trim()).filter(Boolean);
      await api.rules.write(scope, filename, pathsFilter, editContent, pp);
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
      await loadRules();
      if (isNew) {
        const name = filename.replace(/\.md$/, "");
        selected = rules.find((r) => r.name === name) ?? null;
        editing = false;
        isNew = false;
      }
    } catch (e) { saveMessage = `Error: ${e}`; }
    finally { saving = false; }
  }

  async function deleteRule() {
    if (!selected) return;
    try {
      const pp = needsProject ? projectPath ?? undefined : undefined;
      await api.rules.delete(scope, selected.name + ".md", pp);
      selected = null;
      editing = false;
      await loadRules();
    } catch (e) { console.error("Failed:", e); }
    finally { deleteDialogOpen = false; }
  }

  onMount(loadRules);
</script>

<ConfirmDialog
  open={deleteDialogOpen}
  title="Delete Rule"
  message="This rule will be permanently deleted."
  onconfirm={deleteRule}
  oncancel={() => (deleteDialogOpen = false)}
/>

<div class="flex h-full">
  <!-- Sidebar -->
  <div class="w-64 shrink-0 border-r border-border flex flex-col bg-bg-secondary">
    <div class="p-3 border-b border-border space-y-2">
      <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
        <button class="flex-1 px-3 py-1.5 text-xs rounded-md transition-colors {scope === 'global' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { scope = "global"; selected = null; editing = false; loadRules(); }}>Global</button>
        <button class="flex-1 px-3 py-1.5 text-xs rounded-md transition-colors {scope === 'project' ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => { scope = "project"; selected = null; editing = false; loadRules(); }}>Project</button>
      </div>
      {#if needsProject}
        <ProjectPicker onselect={loadRules} />
      {/if}
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2 text-text-muted" />
        <input type="text" class="w-full pl-8 pr-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search rules..." bind:value={searchQuery} />
      </div>
    </div>

    <div class="flex-1 overflow-y-auto py-1">
      {#if loading}
        <p class="text-xs text-text-muted px-3 py-4 text-center">Loading...</p>
      {:else if needsProject && !projectPath}
        <p class="text-xs text-text-muted px-3 py-4 text-center">Select a project</p>
      {:else}
        {#each filteredRules as rule}
          <button
            class="w-full text-left px-3 py-2.5 transition-colors border-b border-border/50
              {selected?.name === rule.name ? 'bg-accent/10 text-accent' : 'text-text-secondary hover:bg-bg-hover'}"
            onclick={() => selectRule(rule)}
          >
            <div class="flex items-center gap-2">
              <Shield size={14} class="shrink-0 text-info" />
              <span class="text-sm font-medium truncate">{rule.name}</span>
            </div>
            {#if rule.paths_filter.length > 0}
              <div class="flex items-center gap-1 mt-1 ml-[22px]">
                <Filter size={10} class="text-text-muted shrink-0" />
                <span class="text-[10px] text-text-muted truncate">{rule.paths_filter.join(", ")}</span>
              </div>
            {/if}
          </button>
        {/each}

      {/if}
    </div>

    <div class="p-3 border-t border-border flex gap-2">
      <button class="flex-1 flex items-center justify-center gap-1.5 py-2 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={startCreate}>
        <Plus size={14} /> Create Rule
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

  <!-- Main content -->
  <div class="flex-1 flex min-w-0">
    {#if editing || isNew}
      <!-- Editor -->
      <div class="flex-1 flex flex-col">
        <div class="flex items-center justify-between px-6 py-3 border-b border-border shrink-0">
          <div class="flex items-center gap-3">
            {#if isNew}
              <label class="flex items-center gap-2">
                <span class="text-xs text-text-muted">Filename</span>
                <input type="text" class="px-3 py-1 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="my-rule.md" bind:value={editFilename} />
              </label>
            {:else}
              <span class="text-sm font-medium text-text-primary">{selected?.name}.md</span>
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
            <button class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50" onclick={save} disabled={saving || (isNew && !editFilename.trim())}>{saving ? "..." : "Save"}</button>
            <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => { editing = false; isNew = false; }} aria-label="Close"><X size={16} /></button>
          </div>
        </div>

        <!-- Path filters -->
        <div class="px-6 py-3 border-b border-border">
          <label class="block">
            <span class="text-xs text-text-muted flex items-center gap-1"><Filter size={10} /> Path filters (one per line — rule only applies to matching files)</span>
            <textarea class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent resize-none" rows="2" placeholder="src/**/*.ts&#10;**/*.test.*" bind:value={editPaths}></textarea>
          </label>
        </div>

        <div class="flex-1 overflow-hidden p-4">
          {#if previewMode}
            <div class="h-full overflow-y-auto md-preview px-4">{@html marked(editContent || "") as string}</div>
          {:else}
            <textarea class="w-full h-full px-4 py-3 text-sm bg-bg-secondary border border-border rounded-lg text-text-primary font-mono resize-none focus:outline-none focus:border-accent" placeholder="# Rule Title&#10;&#10;- Rule 1&#10;- Rule 2" bind:value={editContent}></textarea>
          {/if}
        </div>
      </div>
    {:else if selected}
      <!-- Detail view -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-info/10 flex items-center justify-center">
              <Shield size={20} class="text-info" />
            </div>
            <div>
              <h2 class="text-lg font-semibold text-text-primary">{selected.name}</h2>
              <p class="text-xs text-text-muted font-mono">{selected.path}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button class="px-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30" onclick={startEdit}>Edit</button>
            <button class="p-1.5 text-text-muted hover:text-danger rounded" onclick={() => (deleteDialogOpen = true)} aria-label="Delete"><Trash2 size={14} /></button>
          </div>
        </div>

        <!-- Path filters -->
        {#if selected.paths_filter.length > 0}
          <div class="bg-bg-secondary border border-border rounded-lg p-4">
            <h3 class="text-xs font-medium text-text-muted uppercase tracking-wider mb-2 flex items-center gap-1.5">
              <Filter size={12} /> Path Filters
            </h3>
            <p class="text-xs text-text-muted mb-2">This rule only applies when working on files matching these patterns:</p>
            <div class="flex flex-wrap gap-2">
              {#each selected.paths_filter as path}
                <span class="px-2.5 py-1 text-xs bg-bg-tertiary border border-border rounded-md font-mono text-info">{path}</span>
              {/each}
            </div>
          </div>
        {:else}
          <div class="bg-bg-secondary border border-border rounded-lg p-3 flex items-center gap-2">
            <FileText size={14} class="text-text-muted" />
            <span class="text-xs text-text-muted">No path filters — this rule applies everywhere</span>
          </div>
        {/if}

        <!-- Content -->
        <div class="bg-bg-secondary border border-border rounded-lg p-6">
          <div class="md-preview">
            {@html marked(selected.content || "_No content_") as string}
          </div>
        </div>
      </div>
    {:else}
      <div class="flex-1 flex flex-col items-center justify-center text-text-muted">
        <Shield size={32} class="opacity-20 mb-3" />
        <p class="text-sm">Select a rule or create a new one</p>
        <p class="text-xs mt-1">Rules guide Claude's behavior for specific file patterns</p>
      </div>
    {/if}
  </div>
</div>

<TemplateGallery
  open={galleryOpen}
  defaultCategory="rule"
  onselect={async (template) => {
    const pp = needsProject ? projectPath ?? undefined : undefined;
    const name = template.name.toLowerCase().replace(/\s+/g, "-");
    const filename = name + ".md";
    const paths = template.paths ?? [];
    await api.rules.write(scope, filename, paths, template.content ?? "", pp);
    await loadRules();
    const found = rules.find((r) => r.name === name);
    if (found) selectRule(found);
  }}
  onclose={() => (galleryOpen = false)}
/>
