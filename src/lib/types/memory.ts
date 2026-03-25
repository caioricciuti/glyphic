export interface MemoryFile {
  filename: string;
  path: string;
  content: string;
  name: string | null;
  description: string | null;
  memory_type: string | null;
}

export interface ProjectInfo {
  hash: string;
  path: string;
  has_memory: boolean;
  exists: boolean;
}
