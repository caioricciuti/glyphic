<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/tauri/commands";
  import { HOOK_EVENTS, HOOK_EVENT_DESCRIPTIONS } from "$lib/types";
  import type { HookEvent, HookEventConfig, HookHandler, SettingsScope } from "$lib/types";
  import HookCard from "./HookCard.svelte";
  import ConfirmDialog from "$lib/components/shared/ConfirmDialog.svelte";
  import ProjectPicker from "$lib/components/shared/ProjectPicker.svelte";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import { Plus, Zap, LayoutGrid } from "lucide-svelte";
  import TemplateGallery from "$lib/components/shared/TemplateGallery.svelte";

  interface FlatHook {
    event: string;
    matcher: string | undefined;
    handler: HookHandler;
    configIndex: number;
    handlerIndex: number;
  }

  let rawHooks = $state<Record<string, HookEventConfig[]>>({});
  let loading = $state(true);
  let scope = $state<SettingsScope>("global");
  let saving = $state(false);
  let saveMessage = $state<string | null>(null);
  let selectedEvent = $state<HookEvent | null>(null);
  let showAddForm = $state(false);
  let galleryOpen = $state(false);

  // Delete dialog
  let deleteTarget = $state<FlatHook | null>(null);

  // Add form state
  let newMatcher = $state("");
  let newType = $state<HookHandler["type"]>("command");
  let newValue = $state("");

  const projectPath = $derived(getSelectedProjectPath());
  const needsProject = $derived(scope !== "global");

  // Hooks for selected event
  const eventHooks = $derived(() => {
    if (!selectedEvent) return [];
    const configs = rawHooks[selectedEvent] ?? [];
    const result: FlatHook[] = [];
    for (let ci = 0; ci < configs.length; ci++) {
      for (let hi = 0; hi < configs[ci].hooks.length; hi++) {
        result.push({
          event: selectedEvent,
          matcher: configs[ci].matcher,
          handler: configs[ci].hooks[hi],
          configIndex: ci,
          handlerIndex: hi,
        });
      }
    }
    return result;
  });

  // Count hooks per event
  function hookCount(event: string): number {
    return (rawHooks[event] ?? []).reduce((sum, c) => sum + c.hooks.length, 0);
  }

  async function loadHooks() {
    if (needsProject && !projectPath) { loading = false; rawHooks = {}; return; }
    loading = true;
    try {
      rawHooks = (await api.hooks.get(scope, needsProject ? projectPath ?? undefined : undefined)) as Record<string, HookEventConfig[]>;
    } catch (e) { console.error("Failed:", e); rawHooks = {}; }
    finally { loading = false; }
  }

  async function saveHooks() {
    saving = true; saveMessage = null;
    try {
      await api.hooks.set(scope, rawHooks, needsProject ? projectPath ?? undefined : undefined);
      saveMessage = "Saved!";
      setTimeout(() => (saveMessage = null), 2000);
    } catch (e) { saveMessage = `Error: ${e}`; }
    finally { saving = false; }
  }

  function addHook(event: string, matcher: string | undefined, handler: HookHandler) {
    const updated = { ...rawHooks };
    if (!updated[event]) updated[event] = [];
    updated[event] = [...updated[event], { matcher: matcher || undefined, hooks: [handler] }];
    rawHooks = updated;
    showAddForm = false;
    newValue = "";
    newMatcher = "";
  }

  function addFromForm() {
    if (!selectedEvent) return;
    const handler: HookHandler = { type: newType };
    if (newType === "command") handler.command = newValue || "echo 'hook'";
    else if (newType === "http") handler.url = newValue || "http://localhost:8080/hook";
    else handler.prompt = newValue || "Validate this action";
    addHook(selectedEvent, newMatcher, handler);
  }

  function updateHandler(flat: FlatHook, handler: HookHandler) {
    const updated = { ...rawHooks };
    const configs = [...(updated[flat.event] ?? [])];
    const config = { ...configs[flat.configIndex] };
    config.hooks = [...config.hooks];
    config.hooks[flat.handlerIndex] = handler;
    configs[flat.configIndex] = config;
    updated[flat.event] = configs;
    rawHooks = updated;
  }

  function confirmDeleteHook() {
    if (!deleteTarget) return;
    const flat = deleteTarget;
    const updated = { ...rawHooks };
    const configs = [...(updated[flat.event] ?? [])];
    const config = { ...configs[flat.configIndex] };
    config.hooks = config.hooks.filter((_, i) => i !== flat.handlerIndex);
    if (config.hooks.length === 0) {
      configs.splice(flat.configIndex, 1);
    } else {
      configs[flat.configIndex] = config;
    }
    updated[flat.event] = configs.length > 0 ? configs : [];
    if (configs.length === 0) delete updated[flat.event];
    rawHooks = updated;
    deleteTarget = null;
  }

  function selectEvent(ev: HookEvent) {
    selectedEvent = ev;
    showAddForm = false;
  }

  onMount(loadHooks);
</script>

<ConfirmDialog
  open={deleteTarget !== null}
  title="Delete Hook"
  message="This hook will be removed. Save to apply changes."
  onconfirm={confirmDeleteHook}
  oncancel={() => (deleteTarget = null)}
/>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between px-6 py-3 border-b border-border shrink-0">
    <div class="flex items-center gap-3">
      <div class="flex gap-1 bg-bg-tertiary rounded-lg p-1">
        {#each [{ id: "global" as const, label: "Global" }, { id: "project" as const, label: "Project" }] as tab}
          <button
            class="px-4 py-1.5 text-sm rounded-md transition-colors {scope === tab.id ? 'bg-bg-secondary text-text-primary' : 'text-text-muted hover:text-text-secondary'}"
            onclick={() => { scope = tab.id; loadHooks(); }}
          >{tab.label}</button>
        {/each}
      </div>
      {#if needsProject}
        <ProjectPicker onselect={loadHooks} />
      {/if}
    </div>
    <div class="flex items-center gap-3">
      {#if saveMessage}
        <span class="text-xs {saveMessage.startsWith('Error') ? 'text-danger' : 'text-success'}">{saveMessage}</span>
      {/if}
      <button
        class="flex items-center gap-1.5 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-secondary hover:border-accent/30 hover:text-accent transition-colors"
        onclick={() => (galleryOpen = true)}
      >
        <LayoutGrid size={14} />
        Templates
      </button>
      <button
        class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
        onclick={saveHooks} disabled={saving}
      >{saving ? "Saving..." : "Save"}</button>
    </div>
  </div>

  {#if needsProject && !projectPath}
    <div class="flex items-center justify-center flex-1 text-sm text-text-muted">Select a project</div>
  {:else if loading}
    <div class="flex items-center justify-center flex-1 text-sm text-text-muted">Loading...</div>
  {:else}
    <div class="flex flex-1 min-h-0">
      <!-- Event Sidebar -->
      <div class="w-64 shrink-0 border-r border-border overflow-y-auto py-2">
        {#each HOOK_EVENTS as event}
          {@const count = hookCount(event)}
          <button
            class="w-full flex items-center justify-between px-4 py-2 text-left transition-colors
              {selectedEvent === event
                ? 'bg-accent/10 text-accent border-r-2 border-accent'
                : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'}"
            onclick={() => selectEvent(event)}
          >
            <div class="min-w-0">
              <p class="text-sm font-medium truncate">{event}</p>
              <p class="text-[10px] text-text-muted truncate">{HOOK_EVENT_DESCRIPTIONS[event]}</p>
            </div>
            {#if count > 0}
              <span class="ml-2 text-xs px-2 py-0.5 rounded-full bg-accent/20 text-accent shrink-0">{count}</span>
            {/if}
          </button>
        {/each}
      </div>

      <!-- Right Panel -->
      <div class="flex-1 overflow-y-auto p-6 space-y-4">
        {#if !selectedEvent}
          <div class="flex flex-col items-center justify-center h-full text-text-muted">
            <Zap size={32} class="opacity-20 mb-3" />
            <p class="text-sm">Select an event to configure hooks</p>
            <p class="text-xs mt-1">Hooks run automatically when events occur in Claude Code</p>
          </div>
        {:else}
          <div>
            <h3 class="text-base font-medium text-text-primary">{selectedEvent}</h3>
            <p class="text-xs text-text-muted">{HOOK_EVENT_DESCRIPTIONS[selectedEvent]}</p>
          </div>

          <!-- Existing hooks for this event -->
          <div class="space-y-3">
            {#each eventHooks() as flat}
              <HookCard
                event={flat.event}
                matcher={flat.matcher}
                handler={flat.handler}
                onupdate={(h) => updateHandler(flat, h)}
                ondelete={() => (deleteTarget = flat)}
              />
            {/each}
          </div>

          <!-- Add form -->
          {#if showAddForm}
            <div class="bg-bg-secondary border border-accent/30 rounded-lg p-4 space-y-3">
              <div class="grid grid-cols-2 gap-3">
                <label class="block">
                  <span class="text-xs text-text-muted">Matcher <span class="text-text-muted">(e.g. Bash, Edit, * for all)</span></span>
                  <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder="*" bind:value={newMatcher} />
                </label>
                <div>
                  <span class="text-xs text-text-muted">Type</span>
                  <div class="flex gap-1 mt-1" role="group" aria-label="Hook type">
                    {#each (["command", "http", "prompt", "agent"] as const) as type}
                      <button class="px-2.5 py-1.5 text-xs rounded-md transition-colors {newType === type ? 'bg-accent text-white' : 'bg-bg-tertiary text-text-muted'}" onclick={() => (newType = type)}>{type}</button>
                    {/each}
                  </div>
                </div>
              </div>
              <label class="block">
                <span class="text-xs text-text-muted">{newType === "command" ? "Command" : newType === "http" ? "URL" : "Prompt"}</span>
                <input type="text" class="w-full mt-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono focus:outline-none focus:border-accent" placeholder={newType === "command" ? "/path/to/script.sh" : newType === "http" ? "https://..." : "Describe what to check..."} bind:value={newValue} />
              </label>
              <div class="flex justify-end gap-2">
                <button class="px-4 py-1.5 text-sm text-text-muted hover:text-text-secondary" onclick={() => (showAddForm = false)}>Cancel</button>
                <button class="px-4 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md" onclick={addFromForm}>Add</button>
              </div>
            </div>
          {:else}
            <button
              class="w-full flex items-center justify-center gap-2 py-3 border-2 border-dashed border-border rounded-lg text-sm text-text-muted hover:border-accent/50 hover:text-accent transition-colors"
              onclick={() => (showAddForm = true)}
            >
              <Plus size={16} />
              Add Hook to {selectedEvent}
            </button>
          {/if}

        {/if}
      </div>
    </div>
  {/if}
</div>

<TemplateGallery
  open={galleryOpen}
  defaultCategory="hook"
  onselect={(template) => {
    if (!selectedEvent && template.event) {
      selectedEvent = template.event as HookEvent;
    }
    const event = template.event ?? selectedEvent ?? "PreToolUse";
    const matcher = template.matcher || undefined;
    const hookType = template.hookType ?? "command";
    const handler: HookHandler = { type: hookType };
    if (hookType === "command") handler.command = template.hookValue ?? "echo 'hook'";
    else if (hookType === "http") handler.url = template.hookValue ?? "http://localhost:8080/hook";
    else if (hookType === "prompt") handler.prompt = template.hookValue ?? "Validate this action";
    else handler.prompt = template.hookValue ?? "";
    addHook(event, matcher, handler);
  }}
  onclose={() => (galleryOpen = false)}
/>
