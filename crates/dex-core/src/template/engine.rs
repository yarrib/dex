//! Template rendering engine backed by minijinja.

use minijinja::Environment;

use crate::error::DexError;

/// Wrapper around minijinja for rendering template strings.
pub struct TemplateEngine {
    env: Environment<'static>,
}

impl TemplateEngine {
    /// Create a new template engine with default configuration.
    #[must_use]
    pub fn new() -> Self {
        let mut env = Environment::new();
        // Disable auto-escaping — we're generating source files, not HTML.
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        Self { env }
    }

    /// Render a template string with the given context variables.
    pub fn render_string(
        &self,
        template_str: &str,
        context: &minijinja::Value,
    ) -> Result<String, DexError> {
        let rendered = self.env.render_str(template_str, context)?;
        Ok(rendered)
    }

    /// Render a path string (for variable interpolation in file paths).
    ///
    /// Uses the same Jinja2 syntax: `{{ project_name }}/src/main.py`
    pub fn render_path(
        &self,
        path_template: &str,
        context: &minijinja::Value,
    ) -> Result<String, DexError> {
        self.render_string(path_template, context)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use minijinja::context;

    #[test]
    fn render_simple_string() {
        let engine = TemplateEngine::new();
        let ctx = context! { name => "world" };
        let result = engine.render_string("Hello, {{ name }}!", &ctx).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn render_path_with_variables() {
        let engine = TemplateEngine::new();
        let ctx = context! { project_name => "my_project" };
        let result = engine
            .render_path("src/{{ project_name }}/__init__.py", &ctx)
            .unwrap();
        assert_eq!(result, "src/my_project/__init__.py");
    }

    #[test]
    fn render_with_conditionals() {
        let engine = TemplateEngine::new();
        let ctx = context! { include_ci => true };
        let template = "{% if include_ci %}ci: true{% endif %}";
        let result = engine.render_string(template, &ctx).unwrap();
        assert_eq!(result, "ci: true");
    }
}
