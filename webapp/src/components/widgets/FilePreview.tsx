import { useState } from "react";
import type { GeneratedFile } from "../../types";

interface Props {
  files: GeneratedFile[];
  onAction: (action: string, payload?: unknown) => void;
}

export function FilePreview({ files, onAction }: Props) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [expandTree, setExpandTree] = useState(true);

  // Build tree structure
  const tree = buildTree(files);
  const selected = files.find((f) => f.path === selectedFile);

  return (
    <div className="card overflow-hidden">
      {/* File count header */}
      <div className="flex items-center justify-between px-3 py-2 bg-gray-800/50 -mx-4 -mt-4 mb-3 border-b border-gray-700/50">
        <span className="text-xs text-gray-400">
          {files.length} files will be created
        </span>
        <button
          onClick={() => setExpandTree(!expandTree)}
          className="text-xs text-dex-400 hover:text-dex-300"
        >
          {expandTree ? "Collapse" : "Expand"}
        </button>
      </div>

      {expandTree && (
        <div className="max-h-[200px] overflow-y-auto mb-3 -mx-1">
          <TreeNode node={tree} selectedFile={selectedFile} onSelect={setSelectedFile} depth={0} />
        </div>
      )}

      {/* File content preview */}
      {selected && (
        <div className="mt-2">
          <div className="flex items-center justify-between px-2 py-1.5 bg-gray-800/50 rounded-t-lg border border-b-0 border-gray-700/50">
            <span className="text-xs text-gray-400 font-mono truncate">{selected.path}</span>
            <button
              onClick={() => setSelectedFile(null)}
              className="text-xs text-gray-500 hover:text-gray-300"
            >
              Close
            </button>
          </div>
          <pre className="text-xs font-mono bg-gray-900/80 border border-gray-700/50 rounded-b-lg p-3 overflow-x-auto max-h-[300px] overflow-y-auto text-gray-300">
            {selected.content}
          </pre>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex gap-2 mt-4">
        <button
          onClick={() => onAction("confirm-push")}
          className="btn-primary flex-1"
        >
          Push to Repository
        </button>
        <button
          onClick={() => onAction("back-to-variables")}
          className="btn-secondary"
        >
          Back
        </button>
      </div>
    </div>
  );
}

interface TreeItem {
  name: string;
  path?: string;
  children: Record<string, TreeItem>;
}

function buildTree(files: GeneratedFile[]): TreeItem {
  const root: TreeItem = { name: "", children: {} };
  for (const file of files) {
    const parts = file.path.split("/");
    let current = root;
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]!;
      if (!current.children[part]) {
        current.children[part] = {
          name: part,
          path: i === parts.length - 1 ? file.path : undefined,
          children: {},
        };
      }
      current = current.children[part]!;
    }
  }
  return root;
}

function TreeNode({
  node,
  selectedFile,
  onSelect,
  depth,
}: {
  node: TreeItem;
  selectedFile: string | null;
  onSelect: (path: string) => void;
  depth: number;
}) {
  const entries = Object.values(node.children).sort((a, b) => {
    // Directories first
    const aIsDir = Object.keys(a.children).length > 0;
    const bIsDir = Object.keys(b.children).length > 0;
    if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
    return a.name.localeCompare(b.name);
  });

  return (
    <div>
      {entries.map((entry) => {
        const isDir = Object.keys(entry.children).length > 0;
        const isSelected = entry.path === selectedFile;

        return (
          <div key={entry.name}>
            <button
              onClick={() => entry.path && onSelect(entry.path)}
              className={`flex items-center gap-1.5 w-full px-2 py-0.5 text-left text-xs rounded hover:bg-gray-800/50 ${
                isSelected ? "bg-dex-900/30 text-dex-300" : "text-gray-400"
              }`}
              style={{ paddingLeft: `${depth * 16 + 8}px` }}
              disabled={isDir}
            >
              {isDir ? (
                <svg className="w-3.5 h-3.5 text-dex-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              ) : (
                <svg className="w-3.5 h-3.5 text-gray-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
              )}
              <span className="font-mono truncate">{entry.name}</span>
            </button>
            {isDir && (
              <TreeNode node={entry} selectedFile={selectedFile} onSelect={onSelect} depth={depth + 1} />
            )}
          </div>
        );
      })}
    </div>
  );
}
