import { marked } from "marked";
import DOMPurify from "dompurify";

/**
 * Render markdown to HTML that is safe to inject via `{@html}`.
 *
 * Everything rendered this way may be untrusted — session transcripts, tool
 * outputs from arbitrary repos/web, `@import`ed files, marketplace skills — so
 * the generated HTML is sanitized before it ever reaches the webview DOM.
 */
export function renderMarkdown(md: string): string {
  const html = marked.parse(md ?? "", { async: false }) as string;
  return DOMPurify.sanitize(html);
}
