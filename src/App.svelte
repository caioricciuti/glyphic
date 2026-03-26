<script lang="ts">
  import Sidebar from "$lib/components/layout/Sidebar.svelte";
  import Header from "$lib/components/layout/Header.svelte";
  import UpdateBanner from "$lib/components/layout/UpdateBanner.svelte";
  import DashboardPage from "$lib/components/dashboard/DashboardPage.svelte";
  import SettingsPage from "$lib/components/settings/SettingsPage.svelte";
  import HooksPage from "$lib/components/hooks/HooksPage.svelte";
  import InstructionsPage from "$lib/components/instructions/InstructionsPage.svelte";
  import MemoryPage from "$lib/components/memory/MemoryPage.svelte";
  import McpPage from "$lib/components/mcp/McpPage.svelte";
  import SkillsPage from "$lib/components/skills/SkillsPage.svelte";
  import RulesPage from "$lib/components/rules/RulesPage.svelte";
  import PluginsPage from "$lib/components/plugins/PluginsPage.svelte";
  import GitPage from "$lib/components/git/GitPage.svelte";
  import SessionsPage from "$lib/components/sessions/SessionsPage.svelte";
  import TemplatesPage from "$lib/components/templates/TemplatesPage.svelte";
  import TerminalPage from "$lib/components/terminal/TerminalPage.svelte";
  import AnalyticsPage from "$lib/components/analytics/AnalyticsPage.svelte";
  import { getCurrentPage } from "$lib/stores/navigation.svelte";

  const currentPage = $derived(getCurrentPage());
</script>

<div class="flex h-screen w-screen">
  <Sidebar />

  <main class="flex flex-col flex-1 min-w-0">
    <UpdateBanner />
    <Header />

    <div class="flex-1 overflow-hidden">
      {#if currentPage === "dashboard"}
        <DashboardPage />
      {:else if currentPage === "settings"}
        <SettingsPage />
      {:else if currentPage === "hooks"}
        <HooksPage />
      {:else if currentPage === "instructions"}
        <InstructionsPage />
      {:else if currentPage === "memory"}
        <MemoryPage />
      {:else if currentPage === "mcp"}
        <McpPage />
      {:else if currentPage === "skills"}
        <SkillsPage />
      {:else if currentPage === "rules"}
        <RulesPage />
      {:else if currentPage === "plugins"}
        <PluginsPage />
      {:else if currentPage === "git"}
        <GitPage />
      {:else if currentPage === "sessions"}
        <SessionsPage />
      {:else if currentPage === "templates"}
        <TemplatesPage />
      {:else if currentPage === "terminal"}
        <TerminalPage />
      {:else if currentPage === "analytics"}
        <AnalyticsPage />
      {/if}
    </div>
  </main>
</div>
