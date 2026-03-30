import { useState, useRef, useEffect } from "react";

interface Props {
  onSend: (message: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export function ChatInput({ onSend, placeholder = "Type a message...", disabled }: Props) {
  const [value, setValue] = useState("");
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!disabled) inputRef.current?.focus();
  }, [disabled]);

  function handleSubmit() {
    const trimmed = value.trim();
    if (!trimmed || disabled) return;
    onSend(trimmed);
    setValue("");
    // Reset height
    if (inputRef.current) inputRef.current.style.height = "auto";
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleInput() {
    const el = inputRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 120)}px`;
  }

  return (
    <div className="flex items-end gap-2 p-3 sm:p-4 bg-gray-900/80 backdrop-blur-lg border-t border-gray-800 safe-bottom">
      <textarea
        ref={inputRef}
        value={value}
        onChange={(e) => { setValue(e.target.value); handleInput(); }}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        rows={1}
        className="input-field resize-none min-h-[44px] max-h-[120px] text-base"
      />
      <button
        onClick={handleSubmit}
        disabled={disabled || !value.trim()}
        className="btn-primary flex-shrink-0 h-[44px] w-[44px] flex items-center justify-center rounded-xl"
        aria-label="Send"
      >
        <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M12 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  );
}
