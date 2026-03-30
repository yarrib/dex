import { useState, useRef, useEffect, useCallback } from "react";
import type {
  ChatMessage as ChatMessageType,
  FlowStep,
  GitHubUser,
  GitHubRepo,
  GeneratedFile,
} from "./types";
import { Header } from "./components/Header";
import { ChatMessage } from "./components/ChatMessage";
import { ChatInput } from "./components/ChatInput";
import * as api from "./lib/api";

let nextId = 0;
function makeId(): string {
  return `msg-${++nextId}-${Date.now()}`;
}

export default function App() {
  const [messages, setMessages] = useState<ChatMessageType[]>([]);
  const [step, setStep] = useState<FlowStep>("welcome");
  const [user, setUser] = useState<GitHubUser | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [repo, setRepo] = useState<GitHubRepo | null>(null);
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null);
  const [generatedFiles, setGeneratedFiles] = useState<GeneratedFile[]>([]);
  const [variables, setVariables] = useState<Record<string, string | boolean | string[]>>({});
  const [inputDisabled, setInputDisabled] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom on new messages
  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight, behavior: "smooth" });
  }, [messages]);

  // Restore session from sessionStorage
  useEffect(() => {
    const savedToken = sessionStorage.getItem("gh_token");
    const savedUser = sessionStorage.getItem("gh_user");
    if (savedToken && savedUser) {
      setToken(savedToken);
      setUser(JSON.parse(savedUser) as GitHubUser);
    }
  }, []);

  const addMessage = useCallback(
    (role: ChatMessageType["role"], content: string, widget?: ChatMessageType["widget"]) => {
      setMessages((prev) => [
        ...prev,
        { id: makeId(), role, content, timestamp: Date.now(), widget },
      ]);
    },
    [],
  );

  // Welcome message on mount
  useEffect(() => {
    const timer = setTimeout(() => {
      addMessage(
        "assistant",
        "Welcome to dex scaffold! I'll help you set up a new project — no CLI install needed.\n\nHere's how it works:\n1. Connect your GitHub account\n2. Choose or create a repository\n3. Pick a project template\n4. Configure your project\n5. Push the scaffolded code\n\nLet's get started by connecting your GitHub account.",
        { type: "auth-button" },
      );
      setStep("auth");
    }, 300);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ── Action handler (dispatches widget actions) ──────────────────────

  async function handleAction(action: string, payload?: unknown) {
    switch (action) {
      case "auth-success": {
        const { token: t, user: u } = payload as { token: string; user: GitHubUser };
        setToken(t);
        setUser(u);
        sessionStorage.setItem("gh_token", t);
        sessionStorage.setItem("gh_user", JSON.stringify(u));

        addMessage("system", `Signed in as ${u.login}`);
        addMessage(
          "assistant",
          `Great, you're connected as **${u.name || u.login}**!\n\nNow, would you like to scaffold into an existing repository or create a new one?`,
          {
            type: "choice-buttons",
            choices: [
              { label: "Use existing repo", value: "existing-repo" },
              { label: "Create new repo", value: "new-repo" },
            ],
          },
        );
        setStep("repo");
        break;
      }

      case "auth-error":
        addMessage(
          "assistant",
          "Something went wrong with GitHub authentication. Let's try again.",
          { type: "auth-button" },
        );
        break;

      case "choice": {
        const value = payload as string;
        addMessage("user", value === "existing-repo" ? "Use an existing repo" : "Create a new repo");

        if (value === "existing-repo") {
          addMessage("assistant", "Here are your recent repositories. Pick one to scaffold into:", {
            type: "repo-picker",
          });
        } else if (value === "new-repo") {
          addMessage("assistant", "Let's create a new repository:", { type: "repo-creator" });
        }
        break;
      }

      case "select-repo": {
        const r = payload as GitHubRepo;
        setRepo(r);
        addMessage("user", `Selected ${r.fullName}`);
        addMessage("system", `Repository: ${r.fullName}`);

        // Load templates
        try {
          const templates = await api.listTemplates();
          addMessage(
            "assistant",
            `You're scaffolding into **${r.fullName}**.\n\nNow pick a project template. Each template provides an opinionated starting point for a different kind of Databricks project:`,
            { type: "template-picker", templates },
          );
          setStep("template");
        } catch {
          addMessage("assistant", "Failed to load templates. Please try refreshing the page.");
        }
        break;
      }

      case "create-new-repo":
        addMessage("assistant", "Let's create a new repository:", { type: "repo-creator" });
        break;

      case "back-to-repo-picker":
        addMessage("assistant", "Here are your recent repositories:", { type: "repo-picker" });
        break;

      case "select-template": {
        const name = payload as string;
        setSelectedTemplate(name);
        addMessage("user", `Template: ${name}`);

        try {
          const template = await api.getTemplate(name);
          addMessage("system", `Template: ${template.meta.name} v${template.meta.version}`);

          if (template.variables.length > 0) {
            addMessage(
              "assistant",
              `Great choice! **${template.meta.name}** — ${template.meta.description}\n\nLet's configure your project. Fill in the details below:`,
              { type: "variable-form", variables: template.variables },
            );
            setStep("variables");
          } else {
            // No variables — go straight to rendering
            addMessage("assistant", "This template has no configuration needed. Generating...");
            await renderAndPreview(name, {});
          }
        } catch {
          addMessage("assistant", "Failed to load template details. Please try again.");
        }
        break;
      }

      case "submit-variables": {
        const vars = payload as Record<string, string | boolean | string[]>;
        setVariables(vars);
        const projectName = vars["project_name"] || selectedTemplate;
        addMessage("user", `Configure: ${projectName}`);
        await renderAndPreview(selectedTemplate!, vars);
        break;
      }

      case "confirm-push": {
        if (!repo) return;
        const branchName = `dex/scaffold-${(variables["project_name"] as string) || "project"}`;
        addMessage("user", "Ready to push");
        addMessage("assistant", "Let's review what we're about to do:", {
          type: "push-confirm",
          repo,
          branch: branchName,
          fileCount: generatedFiles.length,
        });
        setStep("push");
        break;
      }

      case "back-to-variables": {
        if (selectedTemplate) {
          const template = await api.getTemplate(selectedTemplate);
          addMessage("assistant", "Let's reconfigure:", {
            type: "variable-form",
            variables: template.variables,
          });
          setStep("variables");
        }
        break;
      }

      case "back-to-preview":
        addMessage("assistant", "Here are your generated files:", {
          type: "file-preview",
          files: generatedFiles,
        });
        setStep("preview");
        break;

      case "execute-push":
        await executePush();
        break;
    }
  }

  // ── Template rendering ──────────────────────────────────────────────

  async function renderAndPreview(
    templateName: string,
    vars: Record<string, string | boolean | string[]>,
  ) {
    try {
      setInputDisabled(true);
      addMessage("system", "Generating project files...");
      const files = await api.renderTemplate(templateName, vars);
      setGeneratedFiles(files);
      addMessage(
        "assistant",
        `Your project is ready! I've generated **${files.length} files**. Review them below — you can click any file to preview its contents.`,
        { type: "file-preview", files },
      );
      setStep("preview");
    } catch (err) {
      addMessage("assistant", `Failed to generate project: ${err}`);
    } finally {
      setInputDisabled(false);
    }
  }

  // ── Push to GitHub ──────────────────────────────────────────────────

  async function executePush() {
    if (!token || !repo) return;
    const branchName = `dex/scaffold-${(variables["project_name"] as string) || "project"}`;

    try {
      setInputDisabled(true);
      addMessage("system", "Pushing files to GitHub...");

      await api.pushFiles(
        token,
        repo.fullName,
        branchName,
        generatedFiles,
        `feat: scaffold ${selectedTemplate} project via dex\n\nGenerated by dex scaffold web app.`,
      );

      addMessage("system", "Files pushed successfully!");

      // Create PR
      addMessage("system", "Creating pull request...");
      const pr = await api.createPR(
        token,
        repo.fullName,
        branchName,
        repo.defaultBranch,
        `feat: scaffold ${selectedTemplate || "dex"} project`,
        `## Scaffolded with dex\n\n**Template:** ${selectedTemplate}\n**Files:** ${generatedFiles.length}\n\nThis project was scaffolded using the [dex scaffold web app](https://github.com/yarrib/dex).\n\n### Generated files\n\n${generatedFiles.map((f) => `- \`${f.path}\``).join("\n")}`,
      );

      addMessage(
        "assistant",
        "Your project has been scaffolded and pushed! Here's your pull request — review it and merge when ready.",
        { type: "pr-link", url: pr.url },
      );

      addMessage(
        "assistant",
        "**What's next?**\n\n1. Review and merge the PR\n2. Clone the repo locally\n3. Use a coding assistant (Claude Code, Codex, Genie) to start building\n4. Run `dex` commands for ongoing operations\n\nYou can close this tab or start a new scaffolding session by refreshing.",
      );

      setStep("done");
    } catch (err) {
      addMessage("assistant", `Push failed: ${err}\n\nPlease check your repository permissions and try again.`);
    } finally {
      setInputDisabled(false);
    }
  }

  // ── Free text input (contextual) ───────────────────────────────────

  function handleSend(text: string) {
    addMessage("user", text);

    // Context-aware responses for free text
    const lower = text.toLowerCase();

    if (step === "welcome" || step === "auth") {
      if (lower.includes("help") || lower.includes("what")) {
        addMessage(
          "assistant",
          "dex scaffold helps you create new Databricks projects from templates without installing anything. Just connect your GitHub, pick a template, configure it, and I'll push the code for you.\n\nLet's start by connecting your GitHub account:",
          { type: "auth-button" },
        );
      } else {
        addMessage("assistant", "Let's get started! First, connect your GitHub account:", {
          type: "auth-button",
        });
      }
    } else if (step === "repo") {
      if (lower.includes("new") || lower.includes("create")) {
        addMessage("assistant", "Let's create a new repository:", { type: "repo-creator" });
      } else if (lower.includes("exist") || lower.includes("use") || lower.includes("pick")) {
        addMessage("assistant", "Here are your repositories:", { type: "repo-picker" });
      } else {
        addMessage("assistant", "Would you like to use an existing repo or create a new one?", {
          type: "choice-buttons",
          choices: [
            { label: "Use existing repo", value: "existing-repo" },
            { label: "Create new repo", value: "new-repo" },
          ],
        });
      }
    } else if (step === "done") {
      addMessage(
        "assistant",
        "Your project is all set! Refresh the page to scaffold another project, or close this tab.",
      );
    } else {
      addMessage(
        "assistant",
        "Please use the interactive elements above to continue. If you need help, just type 'help'.",
      );
    }
  }

  // ── Render ──────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col h-dvh max-h-dvh">
      <Header user={user} currentStep={step} />

      {/* Chat area */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-3 sm:px-6 py-4">
        <div className="max-w-2xl mx-auto">
          {messages.map((msg) => (
            <ChatMessage key={msg.id} message={msg} onAction={handleAction} />
          ))}
        </div>
      </div>

      {/* Input */}
      <div className="max-w-2xl mx-auto w-full">
        <ChatInput
          onSend={handleSend}
          disabled={inputDisabled || step === "done"}
          placeholder={
            step === "done"
              ? "Session complete — refresh to start over"
              : step === "auth"
                ? "Connect GitHub above, or type 'help'"
                : "Type a message or use the controls above..."
          }
        />
      </div>
    </div>
  );
}
