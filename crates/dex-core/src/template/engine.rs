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

    /// Evaluate a Jinja2 expression (e.g. `include_ci` or `database == 'postgres'`)
    /// against `context` and return its truthiness.
    ///
    /// A bare variable name behaves as before; names missing from the context
    /// are undefined and therefore falsy.
    pub fn eval_condition(
        &self,
        expr: &str,
        context: &minijinja::Value,
    ) -> Result<bool, DexError> {
        let compiled = self.env.compile_expression(expr)?;
        Ok(compiled.eval(context)?.is_true())
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
    fn eval_condition_bare_name_and_expression() {
        let engine = TemplateEngine::new();
        let ctx = context! { include_ci => true, database => "postgres", off => false };
        assert!(engine.eval_condition("include_ci", &ctx).unwrap());
        assert!(!engine.eval_condition("off", &ctx).unwrap());
        assert!(!engine.eval_condition("missing", &ctx).unwrap());
        assert!(engine.eval_condition("database == 'postgres'", &ctx).unwrap());
        assert!(!engine.eval_condition("database == 'lakebase'", &ctx).unwrap());
        assert!(
            engine
                .eval_condition("include_ci and database != 'lakebase'", &ctx)
                .unwrap()
        );
        assert!(engine.eval_condition("1 +", &ctx).is_err());
    }

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
