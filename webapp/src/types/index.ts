/** Template variable specification — mirrors dex-core's VariableSpec. */
export interface VariableSpec {
  name: string;
  prompt: string;
  type: "string" | "bool" | "choice" | "multi";
  required: boolean;
  default?: string | boolean | string[];
  choices?: string[];
  validate?: string;
}

/** Template metadata — mirrors dex-core's TemplateMeta. */
export interface TemplateMeta {
  name: string;
  description: string;
  version: string;
}

/** A full template definition with variables and file rules. */
export interface Template {
  meta: TemplateMeta;
  variables: VariableSpec[];
}

/** A generated file from scaffolding. */
export interface GeneratedFile {
  path: string;
  content: string;
  isNew: boolean;
}

/** GitHub repository reference. */
export interface GitHubRepo {
  owner: string;
  name: string;
  fullName: string;
  defaultBranch: string;
  isPrivate: boolean;
  url: string;
}

/** GitHub user info. */
export interface GitHubUser {
  login: string;
  name: string | null;
  avatarUrl: string;
}

/** Conversational flow step identifiers. */
export type FlowStep =
  | "welcome"
  | "auth"
  | "repo"
  | "template"
  | "variables"
  | "preview"
  | "push"
  | "done";

/** A single message in the chat. */
export interface ChatMessage {
  id: string;
  role: "assistant" | "user" | "system";
  content: string;
  timestamp: number;
  /** Optional UI widget to render inline. */
  widget?: MessageWidget;
}

/** Interactive widgets embedded in assistant messages. */
export type MessageWidget =
  | { type: "auth-button" }
  | { type: "repo-picker" }
  | { type: "repo-creator" }
  | { type: "template-picker"; templates: TemplateMeta[] }
  | { type: "variable-form"; variables: VariableSpec[] }
  | { type: "file-preview"; files: GeneratedFile[] }
  | { type: "push-confirm"; repo: GitHubRepo; branch: string; fileCount: number }
  | { type: "pr-link"; url: string }
  | { type: "choice-buttons"; choices: { label: string; value: string }[] };
