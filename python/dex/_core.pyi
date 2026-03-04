"""Type stubs for dex._core — the PyO3-compiled Rust extension."""

class TemplateMetaPy:
    name: str
    description: str
    version: str

class VariableSpecPy:
    name: str
    prompt: str
    var_type: str
    required: bool
    default: str | None
    choices: list[str] | None
    validate: str | None

class DabsBaseSpecPy:
    source: str
    variable_map: dict[str, str]

class TemplateManifestPy:
    name: str
    description: str
    version: str
    dabs: DabsBaseSpecPy | None
    variables: list[VariableSpecPy]

class ScaffoldResultPy:
    files_created: list[str]
    directories_created: list[str]

class AgentScaffoldResultPy:
    project_dir: str
    files_created: list[str]
    system_prompt: str
    claude_md: str

def render_template(template_str: str, variables: dict[str, object]) -> str: ...
def parse_template_manifest(path: str) -> TemplateManifestPy: ...
def scaffold_project(
    template_source: str,
    template_name: str,
    target_dir: str,
    variables: dict[str, object],
) -> ScaffoldResultPy: ...
def scaffold_agent(
    answers: dict[str, object],
    target_dir: str,
) -> AgentScaffoldResultPy: ...
def get_template_variables(source: str, name: str) -> list[VariableSpecPy]: ...
def list_embedded_templates() -> list[TemplateMetaPy]: ...
