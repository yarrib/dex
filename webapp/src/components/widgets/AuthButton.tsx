import { useState, useEffect, useRef } from "react";

interface Props {
  onAction: (action: string, payload?: unknown) => void;
}

interface DeviceCodeResponse {
  deviceCode: string;
  userCode: string;
  verificationUri: string;
  expiresIn: number;
  interval: number;
}

export function AuthButton({ onAction }: Props) {
  const [phase, setPhase] = useState<"idle" | "polling" | "error">("idle");
  const [deviceInfo, setDeviceInfo] = useState<DeviceCodeResponse | null>(null);
  const [copied, setCopied] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);

  useEffect(() => {
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, []);

  async function startDeviceFlow() {
    setPhase("polling");
    try {
      const res = await fetch("/api/auth/github/device", { method: "POST" });
      if (!res.ok) throw new Error("Failed to start device flow");
      const data = (await res.json()) as DeviceCodeResponse;
      setDeviceInfo(data);

      // Start polling for the token
      let elapsed = 0;
      pollRef.current = setInterval(async () => {
        elapsed += data.interval;
        if (elapsed > data.expiresIn) {
          clearInterval(pollRef.current);
          setPhase("error");
          return;
        }

        try {
          const pollRes = await fetch("/api/auth/github/device/poll", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ deviceCode: data.deviceCode }),
          });
          const pollData = (await pollRes.json()) as {
            status: string;
            token?: string;
            user?: { login: string; name: string | null; avatarUrl: string };
          };

          if (pollData.status === "complete" && pollData.token) {
            clearInterval(pollRef.current);
            onAction("auth-success", { token: pollData.token, user: pollData.user });
          }
          // "authorization_pending" and "slow_down" — keep polling
        } catch {
          // Network error — keep polling
        }
      }, data.interval * 1000);
    } catch {
      setPhase("error");
    }
  }

  async function copyCode() {
    if (!deviceInfo) return;
    try {
      await navigator.clipboard.writeText(deviceInfo.userCode);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback for mobile
    }
  }

  if (phase === "idle" || phase === "error") {
    return (
      <div>
        {phase === "error" && (
          <p className="text-xs text-red-400 mb-2">Authentication timed out. Try again.</p>
        )}
        <button
          onClick={startDeviceFlow}
          className="btn-secondary flex items-center gap-2 w-full sm:w-auto"
        >
          <svg className="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
          </svg>
          Sign in with GitHub
        </button>
      </div>
    );
  }

  // Polling phase — show the user code inline
  return (
    <div className="card space-y-4">
      <div className="text-center">
        <p className="text-sm text-gray-300 mb-3">
          Enter this code on GitHub to connect your account:
        </p>

        {/* The verification code — big and tappable */}
        <button
          onClick={copyCode}
          className="inline-flex items-center gap-2 bg-gray-800 border-2 border-dex-500/50
                     rounded-xl px-6 py-3 hover:border-dex-400 transition-colors group"
        >
          <span className="text-2xl font-mono font-bold tracking-widest text-white">
            {deviceInfo?.userCode}
          </span>
          <svg
            className={`w-5 h-5 transition-colors ${copied ? "text-green-400" : "text-gray-500 group-hover:text-gray-300"}`}
            fill="none" viewBox="0 0 24 24" stroke="currentColor"
          >
            {copied ? (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            ) : (
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            )}
          </svg>
        </button>
        <p className="text-xs text-gray-500 mt-2">
          {copied ? "Copied!" : "Tap to copy"}
        </p>
      </div>

      {/* Open GitHub link */}
      <a
        href={deviceInfo?.verificationUri}
        target="_blank"
        rel="noopener noreferrer"
        className="btn-primary w-full flex items-center justify-center gap-2"
      >
        Open GitHub
        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
        </svg>
      </a>

      <div className="flex items-center justify-center gap-2 text-xs text-gray-500">
        <div className="w-3 h-3 border-2 border-dex-500 border-t-transparent rounded-full animate-spin" />
        Waiting for authorization...
      </div>
    </div>
  );
}
