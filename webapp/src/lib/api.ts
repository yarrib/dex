import type { Template, TemplateMeta, GeneratedFile, GitHubUser, GitHubRepo } from "../types";

const BASE = "/api";

async function request<T>(path: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    headers: { "Content-Type": "application/json", ...opts?.headers },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`API ${res.status}: ${body || res.statusText}`);
  }
  return res.json() as Promise<T>;
}

// ── Auth ────────────────────────────────────────────────────────────────

/** Get the GitHub OAuth authorization URL. */
export function getAuthUrl(): Promise<{ url: string }> {
  return request("/auth/github/url");
}

/** Exchange an OAuth code for a token and return the user. */
export function exchangeCode(code: string): Promise<{ token: string; user: GitHubUser }> {
  return request("/auth/github/callback", {
    method: "POST",
    body: JSON.stringify({ code }),
  });
}

// ── GitHub ──────────────────────────────────────────────────────────────

/** List the authenticated user's repos. */
export function listRepos(token: string): Promise<GitHubRepo[]> {
  return request("/github/repos", {
    headers: { Authorization: `Bearer ${token}` },
  });
}

/** Create a new GitHub repo. */
export function createRepo(
  token: string,
  name: string,
  isPrivate: boolean,
  description?: string,
): Promise<GitHubRepo> {
  return request("/github/repos", {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ name, private: isPrivate, description }),
  });
}

/** Push scaffolded files to a GitHub repo. */
export function pushFiles(
  token: string,
  repo: string,
  branch: string,
  files: GeneratedFile[],
  message: string,
): Promise<{ sha: string }> {
  return request("/github/push", {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ repo, branch, files, message }),
  });
}

/** Create a pull request. */
export function createPR(
  token: string,
  repo: string,
  head: string,
  base: string,
  title: string,
  body: string,
): Promise<{ url: string; number: number }> {
  return request("/github/pr", {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify({ repo, head, base, title, body }),
  });
}

// ── Templates ───────────────────────────────────────────────────────────

/** List available templates. */
export function listTemplates(): Promise<TemplateMeta[]> {
  return request("/templates");
}

/** Get a template's full definition (with variables). */
export function getTemplate(name: string): Promise<Template> {
  return request(`/templates/${encodeURIComponent(name)}`);
}

/** Render a template with the given variables. Returns generated files. */
export function renderTemplate(
  name: string,
  variables: Record<string, string | boolean | string[]>,
): Promise<GeneratedFile[]> {
  return request(`/templates/${encodeURIComponent(name)}/render`, {
    method: "POST",
    body: JSON.stringify({ variables }),
  });
}
