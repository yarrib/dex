import { Router } from "express";
import type { Request, Response, NextFunction } from "express";

export const githubRouter = Router();

/** Extract Bearer token from Authorization header. */
function getToken(req: Request, res: Response, next: NextFunction): void {
  const auth = req.headers.authorization;
  if (!auth?.startsWith("Bearer ")) {
    res.status(401).json({ error: "Missing or invalid Authorization header" });
    return;
  }
  (req as Request & { ghToken: string }).ghToken = auth.slice(7);
  next();
}

githubRouter.use(getToken);

/** Helper to call GitHub API. */
async function ghApi(path: string, token: string, opts?: RequestInit) {
  const res = await fetch(`https://api.github.com${path}`, {
    headers: {
      Authorization: `Bearer ${token}`,
      Accept: "application/vnd.github+json",
      "X-GitHub-Api-Version": "2022-11-28",
      ...opts?.headers,
    },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`GitHub API ${res.status}: ${body}`);
  }
  return res.json();
}

interface RepoApiItem {
  owner: { login: string };
  name: string;
  full_name: string;
  default_branch: string;
  private: boolean;
  html_url: string;
}

function mapRepo(r: RepoApiItem) {
  return {
    owner: r.owner.login,
    name: r.name,
    fullName: r.full_name,
    defaultBranch: r.default_branch,
    isPrivate: r.private,
    url: r.html_url,
  };
}

/**
 * GET /api/github/repos
 * List repos for the authenticated user.
 */
githubRouter.get("/repos", async (req, res) => {
  try {
    const token = (req as Request & { ghToken: string }).ghToken;
    const repos = (await ghApi(
      "/user/repos?sort=updated&per_page=30&type=all",
      token,
    )) as RepoApiItem[];
    res.json(repos.map(mapRepo));
  } catch (err) {
    res.status(500).json({ error: `${err}` });
  }
});

/**
 * POST /api/github/repos
 * Create a new repo.
 */
githubRouter.post("/repos", async (req, res) => {
  try {
    const token = (req as Request & { ghToken: string }).ghToken;
    const { name, description } = req.body as {
      name: string;
      private: boolean;
      description?: string;
    };
    const isPrivate = (req.body as { private: boolean }).private;
    const repo = (await ghApi("/user/repos", token, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        name,
        description: description || "",
        private: isPrivate,
        auto_init: true,
      }),
    })) as RepoApiItem;
    res.json(mapRepo(repo));
  } catch (err) {
    res.status(500).json({ error: `${err}` });
  }
});

interface TreeItem {
  path: string;
  mode: string;
  type: string;
  sha?: string;
  content?: string;
}

/**
 * POST /api/github/push
 * Push scaffolded files to a branch using the Git Data API (tree + commit).
 */
githubRouter.post("/push", async (req, res) => {
  try {
    const token = (req as Request & { ghToken: string }).ghToken;
    const { repo, branch, files, message } = req.body as {
      repo: string;
      branch: string;
      files: { path: string; content: string }[];
      message: string;
    };

    // 1. Get the default branch's latest commit SHA
    const refData = (await ghApi(`/repos/${repo}/git/ref/heads/${branch}`, token).catch(
      () => null,
    )) as { object: { sha: string } } | null;

    let baseSha: string;
    let baseTreeSha: string;

    if (refData) {
      // Branch exists — get its tree
      baseSha = refData.object.sha;
      const commit = (await ghApi(`/repos/${repo}/git/commits/${baseSha}`, token)) as {
        tree: { sha: string };
      };
      baseTreeSha = commit.tree.sha;
    } else {
      // Branch doesn't exist — get default branch and create from it
      const repoInfo = (await ghApi(`/repos/${repo}`, token)) as {
        default_branch: string;
      };
      const defaultRef = (await ghApi(
        `/repos/${repo}/git/ref/heads/${repoInfo.default_branch}`,
        token,
      )) as { object: { sha: string } };
      baseSha = defaultRef.object.sha;
      const commit = (await ghApi(`/repos/${repo}/git/commits/${baseSha}`, token)) as {
        tree: { sha: string };
      };
      baseTreeSha = commit.tree.sha;
    }

    // 2. Create tree with all files
    const tree: TreeItem[] = files.map((f) => ({
      path: f.path,
      mode: "100644",
      type: "blob",
      content: f.content,
    }));

    const newTree = (await ghApi(`/repos/${repo}/git/trees`, token, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ base_tree: baseTreeSha, tree }),
    })) as { sha: string };

    // 3. Create commit
    const newCommit = (await ghApi(`/repos/${repo}/git/commits`, token, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        message,
        tree: newTree.sha,
        parents: [baseSha],
      }),
    })) as { sha: string };

    // 4. Create or update branch ref
    if (refData) {
      await ghApi(`/repos/${repo}/git/refs/heads/${branch}`, token, {
        method: "PATCH",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ sha: newCommit.sha }),
      });
    } else {
      await ghApi(`/repos/${repo}/git/refs`, token, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ ref: `refs/heads/${branch}`, sha: newCommit.sha }),
      });
    }

    res.json({ sha: newCommit.sha });
  } catch (err) {
    res.status(500).json({ error: `${err}` });
  }
});

/**
 * POST /api/github/pr
 * Create a pull request.
 */
githubRouter.post("/pr", async (req, res) => {
  try {
    const token = (req as Request & { ghToken: string }).ghToken;
    const { repo, head, base, title, body } = req.body as {
      repo: string;
      head: string;
      base: string;
      title: string;
      body: string;
    };
    const pr = (await ghApi(`/repos/${repo}/pulls`, token, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title, body, head, base }),
    })) as { html_url: string; number: number };
    res.json({ url: pr.html_url, number: pr.number });
  } catch (err) {
    res.status(500).json({ error: `${err}` });
  }
});
