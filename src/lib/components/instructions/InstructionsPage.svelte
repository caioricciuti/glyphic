<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { InstructionFile } from "$lib/types";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { marked } from "marked";
  import { X, FileText, ExternalLink } from "lucide-svelte";

  let activeScope = $state<"global" | "project" | "project-dot">("global");
  let file = $state<InstructionFile | null>(null);
  let loading = $state(true);
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let previewMode = $state(false);
  let content = $state("");

  // Reference sheet
  let refOpen = $state(false);
  let refName = $state("");
  let refContent = $state("");
  let refLoading = $state(false);
  let refError = $state<string | null>(null);

  const renderedHtml = $derived(marked(content || "") as string);

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(activeScope !== "global");

  const scopeLabels: Record<string, string> = {
    global: "~/.claude/CLAUDE.md",
    project: "./CLAUDE.md",
    "project-dot": "./.claude/CLAUDE.md",
  };

  async function loadInstructions() {
    if (needsProject && !projectPath) {
      loading = false;
      file = null;
      content = "";
      return;
    }
    loading = true;
    try {
      file = await api.instructions.read(activeScope, needsProject ? projectPath ?? undefined : undefined);
      content = file.content;
    } catch (e) {
      console.error("Failed to load:", e);
      file = null;
      content = "";
    } finally {
      loading = false;
    }
  }

  async function saveInstructions() {
    saving = true;
    saveMessage = null;
    try {
      await api.instructions.write(activeScope, content, needsProject ? projectPath ?? undefined : undefined);
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function openReference(ref: string) {
    if (!file?.path) return;
    refName = ref;
    refOpen = true;
    refLoading = true;
    refError = null;
    refContent = "";
    try {
      refContent = await api.instructions.readReference(file.path, ref);
    } catch (e) {
      refError = `${e}`;
    } finally {
      refLoading = false;
    }
  }

  function handleScopeChange(scope: typeof activeScope) {
    activeScope = scope;
    loadInstructions();
  }

  onMount(loadInstructions);
</script>

<div class="p-6 overflow-y-auto h-full flex flex-col gap-4">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
        {#each [
          { id: "global" as const, label: "Global" },
          { id: "project" as const, label: "Project" },
          { id: "project-dot" as const, label: "Project (.claude/)" },
        ] as tab}
          <button
            class="px-4 py-1.5 text-sm rounded-md transition-colors
              {activeScope === tab.id
                ? 'bg-bg-secondary text-text-primary'
                : 'text-text-muted hover:text-text-secondary'}"
            onclick={() => handleScopeChange(tab.id)}
          >
            {tab.label}
          </button>
        {/each}
      </div>
      {#if needsProject}
        <ProjectPicker onselect={loadInstructions} />
      {/if}
    </div>

    <div class="flex items-center gap-3">
      {#if saveMessage}
        <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">
          {saveMessage}
        </span>
      {/if}
      <button
        class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
        onclick={saveInstructions}
        disabled={saving}
      >
        {saving ? "Saving..." : "Save"}
      </button>
    </div>
  </div>

  <p class="text-xs text-text-muted font-mono">{scopeLabels[activeScope]}</p>

  {#if needsProject && !projectPath}
    <div class="flex items-center justify-center h-48 text-sm text-text-muted">
      Select a project to edit instructions
    </div>
  {:else if loading}
    <p class="text-sm text-text-muted">Loading...</p>
  {:else}
    <!-- Imports -->
    {#if file?.imports && file.imports.length > 0}
      <div class="bg-bg-secondary border border-border rounded-lg p-3">
        <p class="text-xs text-text-muted mb-2">Referenced imports:</p>
        <div class="flex flex-wrap gap-2">
          {#each file.imports as imp}
            <button
              class="flex items-center gap-1 px-2.5 py-1 text-xs bg-accent/10 text-accent rounded-md font-mono hover:bg-accent/20 transition-colors"
              onclick={() => openReference(imp)}
            >
              <FileText size={12} />
              @{imp}
              <ExternalLink size={10} class="opacity-50" />
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Edit/Preview Toggle -->
    <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1 w-fit">
      <button
        class="px-3 py-1 text-xs rounded-md transition-colors {!previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}"
        onclick={() => (previewMode = false)}
      >Edit</button>
      <button
        class="px-3 py-1 text-xs rounded-md transition-colors {previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}"
        onclick={() => (previewMode = true)}
      >Preview</button>
    </div>

    {#if !file?.exists && !previewMode}
      <div class="bg-bg-secondary border border-border rounded-lg p-4 text-center">
        <p class="text-sm text-text-muted mb-2">This file doesn't exist yet</p>
        <p class="text-xs text-text-muted">Start typing below to create it</p>
      </div>
    {/if}

    {#if previewMode}
      <div class="flex-1 w-full px-6 py-4 bg-bg-secondary border border-border rounded-lg overflow-y-auto md-preview">
        {@html renderedHtml}
      </div>
    {:else}
      <textarea
        class="flex-1 w-full px-4 py-3 text-sm bg-bg-secondary border border-border rounded-lg text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono resize-none"
        placeholder="# Instructions for Claude Code..."
        bind:value={content}
      ></textarea>
    {/if}
  {/if}
</div>

<!-- Reference Sheet (slide-in from right) -->
{#if refOpen}
  <button
    class="fixed inset-0 bg-black/40 z-50"
    onclick={() => (refOpen = false)}
    aria-label="Close reference"
  ></button>
  <div class="fixed top-0 right-0 h-full w-[500px] bg-bg-secondary border-l border-border z-50 flex flex-col shadow-2xl">
    <!-- Sheet header -->
    <div class="flex items-center justify-between px-4 py-3 border-b border-border shrink-0">
      <div class="flex items-center gap-2">
        <FileText size={16} class="text-accent" />
        <span class="text-sm font-medium text-text-primary font-mono">@{refName}</span>
      </div>
      <button
        class="p-1 rounded hover:bg-bg-hover text-text-muted hover:text-text-primary transition-colors"
        onclick={() => (refOpen = false)}
        aria-label="Close"
      >
        <X size={18} />
      </button>
    </div>

    <!-- Sheet body -->
    <div class="flex-1 overflow-y-auto px-6 py-4">
      {#if refLoading}
        <p class="text-sm text-text-muted">Loading...</p>
      {:else if refError}
        <div class="bg-danger/10 border border-danger/20 rounded-lg p-3">
          <p class="text-sm text-danger">{refError}</p>
        </div>
      {:else}
        <div class="md-preview">
          {@html marked(refContent) as string}
        </div>
      {/if}
    </div>
  </div>
{/if}
