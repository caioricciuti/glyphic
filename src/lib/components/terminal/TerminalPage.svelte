<script lang="ts">
  import "@xterm/xterm/css/xterm.css";
  import { onMount, onDestroy } from "svelte";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import { getSelectedProjectPath, getSelectedProjectName } from "$lib/stores/project-context.svelte";
  import {
    getSessions,
    getActiveSessionId,
    getActiveSession,
    createSession,
    closeSession,
    switchSession,
    attachToContainer,
    detachFromContainer,
    fitActiveSession,
  } from "$lib/stores/terminal.svelte";
  import { X } from "lucide-svelte";

  let terminalContainer: HTMLDivElement;
  let starting = $state(false);
  let error = $state<string | null>(null);

  const projectPath = $derived(getSelectedProjectPath());
  const projectName = $derived(getSelectedProjectName());
  const sessions = $derived(getSessions());
  const activeId = $derived(getActiveSessionId());
  const activeSession = $derived(getActiveSession());

  async function startNew() {
    if (!projectPath || starting) return;
    starting = true;
    error = null;
    try {
      await createSession(projectPath, projectName);
      // Attach to container
      requestAnimationFrame(() => {
        if (terminalContainer) attachToContainer(terminalContainer);
      });
    } catch (e) {
      error = `Failed to start: ${e}`;
    } finally {
      starting = false;
    }
  }

  function handleTabSwitch(id: string) {
    detachFromContainer();
    switchSession(id);
    requestAnimationFrame(() => {
      if (terminalContainer) attachToContainer(terminalContainer);
    });
  }

  async function handleClose(id: string) {
    await closeSession(id);
    requestAnimationFrame(() => {
      if (terminalContainer && getActiveSession()) {
        attachToContainer(terminalContainer);
      }
    });
  }

  function handleResize() {
    fitActiveSession();
  }

  onMount(() => {
    window.addEventListener("resize", handleResize);
    // Re-attach existing session if coming back to terminal tab
    if (activeSession && terminalContainer) {
      requestAnimationFrame(() => attachToContainer(terminalContainer));
    }
  });

  onDestroy(() => {
    window.removeEventListener("resize", handleResize);
    // Detach but DON'T kill — session stays alive
    detachFromContainer();
  });
</script>

<div class="flex flex-col h-full">
  <!-- Tab bar -->
  <div class="flex items-center gap-1 px-3 py-2 border-b border-border bg-bg-secondary shrink-0 overflow-x-auto">
    {#each sessions as session}
      <div
        class="flex items-center gap-1.5 pl-3 pr-1 py-1.5 text-xs rounded-md transition-colors shrink-0 cursor-pointer
          {activeId === session.id
            ? 'bg-bg-tertiary text-text-primary'
            : 'text-text-muted hover:text-text-secondary hover:bg-bg-hover'}"
        role="tab"
        tabindex="0"
        onclick={() => handleTabSwitch(session.id)}
        onkeydown={(e) => e.key === "Enter" && handleTabSwitch(session.id)}
      >
        <span class="w-1.5 h-1.5 rounded-full {session.alive ? 'bg-success' : 'bg-text-muted'}"></span>
        <span class="max-w-24 truncate">{session.projectName}</span>
        <button
          class="ml-1 p-0.5 rounded hover:bg-bg-hover text-text-muted hover:text-danger"
          onclick={(e) => { e.stopPropagation(); handleClose(session.id); }}
          aria-label="Close session"
        >
          <X size={12} />
        </button>
      </div>
    {/each}

    <!-- New session -->
    <div class="flex items-center gap-2 ml-1">
      <ProjectPicker onselect={() => {}} />
      <button
        class="px-3 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50 shrink-0"
        onclick={startNew}
        disabled={!projectPath || starting}
      >
        {starting ? "..." : "+ New"}
      </button>
    </div>

    {#if error}
      <span class="text-xs text-danger ml-2 shrink-0">{error}</span>
    {/if}
  </div>

  <!-- Terminal area -->
  <div class="flex-1 relative bg-[#0d1117]">
    {#if sessions.length === 0}
      <div class="absolute inset-0 flex flex-col items-center justify-center text-text-muted">
        <svg class="w-16 h-16 mb-4 opacity-20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M4 17l6-6-6-6M12 19h8" />
        </svg>
        <p class="text-sm mb-1">{projectPath ? "Click + New to start a session" : "Select a project first"}</p>
        <p class="text-xs text-text-muted">Sessions persist when you navigate away</p>
      </div>
    {/if}
    <div
      bind:this={terminalContainer}
      class="absolute inset-0"
      class:hidden={sessions.length === 0}
    ></div>
  </div>
</div>
