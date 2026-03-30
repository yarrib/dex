import type { FlowStep } from "../types";

const STEPS: { key: FlowStep; label: string }[] = [
  { key: "auth", label: "Connect" },
  { key: "repo", label: "Repo" },
  { key: "template", label: "Template" },
  { key: "variables", label: "Configure" },
  { key: "preview", label: "Preview" },
  { key: "push", label: "Push" },
];

interface Props {
  currentStep: FlowStep;
}

const STEP_ORDER: FlowStep[] = ["welcome", "auth", "repo", "template", "variables", "preview", "push", "done"];

function stepIndex(step: FlowStep): number {
  return STEP_ORDER.indexOf(step);
}

export function StepIndicator({ currentStep }: Props) {
  const currentIdx = stepIndex(currentStep);

  return (
    <div className="flex items-center gap-1 sm:gap-2 px-4 py-2 overflow-x-auto">
      {STEPS.map((step, i) => {
        const idx = stepIndex(step.key);
        const isActive = step.key === currentStep;
        const isComplete = currentIdx > idx;
        const isPending = currentIdx < idx;

        return (
          <div key={step.key} className="flex items-center gap-1 sm:gap-2 flex-shrink-0">
            {i > 0 && (
              <div
                className={`w-4 sm:w-8 h-px ${isComplete ? "bg-dex-500" : "bg-gray-700"}`}
              />
            )}
            <div className="flex items-center gap-1.5">
              <div
                className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-medium transition-colors ${
                  isActive
                    ? "bg-dex-600 text-white ring-2 ring-dex-400/30"
                    : isComplete
                      ? "bg-dex-600/30 text-dex-400"
                      : "bg-gray-800 text-gray-500"
                }`}
              >
                {isComplete ? (
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
                  </svg>
                ) : (
                  i + 1
                )}
              </div>
              <span
                className={`text-xs font-medium hidden sm:block ${
                  isActive ? "text-gray-200" : isPending ? "text-gray-600" : "text-gray-400"
                }`}
              >
                {step.label}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
