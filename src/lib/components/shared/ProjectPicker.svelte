<script lang="ts">
  import {
    getProjects,
    getSelectedProjectPath,
    getSelectedProjectName,
    selectProject,
    loadProjects,
    isLoaded,
  } from "$lib/stores/project-context.svelte";
  import { onMount } from "svelte";
  import { FolderOpen } from "lucide-svelte";

  interface Props {
    onselect?: () => void;
  }

  const { onselect }: Props = $props();

  let open = $state(false);
  let search = $state("");
  let showCustomPath = $state(false);
  let customPath = $state("");

  const selectedPath = $derived(getSelectedProjectPath());
  const selectedName = $derived(getSelectedProjectName());
  const projects = $derived(getProjects());
  const loaded = $derived(isLoaded());

  const filtered = $derived(
    projects.filter((p) => {
      if (!search) return true;
      const q = search.toLowerCase();
      return p.path.toLowerCase().includes(q);
    }),
  );

  function pick(path: string) {
    selectProject(path);
    open = false;
    search = "";
    showCustomPath = false;
    onselect?.();
  }

  function submitCustomPath() {
    const p = customPath.trim();
    if (p) {
      pick(p);
      customPath = "";
    }
  }

  onMount(() => {
    if (!loaded) loadProjects();
  });
</script>

<div class="relative">
  <button
    class="flex items-center gap-2 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md hover:border-border-light transition-colors max-w-xs"
    onclick={() => (open = !open)}
  >
    {#if selectedPath}
      <span class="w-2 h-2 rounded-full bg-success shrink-0"></span>
      <span class="truncate text-text-primary">{selectedName}</span>
    {:else}
      <span class="w-2 h-2 rounded-full bg-text-muted shrink-0"></span>
      <span class="text-text-muted">Select project...</span>
    {/if}
    <svg class="w-3 h-3 text-text-muted shrink-0" viewBox="0 0 12 12" fill="currentColor">
      <path d="M2 4l4 4 4-4" stroke="currentColor" stroke-width="1.5" fill="none" />
    </svg>
  </button>

  {#if open}
    <button
      class="fixed inset-0 z-40"
      onclick={() => { open = false; showCustomPath = false; }}
      aria-label="Close project picker"
    ></button>

    <div class="absolute top-full left-0 mt-1 w-[420px] bg-bg-secondary border border-border rounded-lg shadow-xl z-50 overflow-hidden">
      <div class="p-2 border-b border-border">
        <input
          type="text"
          class="w-full px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
          placeholder="Search projects..."
          bind:value={search}
        />
      </div>

      <div class="max-h-72 overflow-y-auto">
        {#each filtered as project}
          <button
            class="w-full text-left px-3 py-2 hover:bg-bg-hover transition-colors flex items-center gap-2
              {selectedPath === project.path ? 'bg-accent/10' : ''}
              {!project.exists ? 'opacity-50' : ''}"
            onclick={() => pick(project.path)}
          >
            <div class="min-w-0 flex-1">
              <p class="text-sm text-text-primary truncate">{project.path.split("/").pop()}</p>
              <p class="text-xs text-text-muted truncate font-mono">{project.path}</p>
            </div>
            <div class="flex items-center gap-1.5 shrink-0">
              {#if !project.exists}
                <span class="text-[10px] text-text-muted">moved</span>
              {/if}
              {#if project.has_memory}
                <span class="text-[10px] text-accent">mem</span>
              {/if}
            </div>
          </button>
        {/each}
        {#if filtered.length === 0}
          <p class="px-3 py-4 text-sm text-text-muted text-center">
            {search ? "No matching projects" : "No projects discovered"}
          </p>
        {/if}
      </div>

      <!-- Custom path -->
      <div class="border-t border-border p-2">
        {#if showCustomPath}
          <div class="flex gap-2">
            <input
              type="text"
              class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent font-mono"
              placeholder="/path/to/project"
              bind:value={customPath}
              onkeydown={(e) => e.key === "Enter" && submitCustomPath()}
            />
            <button
              class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md"
              onclick={submitCustomPath}
            >
              Open
            </button>
          </div>
        {:else}
          <button
            class="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-text-muted hover:text-text-secondary transition-colors"
            onclick={() => (showCustomPath = true)}
          >
            <FolderOpen size={14} />
            <span>Open folder...</span>
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
