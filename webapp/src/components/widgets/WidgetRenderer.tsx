import type { MessageWidget } from "../../types";
import { AuthButton } from "./AuthButton";
import { TemplatePicker } from "./TemplatePicker";
import { VariableForm } from "./VariableForm";
import { FilePreview } from "./FilePreview";
import { RepoPicker } from "./RepoPicker";
import { RepoCreator } from "./RepoCreator";
import { PushConfirm } from "./PushConfirm";
import { PRLink } from "./PRLink";
import { ChoiceButtons } from "./ChoiceButtons";

interface Props {
  widget: MessageWidget;
  onAction: (action: string, payload?: unknown) => void;
}

export function WidgetRenderer({ widget, onAction }: Props) {
  switch (widget.type) {
    case "auth-button":
      return <AuthButton onAction={onAction} />;
    case "repo-picker":
      return <RepoPicker onAction={onAction} />;
    case "repo-creator":
      return <RepoCreator onAction={onAction} />;
    case "template-picker":
      return <TemplatePicker templates={widget.templates} onAction={onAction} />;
    case "variable-form":
      return <VariableForm variables={widget.variables} onAction={onAction} />;
    case "file-preview":
      return <FilePreview files={widget.files} onAction={onAction} />;
    case "push-confirm":
      return (
        <PushConfirm
          repo={widget.repo}
          branch={widget.branch}
          fileCount={widget.fileCount}
          onAction={onAction}
        />
      );
    case "pr-link":
      return <PRLink url={widget.url} />;
    case "choice-buttons":
      return <ChoiceButtons choices={widget.choices} onAction={onAction} />;
  }
}
