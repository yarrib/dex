import type { GitHubUser, FlowStep } from "../types";
import { StepIndicator } from "./StepIndicator";

interface Props {
  user: GitHubUser | null;
  currentStep: FlowStep;
}

export function Header({ user, currentStep }: Props) {
  return (
    <header className="border-b border-gray-800 bg-gray-950/80 backdrop-blur-lg sticky top-0 z-50">
      <div className="flex items-center justify-between px-4 py-3">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-dex-600 flex items-center justify-center">
            <span className="text-sm font-bold text-white">d</span>
          </div>
          <div>
            <h1 className="text-sm font-semibold text-gray-100">dex scaffold</h1>
            <p className="text-[10px] text-gray-500 leading-tight">Project setup assistant</p>
          </div>
        </div>

        {user && (
          <div className="flex items-center gap-2">
            <img
              src={user.avatarUrl}
              alt={user.login}
              className="w-7 h-7 rounded-full border border-gray-700"
            />
            <span className="text-xs text-gray-400 hidden sm:block">{user.login}</span>
          </div>
        )}
      </div>

      {currentStep !== "welcome" && <StepIndicator currentStep={currentStep} />}
    </header>
  );
}
