<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorView, keymap, placeholder as cmPlaceholder } from "@codemirror/view";
  import { EditorState } from "@codemirror/state";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { json } from "@codemirror/lang-json";
  import { javascript } from "@codemirror/lang-javascript";
  import { oneDark } from "@codemirror/theme-one-dark";
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from "@codemirror/language";

  interface Props {
    value: string;
    language?: "json" | "shell" | "javascript" | "text";
    placeholder?: string;
    minHeight?: string;
    maxHeight?: string;
    onchange?: (value: string) => void;
  }

  const {
    value = "",
    language = "text",
    placeholder = "",
    minHeight = "80px",
    maxHeight = "300px",
    onchange,
  }: Props = $props();

  let container: HTMLDivElement;
  let view: EditorView | undefined;
  let ignoreNextUpdate = false;

  function getLangExtension() {
    switch (language) {
      case "json": return [json()];
      case "shell": return [javascript()]; // decent enough for shell
      case "javascript": return [javascript()];
      default: return [];
    }
  }

  onMount(() => {
    const extensions = [
      keymap.of([...defaultKeymap, ...historyKeymap]),
      history(),
      bracketMatching(),
      syntaxHighlighting(defaultHighlightStyle),
      oneDark,
      ...getLangExtension(),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          ignoreNextUpdate = true;
          onchange?.(update.state.doc.toString());
        }
      }),
      EditorView.theme({
        "&": {
          fontSize: "12px",
          minHeight,
          maxHeight,
          borderRadius: "8px",
          border: "1px solid var(--color-border)",
        },
        ".cm-scroller": {
          overflow: "auto",
          fontFamily: "ui-monospace, SFMono-Regular, Menlo, monospace",
        },
        ".cm-content": {
          padding: "8px 0",
          minHeight,
        },
        "&.cm-focused": {
          outline: "none",
          borderColor: "var(--color-accent)",
        },
        ".cm-gutters": {
          backgroundColor: "transparent",
          border: "none",
        },
      }),
      EditorView.lineWrapping,
    ];

    if (placeholder) {
      extensions.push(cmPlaceholder(placeholder));
    }

    view = new EditorView({
      state: EditorState.create({ doc: value, extensions }),
      parent: container,
    });
  });

  // Sync external value changes into the editor
  $effect(() => {
    if (view && !ignoreNextUpdate) {
      const current = view.state.doc.toString();
      if (value !== current) {
        view.dispatch({
          changes: { from: 0, to: current.length, insert: value },
        });
      }
    }
    ignoreNextUpdate = false;
  });

  onDestroy(() => {
    view?.destroy();
  });
</script>

<div class="nodrag nowheel code-editor-wrapper" bind:this={container}></div>

<style>
  .code-editor-wrapper {
    width: 100%;
  }
  .code-editor-wrapper :global(.cm-editor) {
    background: var(--color-bg-tertiary);
  }
</style>
