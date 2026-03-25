export interface InstructionFile {
  path: string;
  content: string;
  exists: boolean;
  imports: string[];
}
