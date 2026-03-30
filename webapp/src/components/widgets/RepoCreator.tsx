import { useState } from "react";
import * as api from "../../lib/api";

interface Props {
  onAction: (action: string, payload?: unknown) => void;
}

export function RepoCreator({ onAction }: Props) {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [isPrivate, setIsPrivate] = useState(true);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleCreate() {
    if (!name.trim()) {
      setError("Repository name is required");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const token = sessionStorage.getItem("gh_token");
      if (!token) throw new Error("Not authenticated");
      const repo = await api.createRepo(token, name, isPrivate, description);
      onAction("select-repo", repo);
    } catch (err) {
      setError(`Failed to create repository: ${err}`);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-3">
      <div>
        <label className="block text-xs font-medium text-gray-400 mb-1">
          Repository name <span className="text-red-400">*</span>
        </label>
        <input
          type="text"
          value={name}
          onChange={(e) => { setName(e.target.value); setError(""); }}
          placeholder="my-project"
          className="input-field text-sm"
        />
      </div>

      <div>
        <label className="block text-xs font-medium text-gray-400 mb-1">
          Description
        </label>
        <input
          type="text"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="A brief description..."
          className="input-field text-sm"
        />
      </div>

      <div>
        <label className="block text-xs font-medium text-gray-400 mb-1">Visibility</label>
        <div className="flex gap-2">
          <button
            onClick={() => setIsPrivate(true)}
            className={`flex-1 px-3 py-2 rounded-lg text-sm border transition-colors ${
              isPrivate
                ? "bg-dex-600/20 border-dex-500/50 text-dex-300"
                : "bg-gray-900 border-gray-700 text-gray-400"
            }`}
          >
            Private
          </button>
          <button
            onClick={() => setIsPrivate(false)}
            className={`flex-1 px-3 py-2 rounded-lg text-sm border transition-colors ${
              !isPrivate
                ? "bg-dex-600/20 border-dex-500/50 text-dex-300"
                : "bg-gray-900 border-gray-700 text-gray-400"
            }`}
          >
            Public
          </button>
        </div>
      </div>

      {error && <p className="text-xs text-red-400">{error}</p>}

      <div className="flex gap-2">
        <button onClick={handleCreate} disabled={loading} className="btn-primary flex-1">
          {loading ? "Creating..." : "Create Repository"}
        </button>
        <button onClick={() => onAction("back-to-repo-picker")} className="btn-secondary">
          Back
        </button>
      </div>
    </div>
  );
}
