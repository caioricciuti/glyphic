<script lang="ts">
  import { AlertTriangle } from "lucide-svelte";

  interface Props {
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    onconfirm: () => void;
    oncancel: () => void;
  }

  const { open, title, message, confirmLabel = "Delete", onconfirm, oncancel }: Props = $props();
</script>

{#if open}
  <!-- Backdrop -->
  <div class="fixed inset-0 z-50 flex items-center justify-center">
    <button
      class="absolute inset-0 bg-black/50"
      onclick={oncancel}
      aria-label="Close dialog"
    ></button>

    <!-- Dialog -->
    <div class="relative bg-bg-secondary border border-border rounded-xl shadow-2xl w-96 p-6 space-y-4 z-10">
      <div class="flex items-start gap-3">
        <div class="w-10 h-10 rounded-full bg-danger/10 flex items-center justify-center shrink-0">
          <AlertTriangle size={20} class="text-danger" />
        </div>
        <div>
          <h3 class="text-base font-semibold text-text-primary">{title}</h3>
          <p class="text-sm text-text-muted mt-1">{message}</p>
        </div>
      </div>

      <div class="flex justify-end gap-2 pt-2">
        <button
          class="px-4 py-2 text-sm text-text-secondary bg-bg-tertiary hover:bg-bg-hover rounded-lg transition-colors"
          onclick={oncancel}
        >
          Cancel
        </button>
        <button
          class="px-4 py-2 text-sm text-white bg-danger hover:bg-danger/80 rounded-lg transition-colors"
          onclick={onconfirm}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
