export type Theme = "dark" | "light";

let theme = $state<Theme>((typeof localStorage !== "undefined" && localStorage.getItem("glyphic-theme") as Theme) || "dark");

export function getTheme(): Theme {
  return theme;
}

export function toggleTheme() {
  theme = theme === "dark" ? "light" : "dark";
  applyTheme();
}

export function setTheme(t: Theme) {
  theme = t;
  applyTheme();
}

function applyTheme() {
  if (typeof document !== "undefined") {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("glyphic-theme", theme);
  }
}

// Apply on load
if (typeof document !== "undefined") {
  applyTheme();
}
