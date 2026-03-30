import { useState } from "react";
import type { VariableSpec } from "../../types";

interface Props {
  variables: VariableSpec[];
  onAction: (action: string, payload?: unknown) => void;
}

export function VariableForm({ variables, onAction }: Props) {
  const [values, setValues] = useState<Record<string, string | boolean | string[]>>(() => {
    const defaults: Record<string, string | boolean | string[]> = {};
    for (const v of variables) {
      if (v.default !== undefined) {
        defaults[v.name] = v.default;
      } else if (v.type === "bool") {
        defaults[v.name] = false;
      } else {
        defaults[v.name] = "";
      }
    }
    return defaults;
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  function handleChange(name: string, value: string | boolean) {
    setValues((prev) => ({ ...prev, [name]: value }));
    setErrors((prev) => {
      const next = { ...prev };
      delete next[name];
      return next;
    });
  }

  function handleSubmit() {
    const newErrors: Record<string, string> = {};
    for (const v of variables) {
      if (v.required && !values[v.name]) {
        newErrors[v.name] = "Required";
      }
      if (v.validate && typeof values[v.name] === "string") {
        const re = new RegExp(v.validate);
        if (!re.test(values[v.name] as string)) {
          newErrors[v.name] = `Must match: ${v.validate}`;
        }
      }
    }

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }

    onAction("submit-variables", values);
  }

  return (
    <div className="space-y-4">
      {variables.map((v) => (
        <div key={v.name}>
          <label className="block text-xs font-medium text-gray-400 mb-1.5">
            {v.prompt}
            {v.required && <span className="text-red-400 ml-1">*</span>}
          </label>

          {v.type === "bool" ? (
            <button
              onClick={() => handleChange(v.name, !values[v.name])}
              className={`flex items-center gap-2 px-3 py-2 rounded-lg border transition-colors ${
                values[v.name]
                  ? "bg-dex-600/20 border-dex-500/50 text-dex-300"
                  : "bg-gray-900 border-gray-700 text-gray-400"
              }`}
            >
              <div
                className={`w-4 h-4 rounded border flex items-center justify-center ${
                  values[v.name] ? "bg-dex-600 border-dex-500" : "border-gray-600"
                }`}
              >
                {values[v.name] && (
                  <svg className="w-3 h-3 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                  </svg>
                )}
              </div>
              {values[v.name] ? "Yes" : "No"}
            </button>
          ) : v.type === "choice" && v.choices ? (
            <div className="flex flex-wrap gap-2">
              {v.choices.map((c) => (
                <button
                  key={c}
                  onClick={() => handleChange(v.name, c)}
                  className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${
                    values[v.name] === c
                      ? "bg-dex-600/20 border-dex-500/50 text-dex-300"
                      : "bg-gray-900 border-gray-700 text-gray-400 hover:border-gray-600"
                  }`}
                >
                  {c}
                </button>
              ))}
            </div>
          ) : (
            <input
              type="text"
              value={(values[v.name] as string) || ""}
              onChange={(e) => handleChange(v.name, e.target.value)}
              placeholder={v.default ? String(v.default) : `Enter ${v.name}`}
              className="input-field text-sm"
            />
          )}

          {errors[v.name] && (
            <p className="text-xs text-red-400 mt-1">{errors[v.name]}</p>
          )}
        </div>
      ))}

      <button onClick={handleSubmit} className="btn-primary w-full mt-4">
        Generate Project
      </button>
    </div>
  );
}
