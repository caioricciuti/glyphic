<script lang="ts">
  import { NAV_ITEMS, getCurrentPage, navigateTo } from "$lib/stores/navigation.svelte";

  const currentPage = $derived(getCurrentPage());
</script>

<aside class="flex flex-col h-full w-60 bg-bg-secondary border-r border-border shrink-0">
  <!-- Logo -->
  <div class="flex items-center gap-2 px-4 py-4 border-b border-border">
    <div class="w-8 h-8 rounded-lg bg-accent flex items-center justify-center text-white font-bold text-sm">
      CC
    </div>
    <div>
      <h1 class="text-sm font-semibold text-text-primary">Claude Code</h1>
      <p class="text-xs text-text-muted">Control Panel</p>
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 py-2 overflow-y-auto">
    {#each NAV_ITEMS as item}
      <button
        class="w-full flex items-center gap-3 px-4 py-2.5 text-sm transition-colors
          {currentPage === item.id
            ? 'bg-accent-dim text-accent border-r-2 border-accent'
            : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'}"
        onclick={() => navigateTo(item.id)}
      >
        <span class="w-5 h-5 flex items-center justify-center">
          {#if item.icon === "chart"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M15.5 2A1.5 1.5 0 0014 3.5v13a1.5 1.5 0 001.5 1.5h1a1.5 1.5 0 001.5-1.5v-13A1.5 1.5 0 0016.5 2h-1zM9.5 6A1.5 1.5 0 008 7.5v9A1.5 1.5 0 009.5 18h1a1.5 1.5 0 001.5-1.5v-9A1.5 1.5 0 0010.5 6h-1zM3.5 10A1.5 1.5 0 002 11.5v5A1.5 1.5 0 003.5 18h1A1.5 1.5 0 006 16.5v-5A1.5 1.5 0 004.5 10h-1z" />
            </svg>
          {:else if item.icon === "gear"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path fill-rule="evenodd" d="M7.84 1.804A1 1 0 018.82 1h2.36a1 1 0 01.98.804l.331 1.652a6.993 6.993 0 011.929 1.115l1.598-.54a1 1 0 011.186.447l1.18 2.044a1 1 0 01-.205 1.251l-1.267 1.113a7.047 7.047 0 010 2.228l1.267 1.113a1 1 0 01.206 1.25l-1.18 2.045a1 1 0 01-1.187.447l-1.598-.54a6.993 6.993 0 01-1.929 1.115l-.33 1.652a1 1 0 01-.98.804H8.82a1 1 0 01-.98-.804l-.331-1.652a6.993 6.993 0 01-1.929-1.115l-1.598.54a1 1 0 01-1.186-.447l-1.18-2.044a1 1 0 01.205-1.251l1.267-1.114a7.05 7.05 0 010-2.227L1.821 7.773a1 1 0 01-.206-1.25l1.18-2.045a1 1 0 011.187-.447l1.598.54A6.993 6.993 0 017.51 3.456l.33-1.652zM10 13a3 3 0 100-6 3 3 0 000 6z" clip-rule="evenodd" />
            </svg>
          {:else if item.icon === "bolt"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M11.983 1.907a.75.75 0 00-1.292-.657l-8.5 9.5A.75.75 0 002.75 12h6.572l-1.305 6.093a.75.75 0 001.292.657l8.5-9.5A.75.75 0 0017.25 8h-6.572l1.305-6.093z" />
            </svg>
          {:else if item.icon === "book"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M10.75 16.82A7.462 7.462 0 0115 15.5c.71 0 1.396.098 2.046.282A.75.75 0 0018 15.06V3.94a.75.75 0 00-.546-.721A9.006 9.006 0 0015 3a8.963 8.963 0 00-4.25 1.065V16.82zM9.25 4.065A8.963 8.963 0 005 3c-.85 0-1.673.118-2.454.34A.75.75 0 002 4.06v11.12a.75.75 0 00.954.721A7.506 7.506 0 015 15.5c1.579 0 3.042.487 4.25 1.32V4.065z" />
            </svg>
          {:else if item.icon === "brain"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M10 .5a9.5 9.5 0 105.598 17.177C14.466 15.177 12.383 13 10 13s-4.466 2.177-5.598 4.677A9.5 9.5 0 0010 .5zM3.5 10a6.5 6.5 0 1113 0 6.5 6.5 0 01-13 0z" />
            </svg>
          {:else if item.icon === "server"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M1 4.75C1 3.784 1.784 3 2.75 3h14.5c.966 0 1.75.784 1.75 1.75v2.5A1.75 1.75 0 0117.25 9H2.75A1.75 1.75 0 011 7.25v-2.5zm0 8C1 11.784 1.784 11 2.75 11h14.5c.966 0 1.75.784 1.75 1.75v2.5A1.75 1.75 0 0117.25 17H2.75A1.75 1.75 0 011 15.25v-2.5zM14.5 6a1 1 0 100-2 1 1 0 000 2zm0 8a1 1 0 100-2 1 1 0 000 2z" />
            </svg>
          {:else if item.icon === "sparkles"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M10 1l2.928 5.856L19 8.292l-4.5 4.392L15.856 19 10 15.856 4.144 19l1.356-6.316L1 8.292l6.072-1.436L10 1z" />
            </svg>
          {:else if item.icon === "shield"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path fill-rule="evenodd" d="M9.661 2.237a.531.531 0 01.678 0 11.947 11.947 0 007.078 2.749.5.5 0 01.479.425c.069.52.104 1.05.104 1.59 0 5.162-3.26 9.563-7.834 11.256a.48.48 0 01-.332 0C5.26 16.564 2 12.163 2 7c0-.538.035-1.069.104-1.589a.5.5 0 01.48-.425 11.947 11.947 0 007.077-2.75z" clip-rule="evenodd" />
            </svg>
          {:else if item.icon === "puzzle"}
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" class="w-5 h-5">
              <path d="M6 3.75A2.75 2.75 0 018.75 1h2.5A2.75 2.75 0 0114 3.75v.443c.572.055 1.14.122 1.706.2C17.053 4.582 18 5.75 18 7.07v3.469c0 1.126-.694 2.191-1.83 2.54-1.952.601-4.018.96-6.17.96s-4.219-.359-6.17-.96C2.694 12.73 2 11.665 2 10.539V7.07c0-1.321.947-2.489 2.294-2.676A41.047 41.047 0 016 4.193V3.75zm6.5 0v.325a41.622 41.622 0 00-5 0V3.75c0-.69.56-1.25 1.25-1.25h2.5c.69 0 1.25.56 1.25 1.25zM10 10a1 1 0 00-1 1v.01a1 1 0 001 1h.01a1 1 0 001-1V11a1 1 0 00-1-1H10z" />
            </svg>
          {/if}
        </span>
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <!-- XP Bar (placeholder) -->
  <div class="p-4 border-t border-border">
    <div class="flex items-center justify-between text-xs text-text-muted mb-1">
      <span>Level 1 — Newcomer</span>
      <span>0 XP</span>
    </div>
    <div class="w-full h-2 bg-bg-tertiary rounded-full overflow-hidden">
      <div class="h-full bg-accent rounded-full transition-all duration-500" style="width: 0%"></div>
    </div>
  </div>
</aside>
