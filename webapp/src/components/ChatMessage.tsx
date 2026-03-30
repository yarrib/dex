import type { ChatMessage as ChatMessageType } from "../types";
import { WidgetRenderer } from "./widgets/WidgetRenderer";

interface Props {
  message: ChatMessageType;
  onAction: (action: string, payload?: unknown) => void;
}

export function ChatMessage({ message, onAction }: Props) {
  if (message.role === "system") {
    return (
      <div className="flex justify-center my-3">
        <span className="text-xs text-gray-500 bg-gray-900 px-3 py-1 rounded-full">
          {message.content}
        </span>
      </div>
    );
  }

  const isAssistant = message.role === "assistant";

  return (
    <div className={`flex ${isAssistant ? "justify-start" : "justify-end"} mb-4`}>
      <div className={`max-w-[90%] sm:max-w-[80%] ${isAssistant ? "mr-8" : "ml-8"}`}>
        {isAssistant && (
          <div className="flex items-center gap-2 mb-1.5">
            <div className="w-6 h-6 rounded-full bg-dex-600 flex items-center justify-center flex-shrink-0">
              <span className="text-xs font-bold text-white">d</span>
            </div>
            <span className="text-xs text-gray-500 font-medium">dex</span>
          </div>
        )}

        <div className={isAssistant ? "chat-bubble-assistant" : "chat-bubble-user"}>
          <div className="text-sm leading-relaxed whitespace-pre-wrap">{message.content}</div>
        </div>

        {message.widget && (
          <div className="mt-3">
            <WidgetRenderer widget={message.widget} onAction={onAction} />
          </div>
        )}
      </div>
    </div>
  );
}
