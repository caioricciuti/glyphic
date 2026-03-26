<script lang="ts">
  import { Search, X, Sparkles, Bot, Shield, Zap, Server, LayoutGrid } from "lucide-svelte";

  export interface Template {
    id: string;
    name: string;
    description: string;
    category: "skill" | "agent" | "rule" | "hook" | "mcp";
    content?: string;
    // For rules
    paths?: string[];
    // For hooks
    event?: string;
    matcher?: string;
    hookType?: "command" | "http" | "prompt" | "agent";
    hookValue?: string;
    // For MCP
    mcpType?: "stdio" | "sse";
    mcpCommand?: string;
    mcpArgs?: string;
    mcpUrl?: string;
  }

  interface Props {
    open: boolean;
    defaultCategory?: Template["category"] | "all";
    onselect: (template: Template) => void;
    onclose: () => void;
  }

  const { open, defaultCategory = "all", onselect, onclose }: Props = $props();

  let activeCategory = $state<Template["category"] | "all">(defaultCategory);
  let search = $state("");

  // Reset when opening
  $effect(() => {
    if (open) {
      activeCategory = defaultCategory;
      search = "";
    }
  });

  const CATEGORIES = [
    { id: "all" as const, label: "All", icon: LayoutGrid },
    { id: "skill" as const, label: "Skills", icon: Sparkles },
    { id: "agent" as const, label: "Agents", icon: Bot },
    { id: "rule" as const, label: "Rules", icon: Shield },
    { id: "hook" as const, label: "Hooks", icon: Zap },
    { id: "mcp" as const, label: "MCP", icon: Server },
  ];

  const TEMPLATES: Template[] = [
    // Skills
    { id: "skill-code-review", name: "Code Review", description: "Review code for quality, bugs, and best practices", category: "skill", content: "---\nname: code-review\ndescription: Review code for quality, bugs, and best practices\nuser-invocable: true\nargument-hint: \"[file or PR]\"\n---\n\nReview the specified code for:\n- Logic errors and edge cases\n- Performance issues\n- Security vulnerabilities\n- Code style and readability\n\nProvide actionable feedback with specific line references.\n" },
    { id: "skill-deploy", name: "Deploy", description: "Deploy application to environment", category: "skill", content: "---\nname: deploy\ndescription: Deploy the application\ndisable-model-invocation: true\nuser-invocable: true\nargument-hint: \"[environment]\"\nallowed-tools: Bash\n---\n\n## Deploy to $ARGUMENTS\n\n1. Run tests\n2. Build the application\n3. Deploy to $ARGUMENTS environment\n4. Verify deployment\n" },
    { id: "skill-explain", name: "Explain Code", description: "Explain code structure and logic in detail", category: "skill", content: "---\nname: explain-code\ndescription: Explain code structure and logic\nuser-invocable: true\nargument-hint: \"[file path]\"\ncontext: fork\nagent: Explore\n---\n\nExplain $ARGUMENTS in detail:\n1. What it does\n2. How it works\n3. Key patterns used\n4. Dependencies\n" },
    { id: "skill-test-runner", name: "Test Runner", description: "Run and analyze test results", category: "skill", content: "---\nname: test-runner\ndescription: Run tests and analyze results\nuser-invocable: true\nallowed-tools: Bash, Read\n---\n\nRun the project's test suite:\n1. Identify the test framework\n2. Run all tests\n3. Analyze failures\n4. Suggest fixes for failing tests\n" },
    { id: "skill-pr-review", name: "PR Review", description: "Review pull request changes", category: "skill", content: "---\nname: pr-review\ndescription: Review a pull request\nuser-invocable: true\nargument-hint: \"[PR number]\"\nallowed-tools: Bash, Read, Glob, Grep\n---\n\nReview PR #$ARGUMENTS:\n\n1. Get PR diff: !`gh pr diff $0`\n2. Get PR description: !`gh pr view $0`\n3. Analyze changes for:\n   - Correctness\n   - Test coverage\n   - Security issues\n   - Performance impact\n4. Provide summary with approve/request-changes recommendation\n" },
    { id: "skill-refactor", name: "Refactor", description: "Refactor code for better structure", category: "skill", content: "---\nname: refactor\ndescription: Refactor code for better structure and readability\nuser-invocable: true\nargument-hint: \"[file or module]\"\n---\n\nRefactor $ARGUMENTS:\n1. Identify code smells\n2. Extract functions/modules\n3. Simplify complex logic\n4. Improve naming\n5. Ensure tests still pass\n" },

    // Agents
    { id: "agent-bug-fixer", name: "Bug Fixer", description: "Investigate and fix reported bugs", category: "agent", content: "---\nname: bug-fixer\ndescription: Investigate and fix reported bugs\nmodel: opus\neffort: high\ntools: Read, Glob, Grep, Edit, Bash\npermissions: acceptEdits\n---\n\nYou are an expert debugger. When given a bug report:\n1. Reproduce the issue by reading relevant code\n2. Identify the root cause\n3. Implement a minimal fix\n4. Verify the fix doesn't break existing tests\n" },
    { id: "agent-security", name: "Security Auditor", description: "Audit code for security vulnerabilities", category: "agent", content: "---\nname: security-auditor\ndescription: Audit code for security vulnerabilities\nmodel: opus\ntools: Read, Glob, Grep\npermissions: plan\nmemory: project\n---\n\nYou are a security expert. Analyze code for:\n- OWASP Top 10 vulnerabilities\n- Injection attacks (SQL, XSS, command)\n- Authentication/authorization flaws\n- Secrets in code\n- Dependency vulnerabilities\n\nProvide severity ratings and remediation steps.\n" },
    { id: "agent-docs", name: "Docs Generator", description: "Generate comprehensive documentation", category: "agent", content: "---\nname: docs-generator\ndescription: Generate comprehensive documentation\nmodel: sonnet\ntools: Read, Glob, Grep, Write\n---\n\nYou generate documentation. For each file/module:\n1. Read and understand the code\n2. Generate JSDoc/TSDoc comments\n3. Create README sections\n4. Add usage examples\n" },
    { id: "agent-perf", name: "Performance Optimizer", description: "Analyze and optimize code performance", category: "agent", content: "---\nname: performance-optimizer\ndescription: Analyze and optimize code performance\nmodel: opus\ntools: Read, Glob, Grep, Edit, Bash\nmemory: project\n---\n\nYou are a performance expert. When analyzing code:\n1. Profile hot paths\n2. Identify bottlenecks (O(n^2), unnecessary allocations)\n3. Suggest optimizations with benchmarks\n4. Implement changes\n" },

    // Rules
    { id: "rule-ts-strict", name: "TypeScript Strict", description: "Enforce strict TypeScript patterns", category: "rule", paths: ["**/*.ts", "**/*.tsx"], content: "# TypeScript Rules\n\n- Use strict mode, no `any` types\n- Named exports over default exports\n- Prefer interfaces over type aliases for objects\n- Use explicit return types on public functions\n- No unused variables or imports" },
    { id: "rule-api", name: "API Design", description: "REST API conventions and validation", category: "rule", paths: ["src/api/**/*", "src/routes/**/*"], content: "# API Design Rules\n\n- All endpoints must validate input\n- Use standard error format: `{ error: string, code: number }`\n- Return proper HTTP status codes\n- Include pagination for list endpoints\n- Document endpoints with JSDoc" },
    { id: "rule-testing", name: "Testing Standards", description: "Test naming, coverage, and patterns", category: "rule", paths: ["**/*.test.*", "**/*.spec.*"], content: "# Testing Rules\n\n- Co-locate test files with source files\n- Use descriptive names: 'should [behavior] when [condition]'\n- No test should depend on another test's state\n- Mock external services, not internal modules\n- Aim for 80%+ coverage on critical paths" },
    { id: "rule-security", name: "Security", description: "Security best practices for code", category: "rule", paths: [], content: "# Security Rules\n\n- Never commit secrets, API keys, or credentials\n- Sanitize user input before database queries\n- Use parameterized queries, never string concatenation\n- Validate all external data at system boundaries\n- Use HTTPS for all external API calls" },
    { id: "rule-git", name: "Git Conventions", description: "Commit messages and branching", category: "rule", paths: [], content: "# Git Commit Rules\n\n- Use conventional commits: `type(scope): description`\n- Types: feat, fix, refactor, test, docs, chore\n- Keep commits atomic and focused\n- Write imperative mood: 'add feature' not 'added feature'\n- Reference issue numbers when applicable" },
    { id: "rule-errors", name: "Error Handling", description: "Error handling patterns", category: "rule", paths: ["src/**/*"], content: "# Error Handling Rules\n\n- Never swallow errors silently\n- Use custom error classes for domain errors\n- Always include error context\n- Log errors at the point of handling, not catching\n- Return user-friendly messages, log technical details" },
    { id: "rule-a11y", name: "Accessibility", description: "UI accessibility standards", category: "rule", paths: ["**/*.svelte", "**/*.tsx", "**/*.jsx"], content: "# Accessibility Rules\n\n- All images must have alt text\n- Use semantic HTML elements\n- Ensure keyboard navigation works\n- Maintain sufficient color contrast\n- Add aria-labels to icon-only buttons" },
    { id: "rule-perf", name: "Performance", description: "Performance optimization guidelines", category: "rule", paths: [], content: "# Performance Rules\n\n- Avoid N+1 queries — use batch loading\n- Use pagination for large data sets\n- Lazy-load non-critical resources\n- Profile before optimizing\n- Cache expensive computations" },

    // Hooks
    { id: "hook-bash-validator", name: "Bash Validator", description: "Validate bash commands before execution", category: "hook", event: "PreToolUse", matcher: "Bash", hookType: "command", hookValue: ".claude/hooks/validate.sh" },
    { id: "hook-webhook", name: "HTTP Webhook", description: "Send events to an HTTP endpoint", category: "hook", event: "PreToolUse", matcher: "", hookType: "http", hookValue: "http://localhost:8080/hook" },
    { id: "hook-prompt-guard", name: "Prompt Guard", description: "AI validates actions before execution", category: "hook", event: "PreToolUse", matcher: "", hookType: "prompt", hookValue: "Check if this action is safe and appropriate" },
    { id: "hook-log-usage", name: "Log Usage", description: "Log tool usage to a file", category: "hook", event: "PostToolUse", matcher: "", hookType: "command", hookValue: "echo \"$(date): tool used\" >> ~/.claude/usage.log" },
    { id: "hook-cleanup", name: "Session Cleanup", description: "Run cleanup on session end", category: "hook", event: "SessionEnd", matcher: "", hookType: "command", hookValue: "echo 'Session ended'" },

    // MCP
    { id: "mcp-filesystem", name: "Filesystem", description: "Read/write local files via MCP", category: "mcp", mcpType: "stdio", mcpCommand: "npx", mcpArgs: "-y @modelcontextprotocol/server-filesystem /path/to/dir" },
    { id: "mcp-github", name: "GitHub", description: "GitHub API access via MCP", category: "mcp", mcpType: "stdio", mcpCommand: "npx", mcpArgs: "-y @modelcontextprotocol/server-github" },
    { id: "mcp-postgres", name: "PostgreSQL", description: "Query PostgreSQL databases", category: "mcp", mcpType: "stdio", mcpCommand: "npx", mcpArgs: "-y @modelcontextprotocol/server-postgres postgresql://localhost/mydb" },
    { id: "mcp-memory", name: "Memory", description: "Persistent memory store via MCP", category: "mcp", mcpType: "stdio", mcpCommand: "npx", mcpArgs: "-y @modelcontextprotocol/server-memory" },
    { id: "mcp-sqlite", name: "SQLite", description: "Query SQLite databases", category: "mcp", mcpType: "stdio", mcpCommand: "npx", mcpArgs: "-y @modelcontextprotocol/server-sqlite /path/to/db.sqlite" },
  ];

  const CATEGORY_COLORS: Record<string, string> = {
    skill: "bg-warning/10 text-warning",
    agent: "bg-accent/10 text-accent",
    rule: "bg-info/10 text-info",
    hook: "bg-success/10 text-success",
    mcp: "bg-danger/10 text-danger",
  };

  const filtered = $derived(
    TEMPLATES.filter((t) => {
      if (activeCategory !== "all" && t.category !== activeCategory) return false;
      if (search && !t.name.toLowerCase().includes(search.toLowerCase()) && !t.description.toLowerCase().includes(search.toLowerCase())) return false;
      return true;
    }),
  );
</script>

{#if open}
  <!-- Backdrop -->
  <button class="fixed inset-0 bg-black/50 z-50" onclick={onclose} aria-label="Close gallery"></button>

  <!-- Bottom sheet -->
  <div class="fixed bottom-0 left-0 right-0 z-50 bg-bg-secondary border-t border-border rounded-t-2xl shadow-2xl" style="height: 65vh">
    <!-- Handle -->
    <div class="flex justify-center pt-2 pb-1">
      <div class="w-10 h-1 rounded-full bg-border"></div>
    </div>

    <!-- Header -->
    <div class="flex items-center justify-between px-6 pb-3">
      <div>
        <h2 class="text-lg font-semibold text-text-primary">Template Gallery</h2>
        <p class="text-xs text-text-muted">{TEMPLATES.length} templates across {CATEGORIES.length - 1} categories</p>
      </div>
      <button class="p-2 text-text-muted hover:text-text-primary rounded-lg hover:bg-bg-hover" onclick={onclose} aria-label="Close">
        <X size={18} />
      </button>
    </div>

    <!-- Category tabs + Search -->
    <div class="px-6 pb-3 space-y-2">
      <div class="flex gap-1 overflow-x-auto">
        {#each CATEGORIES as cat}
          {@const Icon = cat.icon}
          <button
            class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg shrink-0 transition-colors
              {activeCategory === cat.id ? 'bg-accent text-white' : 'bg-bg-tertiary text-text-muted hover:text-text-secondary'}"
            onclick={() => (activeCategory = cat.id)}
          >
            <Icon size={12} />
            {cat.label}
            <span class="opacity-60">
              ({cat.id === "all" ? TEMPLATES.length : TEMPLATES.filter((t) => t.category === cat.id).length})
            </span>
          </button>
        {/each}
      </div>
      <div class="relative">
        <Search size={14} class="absolute left-3 top-2.5 text-text-muted" />
        <input type="text" class="w-full pl-9 pr-3 py-2 text-sm bg-bg-tertiary border border-border rounded-lg text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent" placeholder="Search templates..." bind:value={search} />
      </div>
    </div>

    <!-- Template grid -->
    <div class="px-6 pb-6 overflow-y-auto" style="height: calc(65vh - 170px)">
      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        {#each filtered as template}
          <button
            class="text-left p-4 bg-bg-tertiary border border-border rounded-lg hover:border-accent/30 transition-colors group"
            onclick={() => { onselect(template); onclose(); }}
          >
            <div class="flex items-start justify-between mb-2">
              <span class="text-[10px] px-1.5 py-0.5 rounded-full {CATEGORY_COLORS[template.category]}">{template.category}</span>
              <span class="text-[10px] text-accent opacity-0 group-hover:opacity-100 transition-opacity">+ Add</span>
            </div>
            <p class="text-sm font-medium text-text-primary">{template.name}</p>
            <p class="text-xs text-text-muted mt-1 line-clamp-2">{template.description}</p>
          </button>
        {/each}
      </div>
      {#if filtered.length === 0}
        <p class="text-sm text-text-muted text-center py-8">No templates match your search</p>
      {/if}
    </div>
  </div>
{/if}
