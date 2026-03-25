import { api } from "$lib/tauri/commands";
import type { ProjectInfo } from "$lib/types";

let selectedProjectPath = $state<string | null>(null);
let projects = $state<ProjectInfo[]>([]);
let loaded = $state(false);

export function getSelectedProjectPath(): string | null {
  return selectedProjectPath;
}

export function getSelectedProjectName(): string {
  if (!selectedProjectPath) return "";
  return selectedProjectPath.split("/").pop() ?? selectedProjectPath;
}

export function getProjects(): ProjectInfo[] {
  return projects;
}

export function isLoaded(): boolean {
  return loaded;
}

export function selectProject(path: string) {
  selectedProjectPath = path;
}

export function clearProject() {
  selectedProjectPath = null;
}

export async function loadProjects() {
  try {
    projects = await api.projects.list();
    loaded = true;
  } catch (e) {
    console.error("Failed to load projects:", e);
    projects = [];
    loaded = true;
  }
}
