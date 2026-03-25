<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import type { ProjectInfo, MemoryFile } from "$lib/types";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import { marked } from "marked";
  import { X, Plus, Brain, User, MessageSquare, FolderOpen, BookOpen, Search, Trash2 } from "lucide-svelte";

  let projects = $state<ProjectInfo[]>([]);
  let selectedProject = $state<ProjectInfo | null>(null);
  let memoryFiles = $state<MemoryFile[]>([]);
  let loading = $state(true);
  let searchQuery = $state("");

  // Editor sheet
  let editingFile = $state<MemoryFile | null>(null);
  let editContent = $state("");
  let editName = $state("");
  let editDescription = $state("");
  let editType = $state("");
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let previewMode = $state(false);

  // Create new
  let showCreate = $state(false);
  let newFilename = $state("");

  // Delete
  let deleteDialogOpen = $state(false);

  const TYPE_COLORS: Record<string, string> = {
    user: "bg-info/10 text-info",
    feedback: "bg-warning/10 text-warning",
    project: "bg-success/10 text-success",
    reference: "bg-accent/10 text-accent",
  };

  const TYPE_ICONS = { user: User, feedback: MessageSquare, project: FolderOpen, reference: BookOpen };

  // Filter to only projects with memory
  const filteredProjects = $derived(
    projects
      .filter((p) => p.has_memory)
      .filter((p) => !searchQuery || p.path.toLowerCase().includes(searchQuery.toLowerCase())),
  );

  async function loadProjects() {
    loading = true;
    try {
      projects = await api.projects.list();
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      loading = false;
    }
  }

  async function selectProject(project: ProjectInfo) {
    selectedProject = project;
    editingFile = null;
    try {
      memoryFiles = await api.memory.listFiles(project.hash);
    } catch (e) {
      console.error("Failed:", e);
      memoryFiles = [];
    }
  }

  function openEditor(file: MemoryFile) {
    editingFile = file;
    editContent = file.content;
    editName = file.name ?? "";
    editDescription = file.description ?? "";
    editType = file.memory_type ?? "";
    previewMode = false;
    showCreate = false;
  }

  function startCreate() {
    showCreate = true;
    editingFile = null;
    editContent = "";
    editName = "";
    editDescription = "";
    editType = "project";
    newFilename = "";
    previewMode = false;
  }

  async function saveFile() {
    if (!selectedProject) return;
    const filename = showCreate ? (newFilename.endsWith(".md") ? newFilename : newFilename + ".md") : editingFile?.filename;
    if (!filename) return;
    saving = true;
    saveMessage = null;
    try {
      await api.memory.writeFile(
        selectedProject.hash, filename,
        editName || null, editDescription || null, editType || null, editContent,
      );
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
      memoryFiles = await api.memory.listFiles(selectedProject.hash);
      if (showCreate) {
        showCreate = false;
        const found = memoryFiles.find((f) => f.filename === filename);
        if (found) openEditor(found);
      }
    } catch (e) {
      saveMessage = `Error: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function deleteFile() {
    if (!selectedProject || !editingFile) return;
    try {
      await api.memory.deleteFile(selectedProject.hash, editingFile.filename);
      memoryFiles = memoryFiles.filter((f) => f.filename !== editingFile!.filename);
      editingFile = null;
    } catch (e) {
      console.error("Failed:", e);
    } finally {
      deleteDialogOpen = false;
    }
  }

  onMount(loadProjects);
</script>

<ConfirmDialog
  open={deleteDialogOpen}
  title="Delete Memory File"
  message="This memory file will be permanently deleted."
  onconfirm={deleteFile}
  oncancel={() => (deleteDialogOpen = false)}
/>

<div class="flex h-full">
  <!-- Project Sidebar -->
  <div class="w-56 shrink-0 border-r border-border flex flex-col bg-bg-secondary">
    <div class="p-3 border-b border-border">
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2 text-text-muted" />
        <input
          type="text"
          class="w-full pl-8 pr-3 py-1.5 text-xs bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
          placeholder="Search projects..."
          bind:value={searchQuery}
        />
      </div>
    </div>

    <div class="flex-1 overflow-y-auto py-1">
      {#if loading}
        <p class="text-xs text-text-muted px-3 py-4 text-center">Loading...</p>
      {:else if filteredProjects.length === 0}
        <p class="text-xs text-text-muted px-3 py-4 text-center">No projects with memory</p>
      {:else}
        {#each filteredProjects as project}
          <button
            class="w-full text-left px-3 py-2 transition-colors
              {selectedProject?.hash === project.hash
                ? 'bg-accent/10 text-accent border-r-2 border-accent'
                : 'text-text-secondary hover:bg-bg-hover'}"
            onclick={() => selectProject(project)}
          >
            <div class="flex items-center justify-between">
              <p class="text-sm font-medium truncate">{project.path.split("/").pop()}</p>
              <Brain size={12} class="text-accent shrink-0" />
            </div>
            <p class="text-[10px] text-text-muted truncate font-mono">{project.path}</p>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <!-- Main Content -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if !selectedProject}
      <div class="flex flex-col items-center justify-center h-full text-text-muted">
        <Brain size={32} class="opacity-20 mb-3" />
        <p class="text-sm">Select a project to browse its memory</p>
        <p class="text-xs mt-1">Only projects with memory files are shown</p>
      </div>
    {:else}
      <!-- Header -->
      <div class="flex items-center justify-between mb-4">
        <div>
          <h3 class="text-base font-medium text-text-primary">{selectedProject.path.split("/").pop()}</h3>
          <p class="text-xs text-text-muted font-mono">{selectedProject.path}</p>
        </div>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors"
          onclick={startCreate}
        >
          <Plus size={14} />
          New Memory
        </button>
      </div>

      <!-- Memory file cards -->
      {#if memoryFiles.length === 0 && !showCreate}
        <div class="text-center py-12 text-sm text-text-muted">
          No memory files in this project
        </div>
      {:else}
        <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
          {#each memoryFiles as file}
            {@const typeColor = TYPE_COLORS[file.memory_type ?? ""] ?? "bg-bg-tertiary text-text-muted"}
            {@const TypeIcon = TYPE_ICONS[file.memory_type as keyof typeof TYPE_ICONS] ?? Brain}
            <button
              class="text-left p-4 bg-bg-secondary border rounded-lg transition-colors hover:border-accent/30
                {editingFile?.filename === file.filename ? 'border-accent/50 bg-accent/5' : 'border-border'}"
              onclick={() => openEditor(file)}
            >
              <div class="flex items-start justify-between mb-2">
                <div class="flex items-center gap-2">
                  <div class="w-6 h-6 rounded flex items-center justify-center {typeColor}">
                    <TypeIcon size={12} />
                  </div>
                  {#if file.memory_type}
                    <span class="text-[10px] px-1.5 py-0.5 rounded-full {typeColor}">{file.memory_type}</span>
                  {/if}
                </div>
              </div>
              <p class="text-sm font-medium text-text-primary truncate">{file.name || file.filename}</p>
              {#if file.description}
                <p class="text-xs text-text-muted mt-0.5 line-clamp-2">{file.description}</p>
              {/if}
              <p class="text-xs text-text-muted mt-2 line-clamp-2 font-mono opacity-50">{file.content.slice(0, 100)}</p>
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>

  <!-- Editor Sheet -->
  {#if editingFile || showCreate}
    <div class="w-[450px] shrink-0 border-l border-border flex flex-col bg-bg-secondary">
      <!-- Sheet header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-border shrink-0">
        <span class="text-sm font-medium text-text-primary truncate">
          {showCreate ? "New Memory" : editingFile?.filename}
        </span>
        <div class="flex items-center gap-2">
          {#if saveMessage}
            <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
          {/if}
          {#if editingFile && !showCreate}
            <button class="p-1 text-text-muted hover:text-danger" onclick={() => (deleteDialogOpen = true)} aria-label="Delete">
              <Trash2 size={14} />
            </button>
          {/if}
          <button
            class="px-3 py-1 text-xs bg-accent hover:bg-accent-hover text-white rounded-md disabled:opacity-50"
            onclick={saveFile} disabled={saving || (showCreate && !newFilename.trim())}
          >{saving ? "..." : "Save"}</button>
          <button class="p-1 text-text-muted hover:text-text-primary" onclick={() => { editingFile = null; showCreate = false; }} aria-label="Close">
            <X size={16} />
          </button>
        </div>
      </div>

      <!-- Metadata -->
      <div class="px-4 py-3 border-b border-border space-y-2">
        {#if showCreate}
          <label class="block">
            <span class="text-xs text-text-muted">Filename</span>
            <input type="text" class="w-full mt-1 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="my-memory.md" bind:value={newFilename} />
          </label>
        {/if}
        <div class="grid grid-cols-2 gap-2">
          <label class="block">
            <span class="text-xs text-text-muted">Name</span>
            <input type="text" class="w-full mt-1 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" bind:value={editName} />
          </label>
          <label class="block">
            <span class="text-xs text-text-muted">Type</span>
            <select class="w-full mt-1 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" bind:value={editType}>
              <option value="">none</option>
              <option value="user">user</option>
              <option value="feedback">feedback</option>
              <option value="project">project</option>
              <option value="reference">reference</option>
            </select>
          </label>
        </div>
        <label class="block">
          <span class="text-xs text-text-muted">Description</span>
          <input type="text" class="w-full mt-1 px-2 py-1 text-sm bg-bg-tertiary border border-border rounded text-text-primary focus:outline-none focus:border-accent" placeholder="One-line description..." bind:value={editDescription} />
        </label>
      </div>

      <!-- Edit/Preview toggle -->
      <div class="px-4 pt-3 pb-1">
        <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1 w-fit">
          <button class="px-3 py-1 text-xs rounded-md {!previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => (previewMode = false)}>Edit</button>
          <button class="px-3 py-1 text-xs rounded-md {previewMode ? 'bg-bg-secondary text-text-primary' : 'text-text-muted'}" onclick={() => (previewMode = true)}>Preview</button>
        </div>
      </div>

      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if previewMode}
          <div class="md-preview">
            {@html marked(editContent || "") as string}
          </div>
        {:else}
          <textarea
            class="w-full h-full px-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary font-mono resize-none focus:outline-none focus:border-accent"
            placeholder="Memory content..."
            bind:value={editContent}
          ></textarea>
        {/if}
      </div>
    </div>
  {/if}
</div>
