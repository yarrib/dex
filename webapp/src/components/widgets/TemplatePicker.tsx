import type { TemplateMeta } from "../../types";

interface Props {
  templates: TemplateMeta[];
  onAction: (action: string, payload?: unknown) => void;
}

const TEMPLATE_ICONS: Record<string, string> = {
  default: "P",
  "dabs-ml": "ML",
  "dabs-etl": "E",
  "dabs-package": "Pk",
  "dabs-aiagent": "AI",
};

export function TemplatePicker({ templates, onAction }: Props) {
  return (
    <div className="grid gap-2">
      {templates.map((t) => (
        <button
          key={t.name}
          onClick={() => onAction("select-template", t.name)}
          className="card hover:border-dex-500/50 hover:bg-gray-800/60 transition-all
                     text-left flex items-start gap-3 active:scale-[0.98]"
        >
          <div className="w-10 h-10 rounded-lg bg-dex-900/50 border border-dex-700/30 flex items-center justify-center flex-shrink-0">
            <span className="text-xs font-bold text-dex-400">
              {TEMPLATE_ICONS[t.name] || t.name.charAt(0).toUpperCase()}
            </span>
          </div>
          <div className="min-w-0">
            <div className="text-sm font-medium text-gray-200">{t.name}</div>
            <div className="text-xs text-gray-500 mt-0.5 line-clamp-2">{t.description}</div>
            <div className="text-[10px] text-gray-600 mt-1">v{t.version}</div>
          </div>
        </button>
      ))}
    </div>
  );
}
