import { useState } from "react";
import type { GitHubRepo } from "../../types";

interface Props {
  repo: GitHubRepo;
  branch: string;
  fileCount: number;
  onAction: (action: string, payload?: unknown) => void;
}

export function PushConfirm({ repo, branch, fileCount, onAction }: Props) {
  const [pushing, setPushing] = useState(false);

  function handlePush() {
    setPushing(true);
    onAction("execute-push");
  }

  return (
    <div className="card">
      <div className="flex items-center gap-3 mb-3">
        <div className="w-10 h-10 rounded-lg bg-green-900/30 border border-green-700/30 flex items-center justify-center">
          <svg className="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
          </svg>
        </div>
        <div>
          <div className="text-sm font-medium text-gray-200">Ready to push</div>
          <div className="text-xs text-gray-500">{fileCount} files to {repo.fullName}</div>
        </div>
      </div>

      <div className="bg-gray-800/50 rounded-lg px-3 py-2 space-y-1 text-xs mb-4">
        <div className="flex justify-between">
          <span className="text-gray-500">Repository</span>
          <span className="text-gray-300 font-mono">{repo.fullName}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-500">Branch</span>
          <span className="text-gray-300 font-mono">{branch}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-500">Files</span>
          <span className="text-gray-300">{fileCount}</span>
        </div>
      </div>

      <div className="flex gap-2">
        <button onClick={handlePush} disabled={pushing} className="btn-primary flex-1">
          {pushing ? (
            <span className="flex items-center justify-center gap-2">
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              Pushing...
            </span>
          ) : (
            "Push & Create PR"
          )}
        </button>
        <button
          onClick={() => onAction("back-to-preview")}
          disabled={pushing}
          className="btn-secondary"
        >
          Back
        </button>
      </div>
    </div>
  );
}
