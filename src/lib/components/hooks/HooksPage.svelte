<script lang="ts">
  const HOOK_EVENTS = [
    "PreToolUse", "PostToolUse", "PostToolUseFailure",
    "SessionStart", "SessionEnd", "PermissionRequest",
    "UserPromptSubmit", "Notification", "SubagentStart",
    "SubagentStop", "Stop", "StopFailure", "ConfigChange",
    "PreCompact", "PostCompact", "InstructionsLoaded",
    "WorktreeCreate", "WorktreeRemove", "Elicitation",
    "ElicitationResult", "TeammateIdle", "TaskCompleted",
  ] as const;
</script>

<div class="p-6 overflow-y-auto h-full">
  <div class="grid grid-cols-2 gap-3">
    {#each HOOK_EVENTS as event}
      <button class="flex items-center justify-between p-3 bg-bg-secondary border border-border rounded-lg text-left hover:border-border-light transition-colors">
        <div>
          <p class="text-sm font-medium text-text-primary">{event}</p>
          <p class="text-xs text-text-muted mt-0.5">No hooks configured</p>
        </div>
        <span class="text-xs px-2 py-0.5 rounded-full bg-bg-tertiary text-text-muted">0</span>
      </button>
    {/each}
  </div>
</div>
