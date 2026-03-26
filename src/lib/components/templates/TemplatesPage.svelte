<script lang="ts">
  import { api } from "$lib/tauri/commands";
  import { getSelectedProjectPath } from "$lib/stores/project-context.svelte";
  import type { HookHandler } from "$lib/types";
  import {
    Search,
    Sparkles,
    Bot,
    Shield,
    Zap,
    Server,
    LayoutGrid,
    Check,
    Download,
  } from "lucide-svelte";

  interface Template {
    id: string;
    name: string;
    description: string;
    category: "skill" | "agent" | "rule" | "hook" | "mcp";
    content?: string;
    paths?: string[];
    event?: string;
    matcher?: string;
    hookType?: "command" | "http" | "prompt" | "agent";
    hookValue?: string;
    mcpType?: "stdio" | "sse";
    mcpCommand?: string;
    mcpArgs?: string;
    mcpUrl?: string;
  }

  let activeCategory = $state<Template["category"] | "all">("all");
  let search = $state("");
  let installing = $state<string | null>(null);
  let installed = $state<Set<string>>(new Set());

  const projectPath = $derived(getSelectedProjectPath());

  const CATEGORIES = [
    { id: "all" as const, label: "All", icon: LayoutGrid, color: "" },
    {
      id: "skill" as const,
      label: "Skills",
      icon: Sparkles,
      color: "text-warning",
    },
    { id: "agent" as const, label: "Agents", icon: Bot, color: "text-accent" },
    { id: "rule" as const, label: "Rules", icon: Shield, color: "text-info" },
    { id: "hook" as const, label: "Hooks", icon: Zap, color: "text-success" },
    { id: "mcp" as const, label: "MCP", icon: Server, color: "text-danger" },
  ];

  const CATEGORY_COLORS: Record<string, string> = {
    skill: "bg-warning/10 text-warning border-warning/20",
    agent: "bg-accent/10 text-accent border-accent/20",
    rule: "bg-info/10 text-info border-info/20",
    hook: "bg-success/10 text-success border-success/20",
    mcp: "bg-danger/10 text-danger border-danger/20",
  };

  const TEMPLATES: Template[] = [
    // Skills
    {
      id: "skill-code-review",
      name: "Code Review",
      description: "Review code for quality, bugs, and best practices",
      category: "skill",
      content:
        '---\nname: code-review\ndescription: Review code for quality, bugs, and best practices\nuser-invocable: true\nargument-hint: "[file or PR]"\n---\n\nReview the specified code for:\n- Logic errors and edge cases\n- Performance issues\n- Security vulnerabilities\n- Code style and readability\n\nProvide actionable feedback with specific line references.\n',
    },
    {
      id: "skill-deploy",
      name: "Deploy",
      description: "Deploy application to environment",
      category: "skill",
      content:
        '---\nname: deploy\ndescription: Deploy the application\ndisable-model-invocation: true\nuser-invocable: true\nargument-hint: "[environment]"\nallowed-tools: Bash\n---\n\n## Deploy to $ARGUMENTS\n\n1. Run tests\n2. Build the application\n3. Deploy to $ARGUMENTS environment\n4. Verify deployment\n',
    },
    {
      id: "skill-explain",
      name: "Explain Code",
      description: "Explain code structure and logic in detail",
      category: "skill",
      content:
        '---\nname: explain-code\ndescription: Explain code structure and logic\nuser-invocable: true\nargument-hint: "[file path]"\ncontext: fork\nagent: Explore\n---\n\nExplain $ARGUMENTS in detail:\n1. What it does\n2. How it works\n3. Key patterns used\n4. Dependencies\n',
    },
    {
      id: "skill-test-runner",
      name: "Test Runner",
      description: "Run and analyze test results",
      category: "skill",
      content:
        "---\nname: test-runner\ndescription: Run tests and analyze results\nuser-invocable: true\nallowed-tools: Bash, Read\n---\n\nRun the project's test suite:\n1. Identify the test framework\n2. Run all tests\n3. Analyze failures\n4. Suggest fixes\n",
    },
    {
      id: "skill-pr-review",
      name: "PR Review",
      description: "Review pull request changes",
      category: "skill",
      content:
        '---\nname: pr-review\ndescription: Review a pull request\nuser-invocable: true\nargument-hint: "[PR number]"\nallowed-tools: Bash, Read, Glob, Grep\n---\n\nReview PR #$ARGUMENTS:\n1. Get diff: !`gh pr diff $0`\n2. Analyze for correctness, security, performance\n3. Provide approve/changes recommendation\n',
    },
    {
      id: "skill-refactor",
      name: "Refactor",
      description: "Refactor code for better structure",
      category: "skill",
      content:
        '---\nname: refactor\ndescription: Refactor code for better structure\nuser-invocable: true\nargument-hint: "[file or module]"\n---\n\nRefactor $ARGUMENTS:\n1. Identify code smells\n2. Extract functions/modules\n3. Simplify complex logic\n4. Ensure tests pass\n',
    },

    // Agents
    {
      id: "agent-bug-fixer",
      name: "Bug Fixer",
      description: "Investigate and fix reported bugs",
      category: "agent",
      content:
        "---\nname: bug-fixer\ndescription: Investigate and fix reported bugs\nmodel: opus\neffort: high\ntools: Read, Glob, Grep, Edit, Bash\npermissions: acceptEdits\n---\n\nYou are an expert debugger:\n1. Reproduce the issue\n2. Identify root cause\n3. Implement minimal fix\n4. Verify tests pass\n",
    },
    {
      id: "agent-security",
      name: "Security Auditor",
      description: "Audit code for security vulnerabilities",
      category: "agent",
      content:
        "---\nname: security-auditor\ndescription: Audit code for security vulnerabilities\nmodel: opus\ntools: Read, Glob, Grep\npermissions: plan\nmemory: project\n---\n\nAnalyze code for OWASP Top 10, injection attacks, auth flaws, secrets in code.\nProvide severity ratings and remediation steps.\n",
    },
    {
      id: "agent-docs",
      name: "Docs Generator",
      description: "Generate comprehensive documentation",
      category: "agent",
      content:
        "---\nname: docs-generator\ndescription: Generate comprehensive documentation\nmodel: sonnet\ntools: Read, Glob, Grep, Write\n---\n\nGenerate documentation:\n1. JSDoc/TSDoc comments\n2. README sections\n3. Usage examples\n",
    },
    {
      id: "agent-perf",
      name: "Performance Optimizer",
      description: "Analyze and optimize code performance",
      category: "agent",
      content:
        "---\nname: performance-optimizer\ndescription: Optimize code performance\nmodel: opus\ntools: Read, Glob, Grep, Edit, Bash\nmemory: project\n---\n\nProfile hot paths, identify bottlenecks, suggest optimizations with benchmarks.\n",
    },

    // Rules
    {
      id: "rule-ts",
      name: "TypeScript Strict",
      description: "Enforce strict TypeScript patterns",
      category: "rule",
      paths: ["**/*.ts", "**/*.tsx"],
      content:
        "# TypeScript Rules\n\n- Strict mode, no `any`\n- Named exports over defaults\n- Explicit return types\n- No unused variables",
    },
    {
      id: "rule-api",
      name: "API Design",
      description: "REST API conventions",
      category: "rule",
      paths: ["src/api/**/*"],
      content:
        "# API Rules\n\n- Validate all input\n- Standard error format\n- Proper HTTP status codes\n- Pagination for lists",
    },
    {
      id: "rule-testing",
      name: "Testing",
      description: "Test standards and patterns",
      category: "rule",
      paths: ["**/*.test.*"],
      content:
        "# Testing Rules\n\n- Co-locate tests with source\n- Descriptive names\n- No test dependencies\n- 80%+ coverage on critical paths",
    },
    {
      id: "rule-security",
      name: "Security",
      description: "Security best practices",
      category: "rule",
      paths: [],
      content:
        "# Security Rules\n\n- Never commit secrets\n- Sanitize user input\n- Parameterized queries\n- HTTPS for external APIs",
    },
    {
      id: "rule-git",
      name: "Git Conventions",
      description: "Commit and branching standards",
      category: "rule",
      paths: [],
      content:
        "# Git Rules\n\n- Conventional commits\n- Atomic commits\n- Imperative mood\n- Reference issues",
    },
    {
      id: "rule-errors",
      name: "Error Handling",
      description: "Error handling patterns",
      category: "rule",
      paths: ["src/**/*"],
      content:
        "# Error Handling\n\n- Never swallow errors\n- Custom error classes\n- Include context\n- User-friendly messages",
    },
    {
      id: "rule-a11y",
      name: "Accessibility",
      description: "UI accessibility standards",
      category: "rule",
      paths: ["**/*.svelte", "**/*.tsx"],
      content:
        "# Accessibility\n\n- Alt text on images\n- Semantic HTML\n- Keyboard navigation\n- Color contrast\n- Aria labels",
    },
    {
      id: "rule-perf",
      name: "Performance",
      description: "Performance guidelines",
      category: "rule",
      paths: [],
      content:
        "# Performance\n\n- Avoid N+1 queries\n- Pagination\n- Lazy loading\n- Profile before optimizing",
    },

    // Hooks
    {
      id: "hook-bash",
      name: "Bash Validator",
      description: "Validate bash commands before execution",
      category: "hook",
      event: "PreToolUse",
      matcher: "Bash",
      hookType: "command",
      hookValue: ".claude/hooks/validate.sh",
    },
    {
      id: "hook-webhook",
      name: "HTTP Webhook",
      description: "Send events to an endpoint",
      category: "hook",
      event: "PreToolUse",
      matcher: "",
      hookType: "http",
      hookValue: "http://localhost:8080/hook",
    },
    {
      id: "hook-guard",
      name: "Prompt Guard",
      description: "AI validates actions",
      category: "hook",
      event: "PreToolUse",
      matcher: "",
      hookType: "prompt",
      hookValue: "Check if this action is safe",
    },
    {
      id: "hook-log",
      name: "Log Usage",
      description: "Log tool usage to file",
      category: "hook",
      event: "PostToolUse",
      matcher: "",
      hookType: "command",
      hookValue: 'echo "$(date): tool used" >> ~/.claude/usage.log',
    },
    {
      id: "hook-cleanup",
      name: "Session Cleanup",
      description: "Run cleanup on session end",
      category: "hook",
      event: "SessionEnd",
      matcher: "",
      hookType: "command",
      hookValue: "echo 'Session ended'",
    },

    // MCP
    {
      id: "mcp-fs",
      name: "Filesystem",
      description: "Read/write local files",
      category: "mcp",
      mcpType: "stdio",
      mcpCommand: "npx",
      mcpArgs: "-y @modelcontextprotocol/server-filesystem /path/to/dir",
    },
    {
      id: "mcp-github",
      name: "GitHub",
      description: "GitHub API access",
      category: "mcp",
      mcpType: "stdio",
      mcpCommand: "npx",
      mcpArgs: "-y @modelcontextprotocol/server-github",
    },
    {
      id: "mcp-postgres",
      name: "PostgreSQL",
      description: "Query PostgreSQL databases",
      category: "mcp",
      mcpType: "stdio",
      mcpCommand: "npx",
      mcpArgs:
        "-y @modelcontextprotocol/server-postgres postgresql://localhost/mydb",
    },
    {
      id: "mcp-memory",
      name: "Memory",
      description: "Persistent memory store",
      category: "mcp",
      mcpType: "stdio",
      mcpCommand: "npx",
      mcpArgs: "-y @modelcontextprotocol/server-memory",
    },
    {
      id: "mcp-sqlite",
      name: "SQLite",
      description: "Query SQLite databases",
      category: "mcp",
      mcpType: "stdio",
      mcpCommand: "npx",
      mcpArgs: "-y @modelcontextprotocol/server-sqlite /path/to/db.sqlite",
    },
  ];

  const filtered = $derived(
    TEMPLATES.filter((t) => {
      if (activeCategory !== "all" && t.category !== activeCategory)
        return false;
      if (
        search &&
        !t.name.toLowerCase().includes(search.toLowerCase()) &&
        !t.description.toLowerCase().includes(search.toLowerCase())
      )
        return false;
      return true;
    }),
  );

  const categoryCounts = $derived(
    CATEGORIES.map((c) => ({
      ...c,
      count:
        c.id === "all"
          ? TEMPLATES.length
          : TEMPLATES.filter((t) => t.category === c.id).length,
    })),
  );

  async function installTemplate(template: Template) {
    installing = template.id;
    try {
      const pp = projectPath ?? undefined;
      const name = template.name.toLowerCase().replace(/\s+/g, "-");

      switch (template.category) {
        case "skill":
          await api.skills.write("global", name, template.content ?? "", pp);
          break;
        case "agent":
          await api.agents.write("global", name, template.content ?? "", pp);
          break;
        case "rule":
          await api.rules.write(
            "global",
            name + ".md",
            template.paths ?? [],
            template.content ?? "",
            pp,
          );
          break;
        case "hook": {
          const hooks = (await api.hooks.get("global")) as Record<
            string,
            Array<{ matcher?: string; hooks: HookHandler[] }>
          >;
          const event = template.event ?? "PreToolUse";
          const handler: HookHandler = { type: template.hookType ?? "command" };
          if (handler.type === "command") handler.command = template.hookValue;
          else if (handler.type === "http") handler.url = template.hookValue;
          else handler.prompt = template.hookValue;
          if (!hooks[event]) hooks[event] = [];
          hooks[event].push({
            matcher: template.matcher || undefined,
            hooks: [handler],
          });
          await api.hooks.set("global", hooks);
          break;
        }
        case "mcp":
          if (template.mcpUrl) {
            await api.mcp.upsert("global", name, { url: template.mcpUrl });
          } else {
            await api.mcp.upsert("global", name, {
              command: template.mcpCommand ?? "",
              args: (template.mcpArgs ?? "").split(/\s+/).filter(Boolean),
            });
          }
          break;
      }

      installed = new Set([...installed, template.id]);
    } catch (e) {
      console.error("Failed to install:", e);
    } finally {
      installing = null;
    }
  }
</script>

<div class="p-6 overflow-y-auto h-full space-y-4">
  <!-- Category cards -->
  <div class="flex gap-2 overflow-x-auto pb-1">
    {#each categoryCounts as cat}
      {@const Icon = cat.icon}
      <button
        class="flex items-center gap-2 px-4 py-3 rounded-lg border transition-colors shrink-0
          {activeCategory === cat.id
          ? 'bg-accent/10 border-accent/30 text-accent'
          : 'bg-bg-secondary border-border text-text-secondary hover:border-border-light'}"
        onclick={() => (activeCategory = cat.id)}
      >
        <Icon size={16} />
        <span class="text-sm font-medium">{cat.label}</span>
        <span class="text-xs opacity-60">{cat.count}</span>
      </button>
    {/each}
  </div>

  <!-- Search -->
  <div class="relative">
    <Search size={14} class="absolute left-3 top-2.5 text-text-muted" />
    <input
      type="text"
      class="w-full pl-9 pr-3 py-2 text-sm bg-bg-secondary border border-border rounded-lg text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent"
      placeholder="Search templates..."
      bind:value={search}
    />
  </div>

  <!-- Grid -->
  <div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
    {#each filtered as template}
      {@const isInstalled = installed.has(template.id)}
      <div
        class="bg-bg-secondary border border-border rounded-lg p-4 flex flex-col justify-between hover:border-accent/30 transition-colors group"
      >
        <div>
          <div class="flex items-center justify-between mb-2">
            <span
              class="text-[10px] px-2 py-0.5 rounded-full border {CATEGORY_COLORS[
                template.category
              ]}">{template.category}</span
            >
          </div>
          <p class="text-sm font-medium text-text-primary">{template.name}</p>
          <p class="text-xs text-text-muted mt-1 line-clamp-2">
            {template.description}
          </p>
        </div>
        <div class="mt-3">
          {#if isInstalled}
            <div class="flex items-center gap-1 text-xs text-success">
              <Check size={12} />
              Added
            </div>
          {:else}
            <button
              class="w-full flex items-center justify-center gap-1.5 py-1.5 text-xs bg-accent hover:bg-accent-hover text-white rounded-md transition-colors disabled:opacity-50"
              onclick={() => installTemplate(template)}
              disabled={installing === template.id}
            >
              {#if installing === template.id}
                Adding...
              {:else}
                <Download size={12} />
                Add to Global
              {/if}
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  {#if filtered.length === 0}
    <div class="text-center py-12">
      <Search size={24} class="mx-auto mb-3 opacity-20 text-text-muted" />
      <p class="text-sm text-text-muted">No templates match your search</p>
    </div>
  {/if}
</div>
