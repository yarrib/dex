//! Python bindings for dex-core via PyO3.
//!
//! This crate is a thin FFI bridge. It converts between Python and Rust types,
//! translates errors to Python exceptions, and delegates all logic to dex-core.

use std::collections::HashMap;
use std::path::PathBuf;

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use dex_core::template::manifest::TemplateManifest;
use dex_core::template::TemplateSource;

/// Render a Jinja2 template string with the given variables.
#[pyfunction]
fn render_template(template_str: &str, variables: &Bound<'_, PyDict>) -> PyResult<String> {
    let engine = dex_core::template::TemplateEngine::new();
    let ctx = dict_to_minijinja_value(variables)?;
    engine
        .render_string(template_str, &ctx)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

/// Parse a template.toml manifest and return its metadata as a dict.
#[pyfunction]
fn parse_template_manifest(path: &str) -> PyResult<TemplateManifestPy> {
    let manifest = TemplateManifest::from_path(&PathBuf::from(path))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let dabs = manifest.template.dabs.map(|d| DabsBaseSpecPy {
        source: d.source,
        variable_map: d.variable_map,
    });

    Ok(TemplateManifestPy {
        name: manifest.template.name,
        description: manifest.template.description,
        version: manifest.template.version,
        dabs,
        variables: manifest
            .variables
            .into_iter()
            .map(|v| VariableSpecPy {
                name: v.name,
                prompt: v.prompt,
                var_type: format!("{:?}", v.var_type).to_lowercase(),
                required: v.required,
                default: v.default.map(|d| d.to_string()),
                choices: v.choices,
                validate: v.validate,
            })
            .collect(),
    })
}

/// Scaffold a project from a template directory.
#[pyfunction]
fn scaffold_project(
    template_source: &str,
    template_name: &str,
    target_dir: &str,
    variables: &Bound<'_, PyDict>,
) -> PyResult<ScaffoldResultPy> {
    let source = if template_source == "__embedded__" {
        TemplateSource::Embedded
    } else {
        TemplateSource::Directory(PathBuf::from(template_source))
    };

    let template = dex_core::template::registry::load_template(&source, template_name)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    let vars = dict_to_hashmap(variables)?;
    let result = dex_core::scaffold(&template, &PathBuf::from(target_dir), &vars)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(ScaffoldResultPy {
        files_created: result
            .files_created
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
        directories_created: result
            .directories_created
            .into_iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

/// Scaffold an agent project from Q&A answers.
#[pyfunction]
fn scaffold_agent(
    answers: &Bound<'_, PyDict>,
    target_dir: &str,
) -> PyResult<AgentScaffoldResultPy> {
    let a = dex_core::agent::AgentAnswers {
        name: extract_str(answers, "name")?,
        description: extract_str(answers, "description")?,
        trigger: match extract_str(answers, "trigger")?.as_str() {
            "schedule" => dex_core::AgentTrigger::Schedule,
            "event" => dex_core::AgentTrigger::Event,
            "upstream_system" => dex_core::AgentTrigger::UpstreamSystem,
            _ => dex_core::AgentTrigger::UserRequest,
        },
        success_criteria: extract_str(answers, "success_criteria")?,
        reads: extract_str(answers, "reads")?,
        writes: extract_str(answers, "writes")?,
        handoff: answers
            .get_item("handoff")
            .ok()
            .flatten()
            .and_then(|v| v.extract::<bool>().ok())
            .unwrap_or(false),
        autonomous: answers
            .get_item("autonomous")
            .ok()
            .flatten()
            .and_then(|v| v.extract::<bool>().ok())
            .unwrap_or(true),
        example_input: extract_str(answers, "example_input")?,
        example_output: extract_str(answers, "example_output")?,
        bad_output: extract_str(answers, "bad_output")?,
        deploy_target: match extract_str(answers, "deploy_target")?.as_str() {
            "serving_endpoint" => dex_core::AgentDeployTarget::ServingEndpoint,
            "interactive" => dex_core::AgentDeployTarget::Interactive,
            _ => dex_core::AgentDeployTarget::Job,
        },
    };

    let result = dex_core::agent::scaffold_agent(&a, &PathBuf::from(target_dir))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(AgentScaffoldResultPy {
        project_dir: result.project_dir.to_string_lossy().to_string(),
        files_created: result
            .files_created
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

/// Return the variable specs for a named template.
#[pyfunction]
fn get_template_variables(source: &str, name: &str) -> PyResult<Vec<VariableSpecPy>> {
    let template_source = if source == "__embedded__" {
        TemplateSource::Embedded
    } else {
        TemplateSource::Directory(PathBuf::from(source))
    };

    let template = dex_core::template::registry::load_template(&template_source, name)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(template
        .variables
        .into_iter()
        .map(|v| VariableSpecPy {
            name: v.name,
            prompt: v.prompt,
            var_type: format!("{:?}", v.var_type).to_lowercase(),
            required: v.required,
            default: v.default.map(|d| d.to_string()),
            choices: v.choices,
            validate: v.validate,
        })
        .collect())
}

/// List available embedded templates.
#[pyfunction]
fn list_embedded_templates() -> PyResult<Vec<TemplateMetaPy>> {
    let templates = dex_core::template::registry::list_templates(&TemplateSource::Embedded)
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    Ok(templates
        .into_iter()
        .map(|t| TemplateMetaPy {
            name: t.name,
            description: t.description,
            version: t.version,
        })
        .collect())
}

// --- Python types ---

#[pyclass]
#[derive(Clone)]
struct TemplateManifestPy {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    description: String,
    #[pyo3(get)]
    version: String,
    #[pyo3(get)]
    dabs: Option<DabsBaseSpecPy>,
    #[pyo3(get)]
    variables: Vec<VariableSpecPy>,
}

#[pyclass]
#[derive(Clone)]
struct DabsBaseSpecPy {
    #[pyo3(get)]
    source: String,
    #[pyo3(get)]
    variable_map: HashMap<String, String>,
}

#[pyclass]
#[derive(Clone)]
struct VariableSpecPy {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    prompt: String,
    #[pyo3(get)]
    var_type: String,
    #[pyo3(get)]
    required: bool,
    #[pyo3(get)]
    default: Option<String>,
    #[pyo3(get)]
    choices: Option<Vec<String>>,
    #[pyo3(get)]
    validate: Option<String>,
}

#[pyclass]
#[derive(Clone)]
struct TemplateMetaPy {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    description: String,
    #[pyo3(get)]
    version: String,
}

#[pyclass]
#[derive(Clone)]
struct ScaffoldResultPy {
    #[pyo3(get)]
    files_created: Vec<String>,
    #[pyo3(get)]
    directories_created: Vec<String>,
}

#[pyclass]
#[derive(Clone)]
struct AgentScaffoldResultPy {
    #[pyo3(get)]
    project_dir: String,
    #[pyo3(get)]
    files_created: Vec<String>,
}

// --- Helpers ---

fn dict_to_minijinja_value(dict: &Bound<'_, PyDict>) -> PyResult<minijinja::Value> {
    let mut map = std::collections::BTreeMap::new();
    for (key, value) in dict.iter() {
        let k: String = key.extract()?;
        let v = python_to_minijinja_value(&value)?;
        map.insert(k, v);
    }
    Ok(minijinja::Value::from_serialize(&map))
}

fn dict_to_hashmap(dict: &Bound<'_, PyDict>) -> PyResult<HashMap<String, minijinja::Value>> {
    let mut map = HashMap::new();
    for (key, value) in dict.iter() {
        let k: String = key.extract()?;
        let v = python_to_minijinja_value(&value)?;
        map.insert(k, v);
    }
    Ok(map)
}

fn python_to_minijinja_value(obj: &Bound<'_, PyAny>) -> PyResult<minijinja::Value> {
    if let Ok(s) = obj.extract::<String>() {
        Ok(minijinja::Value::from(s))
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(minijinja::Value::from(b))
    } else if let Ok(i) = obj.extract::<i64>() {
        Ok(minijinja::Value::from(i))
    } else if let Ok(f) = obj.extract::<f64>() {
        Ok(minijinja::Value::from(f))
    } else {
        Ok(minijinja::Value::from(obj.str()?.to_string()))
    }
}

/// Extract a string value from a Python dict, returning a PyResult.
fn extract_str(dict: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    dict.get_item(key)
        .ok()
        .flatten()
        .and_then(|v| v.extract::<String>().ok())
        .ok_or_else(|| PyValueError::new_err(format!("missing required key: {key}")))
}

/// The dex._core Python module.
#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(render_template, m)?)?;
    m.add_function(wrap_pyfunction!(parse_template_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(scaffold_project, m)?)?;
    m.add_function(wrap_pyfunction!(scaffold_agent, m)?)?;
    m.add_function(wrap_pyfunction!(get_template_variables, m)?)?;
    m.add_function(wrap_pyfunction!(list_embedded_templates, m)?)?;
    m.add_class::<TemplateManifestPy>()?;
    m.add_class::<VariableSpecPy>()?;
    m.add_class::<TemplateMetaPy>()?;
    m.add_class::<DabsBaseSpecPy>()?;
    m.add_class::<ScaffoldResultPy>()?;
    m.add_class::<AgentScaffoldResultPy>()?;
    Ok(())
}
