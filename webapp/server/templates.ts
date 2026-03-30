import { Router } from "express";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import nunjucks from "nunjucks";

export const templatesRouter = Router();

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/** Resolve the templates directory — walk up from server/ to repo root. */
function getTemplatesDir(): string {
  // In development: ../templates (from webapp/server/)
  // In production: configurable via TEMPLATES_DIR env var
  if (process.env.TEMPLATES_DIR) {
    return process.env.TEMPLATES_DIR;
  }
  // Walk up to find the repo root templates/ directory
  let dir = path.resolve(__dirname, "..");
  for (let i = 0; i < 5; i++) {
    const candidate = path.join(dir, "templates");
    if (fs.existsSync(candidate)) {
      return candidate;
    }
    dir = path.dirname(dir);
  }
  return path.resolve(__dirname, "../../templates");
}

const TEMPLATES_DIR = getTemplatesDir();

interface TomlVariable {
  name?: string;
  prompt: string;
  type?: string;
  required?: boolean;
  default?: string | boolean | string[];
  choices?: string[];
  validate?: string;
  order?: number;
}

interface TomlManifest {
  template: { name: string; description: string; version: string };
  variables?: TomlVariable[] | Record<string, TomlVariable>;
  files?: { src: string; condition?: string; dest?: string; overwrite?: boolean }[];
}

/** Minimal TOML parser for template.toml files.
 *  Handles the subset we need without pulling in a full TOML library.
 *  Falls back gracefully for complex cases. */
function parseTemplateToml(content: string): TomlManifest {
  const result: Record<string, Record<string, unknown>> = {};
  let currentSection = "";
  let currentArrayItem: Record<string, unknown> | null = null;
  const arrayItems: Record<string, Record<string, unknown>[]> = {};
  const inlineTableSection: Record<string, Record<string, Record<string, unknown>>> = {};

  for (const rawLine of content.split("\n")) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) continue;

    // Array of tables: [[variables]]
    const arrayMatch = line.match(/^\[\[(\w+)\]\]$/);
    if (arrayMatch) {
      const key = arrayMatch[1]!;
      currentSection = `__array__${key}`;
      currentArrayItem = {};
      if (!arrayItems[key]) arrayItems[key] = [];
      arrayItems[key]!.push(currentArrayItem);
      continue;
    }

    // Section header: [template] or [variables]
    const sectionMatch = line.match(/^\[([^\[\]]+)\]$/);
    if (sectionMatch) {
      currentSection = sectionMatch[1]!;
      currentArrayItem = null;
      if (!result[currentSection]) result[currentSection] = {};
      continue;
    }

    // Key-value pair
    const kvMatch = line.match(/^(\w+)\s*=\s*(.+)$/);
    if (kvMatch) {
      const key = kvMatch[1]!;
      const rawValue = kvMatch[2]!;
      const value = parseTomlValue(rawValue);

      if (currentArrayItem) {
        currentArrayItem[key] = value;
      } else if (currentSection.startsWith("variables.") || isInlineVariableSection(currentSection, rawValue)) {
        // Handle [variables] section with inline table values
        // e.g., project_name = { prompt = "Project name", required = true }
        if (currentSection === "variables" && typeof value === "object" && value !== null && !Array.isArray(value)) {
          if (!inlineTableSection["variables"]) inlineTableSection["variables"] = {};
          inlineTableSection["variables"]![key] = value as Record<string, unknown>;
        } else {
          if (!result[currentSection]) result[currentSection] = {};
          result[currentSection]![key] = value;
        }
      } else {
        if (!result[currentSection]) result[currentSection] = {};
        result[currentSection]![key] = value;
      }
    }
  }

  // Build the manifest
  const template = result["template"] || {};
  const manifest: TomlManifest = {
    template: {
      name: (template["name"] as string) || "",
      description: (template["description"] as string) || "",
      version: (template["version"] as string) || "0.1.0",
    },
  };

  // Handle variables — either [[variables]] array or [variables] inline map
  if (arrayItems["variables"]) {
    manifest.variables = arrayItems["variables"] as unknown as TomlVariable[];
  } else if (inlineTableSection["variables"]) {
    manifest.variables = {};
    for (const [key, val] of Object.entries(inlineTableSection["variables"])) {
      (manifest.variables as Record<string, TomlVariable>)[key] = val as unknown as TomlVariable;
    }
  }

  if (arrayItems["files"]) {
    manifest.files = arrayItems["files"] as unknown as TomlManifest["files"];
  }

  return manifest;
}

function isInlineVariableSection(section: string, rawValue: string): boolean {
  return section === "variables" && rawValue.trim().startsWith("{");
}

function parseTomlValue(raw: string): unknown {
  const trimmed = raw.trim();

  // String
  if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return trimmed.slice(1, -1);
  }

  // Boolean
  if (trimmed === "true") return true;
  if (trimmed === "false") return false;

  // Number
  if (/^-?\d+(\.\d+)?$/.test(trimmed)) return Number(trimmed);

  // Array
  if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
    const inner = trimmed.slice(1, -1).trim();
    if (!inner) return [];
    return inner.split(",").map((s) => parseTomlValue(s.trim()));
  }

  // Inline table: { key = "value", key2 = true }
  if (trimmed.startsWith("{") && trimmed.endsWith("}")) {
    const inner = trimmed.slice(1, -1).trim();
    const obj: Record<string, unknown> = {};
    // Split on commas but respect nested structures
    const parts = splitInlineTable(inner);
    for (const part of parts) {
      const m = part.trim().match(/^(\w+)\s*=\s*(.+)$/);
      if (m) {
        obj[m[1]!] = parseTomlValue(m[2]!);
      }
    }
    return obj;
  }

  return trimmed;
}

function splitInlineTable(s: string): string[] {
  const parts: string[] = [];
  let depth = 0;
  let current = "";
  for (const ch of s) {
    if (ch === "[" || ch === "{") depth++;
    else if (ch === "]" || ch === "}") depth--;
    if (ch === "," && depth === 0) {
      parts.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  if (current.trim()) parts.push(current);
  return parts;
}

/** Normalize variables from either format into a consistent array. */
function normalizeVariables(
  vars: TomlVariable[] | Record<string, TomlVariable> | undefined,
): Array<{
  name: string;
  prompt: string;
  type: string;
  required: boolean;
  default?: string | boolean | string[];
  choices?: string[];
  validate?: string;
}> {
  if (!vars) return [];

  const list: TomlVariable[] = Array.isArray(vars)
    ? vars
    : Object.entries(vars).map(([name, spec]) => ({ ...spec, name }));

  return list.map((v) => ({
    name: v.name || "",
    prompt: v.prompt || v.name || "",
    type: v.type || "string",
    required: v.required ?? false,
    default: v.default,
    choices: v.choices,
    validate: v.validate,
  }));
}

/**
 * GET /api/templates
 * List available templates.
 */
templatesRouter.get("/", (_req, res) => {
  try {
    if (!fs.existsSync(TEMPLATES_DIR)) {
      res.json([]);
      return;
    }
    const dirs = fs.readdirSync(TEMPLATES_DIR, { withFileTypes: true });
    const templates = [];

    for (const dir of dirs) {
      if (!dir.isDirectory()) continue;
      const manifestPath = path.join(TEMPLATES_DIR, dir.name, "template.toml");
      if (!fs.existsSync(manifestPath)) continue;

      const content = fs.readFileSync(manifestPath, "utf-8");
      const manifest = parseTemplateToml(content);
      templates.push({
        name: manifest.template.name,
        description: manifest.template.description,
        version: manifest.template.version,
      });
    }

    res.json(templates);
  } catch (err) {
    res.status(500).json({ error: `Failed to list templates: ${err}` });
  }
});

/**
 * GET /api/templates/:name
 * Get a template's full definition with variables.
 */
templatesRouter.get("/:name", (req, res) => {
  try {
    const name = req.params["name"]!;
    const templateDir = path.join(TEMPLATES_DIR, name);
    const manifestPath = path.join(templateDir, "template.toml");

    if (!fs.existsSync(manifestPath)) {
      res.status(404).json({ error: `Template '${name}' not found` });
      return;
    }

    const content = fs.readFileSync(manifestPath, "utf-8");
    const manifest = parseTemplateToml(content);

    res.json({
      meta: {
        name: manifest.template.name,
        description: manifest.template.description,
        version: manifest.template.version,
      },
      variables: normalizeVariables(manifest.variables),
    });
  } catch (err) {
    res.status(500).json({ error: `${err}` });
  }
});

/**
 * POST /api/templates/:name/render
 * Render a template with the given variables. Returns generated files.
 */
templatesRouter.post("/:name/render", (req, res) => {
  try {
    const name = req.params["name"]!;
    const { variables } = req.body as { variables: Record<string, string | boolean | string[]> };
    const templateDir = path.join(TEMPLATES_DIR, name);
    const filesDir = path.join(templateDir, "files");
    const manifestPath = path.join(templateDir, "template.toml");

    if (!fs.existsSync(manifestPath)) {
      res.status(404).json({ error: `Template '${name}' not found` });
      return;
    }

    // Parse manifest for file rules
    const manifestContent = fs.readFileSync(manifestPath, "utf-8");
    const manifest = parseTemplateToml(manifestContent);

    // Configure nunjucks for Jinja2-compatible rendering
    const env = nunjucks.configure(filesDir, { autoescape: false });

    const generatedFiles: { path: string; content: string; isNew: boolean }[] = [];

    function walkDir(dir: string, relBase: string) {
      if (!fs.existsSync(dir)) return;
      const entries = fs.readdirSync(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        // Render directory names with variables (e.g., {{ project_name }})
        const renderedName = renderPath(entry.name, variables);

        if (entry.isDirectory()) {
          const relDir = relBase ? `${relBase}/${renderedName}` : renderedName;

          // Check file rules for conditional inclusion
          if (!shouldInclude(relDir, manifest.files, variables)) continue;

          walkDir(fullPath, relDir);
        } else {
          let relPath = relBase ? `${relBase}/${renderedName}` : renderedName;

          // Check file rules for conditional inclusion
          const relDir = relBase || "";
          if (!shouldInclude(`${relDir}/`, manifest.files, variables)) continue;

          let content: string;
          if (entry.name.endsWith(".j2")) {
            // Render as Jinja2 template
            const templateContent = fs.readFileSync(fullPath, "utf-8");
            content = env.renderString(templateContent, variables);
            // Strip .j2 extension
            relPath = relPath.replace(/\.j2$/, "");
          } else {
            content = fs.readFileSync(fullPath, "utf-8");
          }

          generatedFiles.push({ path: relPath, content, isNew: true });
        }
      }
    }

    walkDir(filesDir, "");
    res.json(generatedFiles);
  } catch (err) {
    res.status(500).json({ error: `Template rendering failed: ${err}` });
  }
});

/** Render variable interpolation in file/directory paths. */
function renderPath(pathStr: string, variables: Record<string, string | boolean | string[]>): string {
  return pathStr.replace(/\{\{\s*(\w+)\s*\}\}/g, (_, key: string) => {
    const val = variables[key];
    return typeof val === "string" ? val : String(val ?? key);
  });
}

/** Check if a file/dir should be included based on file rules. */
function shouldInclude(
  relPath: string,
  rules: TomlManifest["files"],
  variables: Record<string, string | boolean | string[]>,
): boolean {
  if (!rules) return true;
  for (const rule of rules) {
    if (relPath.startsWith(rule.src)) {
      if (rule.condition) {
        const val = variables[rule.condition];
        if (!val) return false;
      }
    }
  }
  return true;
}
