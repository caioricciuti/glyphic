<script lang="ts">
  import { onMount } from "svelte";
  import { Download, X, RefreshCw } from "lucide-svelte";

  let updateAvailable = $state(false);
  let updateVersion = $state("");
  let downloading = $state(false);
  let dismissed = $state(false);

  onMount(async () => {
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        updateAvailable = true;
        updateVersion = update.version;
      }
    } catch {
      // Updater not configured or no update — silent
    }
  });

  async function installUpdate() {
    downloading = true;
    try {
      const { check } = await import("@tauri-apps/plugin-updater");
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        // Relaunch after install
        const { relaunch } = await import("@tauri-apps/plugin-process");
        await relaunch();
      }
    } catch (e) {
      console.error("Update failed:", e);
      downloading = false;
    }
  }
</script>

{#if updateAvailable && !dismissed}
  <div class="flex items-center justify-between px-4 py-2 bg-accent/10 border-b border-accent/20 shrink-0">
    <div class="flex items-center gap-2 text-xs">
      <Download size={14} class="text-accent" />
      <span class="text-text-primary">Glyphic <strong>{updateVersion}</strong> is available</span>
    </div>
    <div class="flex items-center gap-2">
      <button
        class="flex items-center gap-1 px-3 py-1 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
        onclick={installUpdate}
        disabled={downloading}
      >
        {#if downloading}
          <RefreshCw size={12} class="animate-spin" />
          Updating...
        {:else}
          Update Now
        {/if}
      </button>
      <button
        class="p-1 text-text-muted hover:text-text-primary"
        onclick={() => (dismissed = true)}
        aria-label="Dismiss"
      >
        <X size={14} />
      </button>
    </div>
  </div>
{/if}
