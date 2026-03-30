interface Props {
  url: string;
}

export function PRLink({ url }: Props) {
  return (
    <a
      href={url}
      target="_blank"
      rel="noopener noreferrer"
      className="card flex items-center gap-3 hover:border-green-500/50 transition-colors group"
    >
      <div className="w-10 h-10 rounded-lg bg-green-900/30 border border-green-700/30 flex items-center justify-center">
        <svg className="w-5 h-5 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        </svg>
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-sm font-medium text-green-400">Pull request created!</div>
        <div className="text-xs text-gray-500 truncate group-hover:text-gray-400">{url}</div>
      </div>
      <svg className="w-4 h-4 text-gray-600 group-hover:text-gray-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
      </svg>
    </a>
  );
}
