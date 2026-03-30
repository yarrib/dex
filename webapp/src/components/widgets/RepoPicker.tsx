import { useState, useEffect } from "react";
import type { GitHubRepo } from "../../types";
import * as api from "../../lib/api";

interface Props {
  onAction: (action: string, payload?: unknown) => void;
}

export function RepoPicker({ onAction }: Props) {
  const [repos, setRepos] = useState<GitHubRepo[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");

  useEffect(() => {
    const token = sessionStorage.getItem("gh_token");
    if (!token) return;
    api
      .listRepos(token)
      .then(setRepos)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const filtered = repos.filter(
    (r) =>
      r.name.toLowerCase().includes(search.toLowerCase()) ||
      r.fullName.toLowerCase().includes(search.toLowerCase()),
  );

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-400 py-4">
        <div className="w-4 h-4 border-2 border-dex-500 border-t-transparent rounded-full animate-spin" />
        Loading your repositories...
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <input
        type="text"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        placeholder="Search repositories..."
        className="input-field text-sm"
      />

      <div className="max-h-[240px] overflow-y-auto space-y-1">
        {filtered.map((r) => (
          <button
            key={r.fullName}
            onClick={() => onAction("select-repo", r)}
            className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-800 transition-colors flex items-center justify-between"
          >
            <div className="min-w-0">
              <div className="text-sm text-gray-200 truncate">{r.fullName}</div>
              <div className="text-[10px] text-gray-500">
                {r.isPrivate ? "Private" : "Public"} · {r.defaultBranch}
              </div>
            </div>
            <svg className="w-4 h-4 text-gray-600 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>
        ))}
      </div>

      <div className="border-t border-gray-800 pt-3">
        <button
          onClick={() => onAction("create-new-repo")}
          className="btn-secondary w-full flex items-center justify-center gap-2"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          Create new repository
        </button>
      </div>
    </div>
  );
}
