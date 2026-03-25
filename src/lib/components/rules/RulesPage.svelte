<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { RuleFile } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { marked } from "marked";
  import { Shield, Plus, Search, X, Trash2, FileText, Filter } from "lucide-svelte";

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

  // Templates
  const TEMPLATES = [
    { name: "typescript-strict", label: "TypeScript Strict", paths: ["**/*.ts", "**/*.tsx"], content: "# TypeScript Rules\n\n- Use strict mode, no `any` types\n- Named exports over default exports\n- Prefer interfaces over type aliases for objects\n- Use explicit return types on public functions\n- No unused variables or imports" },
    { name: "api-design", label: "API Design", paths: ["src/api/**/*", "src/routes/**/*"], content: "# API Design Rules\n\n- All endpoints must validate input\n- Use standard error format: `{ error: string, code: number }`\n- Return proper HTTP status codes\n- Include pagination for list endpoints\n- Document endpoints with JSDoc" },
    { name: "testing", label: "Testing Standards", paths: ["**/*.test.*", "**/*.spec.*"], content: "# Testing Rules\n\n- Co-locate test files with source files\n- Use descriptive names: 'should [behavior] when [condition]'\n- No test should depend on another test's state\n- Mock external services, not internal modules\n- Aim for 80%+ coverage on critical paths" },
    { name: "security", label: "Security", paths: [], content: "# Security Rules\n\n- Never commit secrets, API keys, or credentials\n- Sanitize user input before database queries\n- Use parameterized queries, never string concatenation\n- Validate all external data at system boundaries\n- Use HTTPS for all external API calls" },
    { name: "git-commits", label: "Git Conventions", paths: [], content: "# Git Commit Rules\n\n- Use conventional commits: `type(scope): description`\n- Types: feat, fix, refactor, test, docs, chore\n- Keep commits atomic and focused\n- Write imperative mood: 'add feature' not 'added feature'\n- Reference issue numbers when applicable" },
    { name: "error-handling", label: "Error Handling", paths: ["src/**/*"], content: "# Error Handling Rules\n\n- Never swallow errors silently\n- Use custom error classes for domain errors\n- Always include error context (what was being done)\n- Log errors at the point of handling, not catching\n- Return user-friendly messages, log technical details" },
    { name: "performance", label: "Performance", paths: [], content: "# Performance Rules\n\n- Avoid N+1 queries — use batch loading\n- Use pagination for large data sets\n- Lazy-load non-critical resources\n- Profile before optimizing\n- Cache expensive computations" },
    { name: "accessibility", label: "Accessibility", paths: ["**/*.svelte", "**/*.tsx", "**/*.jsx"], content: "# Accessibility Rules\n\n- All images must have alt text\n- Use semantic HTML elements\n- Ensure keyboard navigation works\n- Maintain sufficient color contrast\n- Add aria-labels to icon-only buttons" },
  ];

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

        {#if filteredRules.length === 0 && !isNew}
          <div class="px-3 py-3 space-y-2">
            <p class="text-xs text-text-muted">No rules. Try a template:</p>
            {#each TEMPLATES as tpl}
              <button
                class="w-full text-left p-2 bg-bg-tertiary border border-border rounded-md hover:border-accent/30 transition-colors"
                onclick={async () => {
                  const pp = needsProject ? projectPath ?? undefined : undefined;
                  await api.rules.write(scope, tpl.name + ".md", tpl.paths, tpl.content, pp);
                  await loadRules();
                  const found = rules.find((r) => r.name === tpl.name);
                  if (found) selectRule(found);
                }}
              >
                <p class="text-xs font-medium text-accent">{tpl.label}</p>
                <p class="text-[10px] text-text-muted truncate">{tpl.content.split("\n")[2]}</p>
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </div>

    <div class="p-3 border-t border-border">
      <button class="w-full flex items-center justify-center gap-1.5 py-2 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={startCreate}>
        <Plus size={14} /> Create Rule
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
