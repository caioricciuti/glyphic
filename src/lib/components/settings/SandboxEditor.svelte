<script lang="ts">
  import type { Settings, SandboxSettings } from "$lib/types";
  import { Shield } from "lucide-svelte";

  interface Props {
    settings: Settings;
  }

  let { settings = $bindable() }: Props = $props();

  const sandbox = $derived(settings.sandbox ?? {});
  const filesystem = $derived(sandbox.filesystem ?? {});
  const network = $derived(sandbox.network ?? {});
  const credentials = $derived(sandbox.credentials ?? {});

  let newDomain = $state("");
  let newPath = $state("");
  let newPathList = $state<"allowWrite" | "denyWrite" | "allowRead" | "denyRead">("allowWrite");
  let newCredKind = $state<"file" | "envVar">("envVar");
  let newCredValue = $state("");
  let newCredMode = $state<"deny" | "mask">("deny");

  function patch(partial: Partial<SandboxSettings>) {
    settings = { ...settings, sandbox: { ...sandbox, ...partial } };
  }

  function addDomain() {
    const d = newDomain.trim();
    if (!d) return;
    const domains = network.allowedDomains ?? [];
    if (!domains.includes(d)) {
      patch({ network: { ...network, allowedDomains: [...domains, d] } });
    }
    newDomain = "";
  }

  function removeDomain(d: string) {
    patch({ network: { ...network, allowedDomains: (network.allowedDomains ?? []).filter((x) => x !== d) } });
  }

  function addPath() {
    const p = newPath.trim();
    if (!p) return;
    const list = filesystem[newPathList] ?? [];
    if (!list.includes(p)) {
      patch({ filesystem: { ...filesystem, [newPathList]: [...list, p] } });
    }
    newPath = "";
  }

  function removePath(key: "allowWrite" | "denyWrite" | "allowRead" | "denyRead", p: string) {
    patch({ filesystem: { ...filesystem, [key]: (filesystem[key] ?? []).filter((x) => x !== p) } });
  }

  function addCredential() {
    const v = newCredValue.trim();
    if (!v) return;
    if (newCredKind === "file") {
      const files = credentials.files ?? [];
      if (!files.some((f) => f.path === v)) {
        patch({ credentials: { ...credentials, files: [...files, { path: v, mode: newCredMode }] } });
      }
    } else {
      const envVars = credentials.envVars ?? [];
      if (!envVars.some((e) => e.name === v)) {
        patch({ credentials: { ...credentials, envVars: [...envVars, { name: v, mode: newCredMode }] } });
      }
    }
    newCredValue = "";
  }

  function removeCredFile(path: string) {
    patch({ credentials: { ...credentials, files: (credentials.files ?? []).filter((f) => f.path !== path) } });
  }

  function removeCredEnvVar(name: string) {
    patch({ credentials: { ...credentials, envVars: (credentials.envVars ?? []).filter((e) => e.name !== name) } });
  }

  const pathListLabels = {
    allowWrite: { label: "Allow write", color: "text-success" },
    denyWrite: { label: "Deny write", color: "text-danger" },
    allowRead: { label: "Allow read", color: "text-success" },
    denyRead: { label: "Deny read", color: "text-danger" },
  } as const;
</script>

<div class="bg-bg-secondary border border-border rounded-lg p-4 space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-text-secondary flex items-center gap-1.5">
      <Shield size={14} />
      Sandbox
    </h3>
    <label class="flex items-center gap-2 text-sm text-text-primary cursor-pointer">
      <input
        type="checkbox"
        class="accent-accent"
        checked={sandbox.enabled ?? false}
        onchange={(e) => patch({ enabled: (e.target as HTMLInputElement).checked })}
      />
      Enabled
    </label>
  </div>
  <p class="text-xs text-text-muted">
    OS-enforced filesystem and network isolation for Bash commands. macOS, Linux, and WSL2 only.
  </p>

  {#if sandbox.enabled}
    <!-- Unsandboxed fallback -->
    <label class="flex items-center justify-between cursor-pointer">
      <div>
        <span class="text-sm text-text-primary">Allow unsandboxed fallback</span>
        <p class="text-xs text-text-muted">Commands that fail under the sandbox may re-run unsandboxed (with approval)</p>
      </div>
      <input
        type="checkbox"
        class="accent-accent"
        checked={sandbox.allowUnsandboxedCommands ?? false}
        onchange={(e) => patch({ allowUnsandboxedCommands: (e.target as HTMLInputElement).checked })}
      />
    </label>

    <!-- Network -->
    <div>
      <p class="text-xs uppercase tracking-wider text-text-muted mb-2">Network — allowed domains</p>
      <div class="flex gap-2 mb-2">
        <input
          type="text"
          class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono placeholder:text-text-muted focus:outline-none focus:border-accent"
          placeholder="github.com, *.npmjs.org"
          bind:value={newDomain}
          onkeydown={(e) => e.key === "Enter" && addDomain()}
        />
        <button class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={addDomain}>Add</button>
      </div>
      {#if (network.allowedDomains ?? []).length > 0}
        <div class="flex flex-wrap gap-1.5">
          {#each network.allowedDomains ?? [] as d}
            <span class="inline-flex items-center gap-1.5 px-2 py-1 bg-bg-tertiary rounded-md text-xs font-mono text-text-primary">
              {d}
              <button class="text-text-muted hover:text-danger" onclick={() => removeDomain(d)} aria-label="Remove {d}">×</button>
            </span>
          {/each}
        </div>
      {:else}
        <p class="text-xs text-text-muted">No allowlist: sandboxed commands have no network restrictions from this scope.</p>
      {/if}
    </div>

    <!-- Filesystem -->
    <div>
      <div class="flex items-center justify-between mb-2">
        <p class="text-xs uppercase tracking-wider text-text-muted">Filesystem</p>
        <label class="flex items-center gap-2 text-xs text-text-muted cursor-pointer">
          <input
            type="checkbox"
            class="accent-accent"
            checked={filesystem.disabled ?? false}
            onchange={(e) => patch({ filesystem: { ...filesystem, disabled: (e.target as HTMLInputElement).checked } })}
          />
          Disable filesystem isolation (keep network only)
        </label>
      </div>
      {#if !filesystem.disabled}
        <div class="flex gap-2 mb-2">
          <select class="px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent" bind:value={newPathList}>
            {#each Object.entries(pathListLabels) as [key, meta]}
              <option value={key}>{meta.label}</option>
            {/each}
          </select>
          <input
            type="text"
            class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono placeholder:text-text-muted focus:outline-none focus:border-accent"
            placeholder="~/.kube, /tmp/build, ./output"
            bind:value={newPath}
            onkeydown={(e) => e.key === "Enter" && addPath()}
          />
          <button class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={addPath}>Add</button>
        </div>
        {#each Object.entries(pathListLabels) as [key, meta]}
          {@const k = key as keyof typeof pathListLabels}
          {@const paths = filesystem[k] ?? []}
          {#if paths.length > 0}
            <p class="text-xs {meta.color} mt-2 mb-1">{meta.label}</p>
            <div class="space-y-1">
              {#each paths as p}
                <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group">
                  <code class="text-sm text-text-primary font-mono">{p}</code>
                  <button class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs" onclick={() => removePath(k, p)}>remove</button>
                </div>
              {/each}
            </div>
          {/if}
        {/each}
      {/if}
    </div>

    <!-- Credentials -->
    <div>
      <p class="text-xs uppercase tracking-wider text-text-muted mb-2">Protected credentials</p>
      <div class="flex gap-2 mb-2">
        <select class="px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent" bind:value={newCredKind}>
          <option value="envVar">Env var</option>
          <option value="file">File</option>
        </select>
        <select class="px-2 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary focus:outline-none focus:border-accent" bind:value={newCredMode}>
          <option value="deny">Deny</option>
          <option value="mask">Mask</option>
        </select>
        <input
          type="text"
          class="flex-1 px-3 py-1.5 text-sm bg-bg-tertiary border border-border rounded-md text-text-primary font-mono placeholder:text-text-muted focus:outline-none focus:border-accent"
          placeholder={newCredKind === "envVar" ? "GITHUB_TOKEN" : "~/.aws/credentials"}
          bind:value={newCredValue}
          onkeydown={(e) => e.key === "Enter" && addCredential()}
        />
        <button class="px-3 py-1.5 text-sm bg-accent hover:bg-accent-hover text-white rounded-md transition-colors" onclick={addCredential}>Add</button>
      </div>
      {#each credentials.envVars ?? [] as ev}
        <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group mb-1">
          <span class="text-sm font-mono text-text-primary">{ev.name} <span class="text-xs {ev.mode === 'deny' ? 'text-danger' : 'text-warning'}">({ev.mode})</span></span>
          <button class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs" onclick={() => removeCredEnvVar(ev.name)}>remove</button>
        </div>
      {/each}
      {#each credentials.files ?? [] as f}
        <div class="flex items-center justify-between px-3 py-1.5 bg-bg-tertiary rounded-md group mb-1">
          <span class="text-sm font-mono text-text-primary">{f.path} <span class="text-xs {f.mode === 'deny' ? 'text-danger' : 'text-warning'}">({f.mode})</span></span>
          <button class="text-text-muted hover:text-danger opacity-0 group-hover:opacity-100 transition-opacity text-xs" onclick={() => removeCredFile(f.path)}>remove</button>
        </div>
      {/each}
      <p class="text-xs text-text-muted mt-1">Deny blocks the credential inside the sandbox. Mask shows commands a placeholder and injects the real value only on outbound requests to allowed hosts (needs TLS termination, edit injectHosts in raw JSON for per-host control).</p>
    </div>
  {/if}
</div>
