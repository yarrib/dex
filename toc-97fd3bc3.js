// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="index.html"><strong aria-hidden="true">1.</strong> Home</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="quickstart.html"><strong aria-hidden="true">2.</strong> Quickstart</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="installation.html"><strong aria-hidden="true">3.</strong> Installation</a></span></li><li class="chapter-item expanded "><li class="part-title">Usage</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="usage/index.html"><strong aria-hidden="true">4.</strong> Overview</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="usage/init.html"><strong aria-hidden="true">5.</strong> dex init</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="usage/templates.html"><strong aria-hidden="true">6.</strong> Templates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="usage/agent.html"><strong aria-hidden="true">7.</strong> dex agent new</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="usage/mcp.html"><strong aria-hidden="true">8.</strong> dex mcp serve</a></span></li><li class="chapter-item expanded "><li class="part-title">Templates</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="templates/built-in.html"><strong aria-hidden="true">9.</strong> Built-in Templates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="templates/authoring.html"><strong aria-hidden="true">10.</strong> Authoring</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="templates/org-templates-guide.html"><strong aria-hidden="true">11.</strong> Building Org Templates</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="templates/org-templates.html"><strong aria-hidden="true">12.</strong> Org Template Registries</a></span></li><li class="chapter-item expanded "><li class="part-title">Reference</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="why-dex.html"><strong aria-hidden="true">13.</strong> Why dex?</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="extending.html"><strong aria-hidden="true">14.</strong> Extending dex</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="contributing.html"><strong aria-hidden="true">15.</strong> Contributing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="releasing.html"><strong aria-hidden="true">16.</strong> Releasing</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="changelog.html"><strong aria-hidden="true">17.</strong> Changelog</a></span></li><li class="chapter-item expanded "><li class="part-title">Project Memory</li></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/graph.html"><strong aria-hidden="true">18.</strong> Graph view</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/USER_MANUAL.html"><strong aria-hidden="true">19.</strong> How to read it</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/INDEX.html"><strong aria-hidden="true">20.</strong> Knowledge Map</a></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.1.</strong> Foundation &amp; Architecture</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/ff57dbb-add-runnable-org-setup-examples-and-fix.html"><strong aria-hidden="true">20.1.1.</strong> ff57dbb docs: add runnable org-setup examples and fix template config syntax (#56)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/b8fa631-add-prd-for-snowflake-templates-35.html"><strong aria-hidden="true">20.1.2.</strong> b8fa631 docs: add PRD for Snowflake templates (#35)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/bf5f8b5-add-regression-tests-for-embedded-template.html"><strong aria-hidden="true">20.1.3.</strong> bf5f8b5 test(core): add regression tests for embedded template variable and file loading (#27)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/d28c6c6-feat-user-config-14.html"><strong aria-hidden="true">20.1.4.</strong> d28c6c6 Feat/user config (#14)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/0ecad9f-add-cli-smoke-tests-to-fix-pytest-exit-code-5.html"><strong aria-hidden="true">20.1.5.</strong> 0ecad9f test: add CLI smoke tests to fix pytest exit code 5</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/2f9817c-add-core-pyi-stubs-suppress-ty-false-positives.html"><strong aria-hidden="true">20.1.6.</strong> 2f9817c fix(types): add _core.pyi stubs, suppress ty false positives on click.BaseCommand</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/0269bf6-resolve-clippy-collapsible-if-and-ruff-errors.html"><strong aria-hidden="true">20.1.7.</strong> 0269bf6 fix(lint): resolve clippy collapsible_if and ruff errors</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/c8b5f61-apply-ruff-format-to-cli-py.html"><strong aria-hidden="true">20.1.8.</strong> c8b5f61 fix(fmt): apply ruff format to cli.py</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/4dc2eb0-apply-cargo-fmt-to-bring-rust-sources-in-line.html"><strong aria-hidden="true">20.1.9.</strong> 4dc2eb0 fix(fmt): apply cargo fmt to bring Rust sources in line with rustfmt</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/c576227-slugify-hyphens-to-underscores-expose-system.html"><strong aria-hidden="true">20.1.10.</strong> c576227 fix: slugify hyphens to underscores, expose system_prompt and claude_md in PyO3 binding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/476e1ac-add-dabs-package-template-fix-multi-variable.html"><strong aria-hidden="true">20.1.11.</strong> 476e1ac feat: add dabs-package template, fix multi-variable scaffolding, add docs site</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/7409110-add-dabs-prompt-modes-and-agent-scaffolding.html"><strong aria-hidden="true">20.1.12.</strong> 7409110 feat: add DABs prompt modes and agent scaffolding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/4188e3b-add-project-specification-architecture-and.html"><strong aria-hidden="true">20.1.13.</strong> 4188e3b feat: add project specification, architecture, and initial scaffolding</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/493367b-initial-commit.html"><strong aria-hidden="true">20.1.14.</strong> 493367b Initial commit</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.2.</strong> Scaffolding &amp; Project Generation</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/6f10095-fade-python-docs-ci-churn-in-the-graph-view-69.html"><strong aria-hidden="true">20.2.1.</strong> 6f10095 feat(context): fade Python &amp; docs/CI churn in the graph view (#69)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/9e49d20-interactive-force-directed-graph-view-in-the.html"><strong aria-hidden="true">20.2.2.</strong> 9e49d20 feat(context): interactive force-directed graph view in the docs site (#64)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/d712d89-add-post-scaffold-activation-hook-on-success-49.html"><strong aria-hidden="true">20.2.3.</strong> d712d89 feat(scaffold): add post-scaffold activation hook (on_success) (#49)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/a25d62d-populate-tasks-from-scaffolded-dex-toml-48.html"><strong aria-hidden="true">20.2.4.</strong> a25d62d fix(context-map): populate tasks from scaffolded dex.toml (#48)</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.3.</strong> CLI &amp; Interfaces</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/0752c04-add-dex-templates-list-show-40.html"><strong aria-hidden="true">20.3.1.</strong> 0752c04 feat(cli): add dex templates list/show (#40)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/bacd088-add-dex-run-task-command-28.html"><strong aria-hidden="true">20.3.2.</strong> bacd088 feat(cli): add dex run  command (#28)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/a6ab3a6-add-web-based-project-scaffolding-app-26.html"><strong aria-hidden="true">20.3.3.</strong> a6ab3a6 feat: add web-based project scaffolding app (#26)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/30690b9-port-dex-to-pure-rust-single-binary-21.html"><strong aria-hidden="true">20.3.4.</strong> 30690b9 feat: port dex to pure Rust single binary (#21)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/22e29a5-resolve-all-python-bug-backlog-items.html"><strong aria-hidden="true">20.3.5.</strong> 22e29a5 fix(cli): resolve all Python bug backlog items</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.4.</strong> Skills, Traits &amp; Extensibility</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/2f1e593-add-dex-skills-system-agent-skill-pack.html"><strong aria-hidden="true">20.4.1.</strong> 2f1e593 feat(skills): add dex skills system — agent skill pack management (#36)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/6cbcd49-add-prd-for-ai-ready-scaffolding-context-map.html"><strong aria-hidden="true">20.4.2.</strong> 6cbcd49 docs: add PRD for AI-ready scaffolding (context map, traits, WASM) (#34)</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.5.</strong> MCP &amp; AI Integration</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/9013fda-dex-mcp-install-client-wiring-docs-55.html"><strong aria-hidden="true">20.5.1.</strong> 9013fda feat(mcp): dex mcp install + client wiring docs (#55)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/723c342-complete-mcp-v0-2-integration-tests-and.html"><strong aria-hidden="true">20.5.2.</strong> 723c342 feat(mcp): complete MCP v0.2 — integration tests and variable annotation (#51)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/3035344-implement-scaffold-agent-tool-and-add-mcp-json.html"><strong aria-hidden="true">20.5.3.</strong> 3035344 feat(mcp): implement scaffold_agent tool and add .mcp.json (#37)</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.6.</strong> Templates &amp; Built-in Content</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/ee9d6c7-add-dabs-tf-agent-agentic-terraform-iac.html"><strong aria-hidden="true">20.6.1.</strong> ee9d6c7 feat(templates): add dabs-tf-agent — agentic Terraform IaC template (#70)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/7f7b8e0-batteries-included-assistant-agnostic-agent.html"><strong aria-hidden="true">20.6.2.</strong> 7f7b8e0 feat(agent): batteries-included, assistant-agnostic agent scaffolding (#53)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/376c137-add-conditional-variable-visibility-via-when.html"><strong aria-hidden="true">20.6.3.</strong> 376c137 feat(templates): add conditional variable visibility via when field (#50)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/95dba9b-add-next-js-template-notebook-trait-and-context.html"><strong aria-hidden="true">20.6.4.</strong> 95dba9b feat: add next.js template, notebook trait, and context-map generation (#47)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/7062c81-add-scope-md-product-scope-guardrails-and.html"><strong aria-hidden="true">20.6.5.</strong> 7062c81 docs: add SCOPE.md — product scope guardrails and decision filter (#44)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/2eeb5a0-add-dabs-dashboard-template-43.html"><strong aria-hidden="true">20.6.6.</strong> 2eeb5a0 feat(templates): add dabs-dashboard template (#43)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/2239358-add-databricks-app-streamlit-template-42.html"><strong aria-hidden="true">20.6.7.</strong> 2239358 feat(templates): add databricks-app-streamlit template (#42)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/6e3f963-add-databricks-app-streamlit-template-41.html"><strong aria-hidden="true">20.6.8.</strong> 6e3f963 feat(templates): add databricks-app-streamlit template (#41)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/dfdc0e9-add-python-package-template-29.html"><strong aria-hidden="true">20.6.9.</strong> dfdc0e9 feat(templates): add python-package template (#29)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/7719579-inline-variables-format-order-field-and.html"><strong aria-hidden="true">20.6.10.</strong> 7719579 feat(templates): inline variables format, order field, and standards pre-fill (#20)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/1ad4db2-lighten-ai-agent-template-no-langchain-dbutils.html"><strong aria-hidden="true">20.6.11.</strong> 1ad4db2 refactor(dabs-aiagent): lighten AI agent template — no LangChain, dbutils notebooks, DABs deploy job</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/c3a7528-add-dabs-etl-dabs-ml-and-dabs-aiagent-pattern.html"><strong aria-hidden="true">20.6.12.</strong> c3a7528 feat(templates): add dabs-etl, dabs-ml, and dabs-aiagent pattern templates</a></span></li></ol><li class="chapter-item expanded "><span class="chapter-link-wrapper"><span><strong aria-hidden="true">20.7.</strong> Docs, CI &amp; Release</span></span><ol class="section"><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/326e811-authenticate-git-cliff-github-api-calls-v0-3-2.html"><strong aria-hidden="true">20.7.1.</strong> 326e811 fix(release): authenticate git-cliff GitHub API calls (v0.3.2 changelog 403) (#68)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/900b30f-cleaner-graph-drop-churn-strip-ai-trailers-tap.html"><strong aria-hidden="true">20.7.2.</strong> 900b30f feat(context): cleaner graph — drop churn, strip AI trailers, tap-to-open (#66)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/44ae727-project-memory-knowledge-graph-engine-62.html"><strong aria-hidden="true">20.7.3.</strong> 44ae727 feat(context): project-memory knowledge-graph engine (#62)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/9a70850-reliable-docs-redeploy-on-release-draft-github.html"><strong aria-hidden="true">20.7.4.</strong> 9a70850 fix(release): reliable docs redeploy on release + draft GitHub Releases (#61)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/be11212-generate-release-notes-with-latest-and-auto.html"><strong aria-hidden="true">20.7.5.</strong> be11212 fix(release): generate release notes with --latest and auto-annotate tags (#54)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/c04db4f-add-prd-for-org-validated-skills-mcp-server.html"><strong aria-hidden="true">20.7.6.</strong> c04db4f docs: add PRD for org-validated skills &amp; MCP server catalog (#52)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/a1dda0c-add-scaffolding-differentiation-prd-46.html"><strong aria-hidden="true">20.7.7.</strong> a1dda0c docs: add scaffolding differentiation PRD (#46)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/7b2cd91-add-prd-for-dex-in-browser-wasm-feature-45.html"><strong aria-hidden="true">20.7.8.</strong> 7b2cd91 docs: add PRD for dex-in-browser WASM feature (#45)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/4535cd0-ai-dev-kit-integration-with-profile-based-skill.html"><strong aria-hidden="true">20.7.9.</strong> 4535cd0 feat(devcontainer): ai-dev-kit integration with profile-based skill setup (#38)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/bb22702-use-cross-for-linux-musl-targets.html"><strong aria-hidden="true">20.7.10.</strong> bb22702 fix(release): use cross for Linux musl targets</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/b604a2d-support-workflow-dispatch-and-fix-first-release.html"><strong aria-hidden="true">20.7.11.</strong> b604a2d fix(release): support workflow_dispatch and fix first-release changelog</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/5da5907-support-workflow-dispatch-and-fix-first-release.html"><strong aria-hidden="true">20.7.12.</strong> 5da5907 fix(release): support workflow_dispatch and fix first-release changelog (#31)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/a378387-align-install-sh-artifact-names-with-release.html"><strong aria-hidden="true">20.7.13.</strong> a378387 fix(release): align install.sh artifact names with release.yml and add linux aarch64 target (#30)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/e3efd53-rewrite-all-docs-for-rust-binary-architecture-25.html"><strong aria-hidden="true">20.7.14.</strong> e3efd53 docs: rewrite all docs for Rust binary architecture (#25)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/bfeb2a5-move-no-mkdocs-2-warning-to-job-level-delete.html"><strong aria-hidden="true">20.7.15.</strong> bfeb2a5 fix(docs): move NO_MKDOCS_2_WARNING to job level, delete version before deploy</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/0dcf505-suppress-mkdocs-2-0-compatibility-warning.html"><strong aria-hidden="true">20.7.16.</strong> 0dcf505 fix(docs): suppress MkDocs 2.0 compatibility warning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/0c7c4f8-use-latest-as-version-name-deploy-numbered.html"><strong aria-hidden="true">20.7.17.</strong> 0c7c4f8 fix(docs): use latest as version name, deploy numbered versions on tags</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/d8715ff-add-mike-set-default-to-create-root-redirect-19.html"><strong aria-hidden="true">20.7.18.</strong> d8715ff fix(docs): add mike set-default to create root redirect (#19)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/439426a-resolve-gh-pages-deploy-alias-conflict-18.html"><strong aria-hidden="true">20.7.19.</strong> 439426a fix(docs): resolve gh-pages deploy alias conflict (#18)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/18f3db6-remove-redundant-version-stamp-step-in-build.html"><strong aria-hidden="true">20.7.20.</strong> 18f3db6 fix(release): remove redundant version stamp step in build jobs</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/b5d760e-fix-bump-version-idempotency-and-replace-git.html"><strong aria-hidden="true">20.7.21.</strong> b5d760e fix(release): fix bump-version idempotency and replace git-cliff Docker action</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/31c18f8-feat-documentation-12.html"><strong aria-hidden="true">20.7.22.</strong> 31c18f8 Feat/documentation (#12)</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/cfee45e-hoist-uv-python-to-job-level-env.html"><strong aria-hidden="true">20.7.23.</strong> cfee45e fix(ci): hoist UV_PYTHON to job-level env</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/289e43e-add-maturin-to-dev-extras-set-fail-fast-false.html"><strong aria-hidden="true">20.7.24.</strong> 289e43e fix(ci): add maturin to dev extras, set fail-fast: false on Python matrix</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/d439535-use-dtolnay-rust-toolchain-master-with-explicit.html"><strong aria-hidden="true">20.7.25.</strong> d439535 fix(ci): use dtolnay/rust-toolchain@master with explicit toolchain input</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/1772eb7-default-to-uv-tool-install-from-github-releases.html"><strong aria-hidden="true">20.7.26.</strong> 1772eb7 docs: default to uv tool install from GitHub Releases, remove pip/PyPI references</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/22f9f80-tag-only-versioning-to-work-with-branch.html"><strong aria-hidden="true">20.7.27.</strong> 22f9f80 fix: tag-only versioning to work with branch protection rules</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/8e0557a-add-actions-based-pages-deploy-release-pipeline.html"><strong aria-hidden="true">20.7.28.</strong> 8e0557a feat: add Actions-based Pages deploy, release pipeline, and auto-versioning</a></span></li><li class="chapter-item expanded "><span class="chapter-link-wrapper"><a href="wiki/32f9d15-add-dabs-template-composition-model.html"><strong aria-hidden="true">20.7.29.</strong> 32f9d15 feat: add DABs template composition model</a></span></li></ol></li></ol></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split('#')[0].split('?')[0];
        if (current_page.endsWith('/')) {
            current_page += 'index.html';
        }
        const links = Array.prototype.slice.call(this.querySelectorAll('a'));
        const l = links.length;
        for (let i = 0; i < l; ++i) {
            const link = links[i];
            const href = link.getAttribute('href');
            if (href && !href.startsWith('#') && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The 'index' page is supposed to alias the first chapter in the book.
            // Check both with and without the '.html' suffix to be robust against pretty URLs
            if (link.href.replace(/\.html$/, '') === current_page.replace(/\.html$/, '')
                || i === 0
                && path_to_root === ''
                && current_page.endsWith('/index.html')) {
                link.classList.add('active');
                let parent = link.parentElement;
                while (parent) {
                    if (parent.tagName === 'LI' && parent.classList.contains('chapter-item')) {
                        parent.classList.add('expanded');
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', e => {
            if (e.target.tagName === 'A') {
                const clientRect = e.target.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                sessionStorage.setItem('sidebar-scroll-offset', clientRect.top - sidebarRect.top);
            }
        }, { passive: true });
        const sidebarScrollOffset = sessionStorage.getItem('sidebar-scroll-offset');
        sessionStorage.removeItem('sidebar-scroll-offset');
        if (sidebarScrollOffset !== null) {
            // preserve sidebar scroll position when navigating via links within sidebar
            const activeSection = this.querySelector('.active');
            if (activeSection) {
                const clientRect = activeSection.getBoundingClientRect();
                const sidebarRect = this.getBoundingClientRect();
                const currentOffset = clientRect.top - sidebarRect.top;
                this.scrollTop += currentOffset - parseFloat(sidebarScrollOffset);
            }
        } else {
            // scroll sidebar to current active section when navigating via
            // 'next/previous chapter' buttons
            const activeSection = document.querySelector('#mdbook-sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        const sidebarAnchorToggles = document.querySelectorAll('.chapter-fold-toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(el => {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define('mdbook-sidebar-scrollbox', MDBookSidebarScrollbox);


// ---------------------------------------------------------------------------
// Support for dynamically adding headers to the sidebar.

(function() {
    // This is used to detect which direction the page has scrolled since the
    // last scroll event.
    let lastKnownScrollPosition = 0;
    // This is the threshold in px from the top of the screen where it will
    // consider a header the "current" header when scrolling down.
    const defaultDownThreshold = 150;
    // Same as defaultDownThreshold, except when scrolling up.
    const defaultUpThreshold = 300;
    // The threshold is a virtual horizontal line on the screen where it
    // considers the "current" header to be above the line. The threshold is
    // modified dynamically to handle headers that are near the bottom of the
    // screen, and to slightly offset the behavior when scrolling up vs down.
    let threshold = defaultDownThreshold;
    // This is used to disable updates while scrolling. This is needed when
    // clicking the header in the sidebar, which triggers a scroll event. It
    // is somewhat finicky to detect when the scroll has finished, so this
    // uses a relatively dumb system of disabling scroll updates for a short
    // time after the click.
    let disableScroll = false;
    // Array of header elements on the page.
    let headers;
    // Array of li elements that are initially collapsed headers in the sidebar.
    // I'm not sure why eslint seems to have a false positive here.
    // eslint-disable-next-line prefer-const
    let headerToggles = [];
    // This is a debugging tool for the threshold which you can enable in the console.
    let thresholdDebug = false;

    // Updates the threshold based on the scroll position.
    function updateThreshold() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;
        const windowHeight = window.innerHeight;
        const documentHeight = document.documentElement.scrollHeight;

        // The number of pixels below the viewport, at most documentHeight.
        // This is used to push the threshold down to the bottom of the page
        // as the user scrolls towards the bottom.
        const pixelsBelow = Math.max(0, documentHeight - (scrollTop + windowHeight));
        // The number of pixels above the viewport, at least defaultDownThreshold.
        // Similar to pixelsBelow, this is used to push the threshold back towards
        // the top when reaching the top of the page.
        const pixelsAbove = Math.max(0, defaultDownThreshold - scrollTop);
        // How much the threshold should be offset once it gets close to the
        // bottom of the page.
        const bottomAdd = Math.max(0, windowHeight - pixelsBelow - defaultDownThreshold);
        let adjustedBottomAdd = bottomAdd;

        // Adjusts bottomAdd for a small document. The calculation above
        // assumes the document is at least twice the windowheight in size. If
        // it is less than that, then bottomAdd needs to be shrunk
        // proportional to the difference in size.
        if (documentHeight < windowHeight * 2) {
            const maxPixelsBelow = documentHeight - windowHeight;
            const t = 1 - pixelsBelow / Math.max(1, maxPixelsBelow);
            const clamp = Math.max(0, Math.min(1, t));
            adjustedBottomAdd *= clamp;
        }

        let scrollingDown = true;
        if (scrollTop < lastKnownScrollPosition) {
            scrollingDown = false;
        }

        if (scrollingDown) {
            // When scrolling down, move the threshold up towards the default
            // downwards threshold position. If near the bottom of the page,
            // adjustedBottomAdd will offset the threshold towards the bottom
            // of the page.
            const amountScrolledDown = scrollTop - lastKnownScrollPosition;
            const adjustedDefault = defaultDownThreshold + adjustedBottomAdd;
            threshold = Math.max(adjustedDefault, threshold - amountScrolledDown);
        } else {
            // When scrolling up, move the threshold down towards the default
            // upwards threshold position. If near the bottom of the page,
            // quickly transition the threshold back up where it normally
            // belongs.
            const amountScrolledUp = lastKnownScrollPosition - scrollTop;
            const adjustedDefault = defaultUpThreshold - pixelsAbove
                + Math.max(0, adjustedBottomAdd - defaultDownThreshold);
            threshold = Math.min(adjustedDefault, threshold + amountScrolledUp);
        }

        if (documentHeight <= windowHeight) {
            threshold = 0;
        }

        if (thresholdDebug) {
            const id = 'mdbook-threshold-debug-data';
            let data = document.getElementById(id);
            if (data === null) {
                data = document.createElement('div');
                data.id = id;
                data.style.cssText = `
                    position: fixed;
                    top: 50px;
                    right: 10px;
                    background-color: 0xeeeeee;
                    z-index: 9999;
                    pointer-events: none;
                `;
                document.body.appendChild(data);
            }
            data.innerHTML = `
                <table>
                  <tr><td>documentHeight</td><td>${documentHeight.toFixed(1)}</td></tr>
                  <tr><td>windowHeight</td><td>${windowHeight.toFixed(1)}</td></tr>
                  <tr><td>scrollTop</td><td>${scrollTop.toFixed(1)}</td></tr>
                  <tr><td>pixelsAbove</td><td>${pixelsAbove.toFixed(1)}</td></tr>
                  <tr><td>pixelsBelow</td><td>${pixelsBelow.toFixed(1)}</td></tr>
                  <tr><td>bottomAdd</td><td>${bottomAdd.toFixed(1)}</td></tr>
                  <tr><td>adjustedBottomAdd</td><td>${adjustedBottomAdd.toFixed(1)}</td></tr>
                  <tr><td>scrollingDown</td><td>${scrollingDown}</td></tr>
                  <tr><td>threshold</td><td>${threshold.toFixed(1)}</td></tr>
                </table>
            `;
            drawDebugLine();
        }

        lastKnownScrollPosition = scrollTop;
    }

    function drawDebugLine() {
        if (!document.body) {
            return;
        }
        const id = 'mdbook-threshold-debug-line';
        const existingLine = document.getElementById(id);
        if (existingLine) {
            existingLine.remove();
        }
        const line = document.createElement('div');
        line.id = id;
        line.style.cssText = `
            position: fixed;
            top: ${threshold}px;
            left: 0;
            width: 100vw;
            height: 2px;
            background-color: red;
            z-index: 9999;
            pointer-events: none;
        `;
        document.body.appendChild(line);
    }

    function mdbookEnableThresholdDebug() {
        thresholdDebug = true;
        updateThreshold();
        drawDebugLine();
    }

    window.mdbookEnableThresholdDebug = mdbookEnableThresholdDebug;

    // Updates which headers in the sidebar should be expanded. If the current
    // header is inside a collapsed group, then it, and all its parents should
    // be expanded.
    function updateHeaderExpanded(currentA) {
        // Add expanded to all header-item li ancestors.
        let current = currentA.parentElement;
        while (current) {
            if (current.tagName === 'LI' && current.classList.contains('header-item')) {
                current.classList.add('expanded');
            }
            current = current.parentElement;
        }
    }

    // Updates which header is marked as the "current" header in the sidebar.
    // This is done with a virtual Y threshold, where headers at or below
    // that line will be considered the current one.
    function updateCurrentHeader() {
        if (!headers || !headers.length) {
            return;
        }

        // Reset the classes, which will be rebuilt below.
        const els = document.getElementsByClassName('current-header');
        for (const el of els) {
            el.classList.remove('current-header');
        }
        for (const toggle of headerToggles) {
            toggle.classList.remove('expanded');
        }

        // Find the last header that is above the threshold.
        let lastHeader = null;
        for (const header of headers) {
            const rect = header.getBoundingClientRect();
            if (rect.top <= threshold) {
                lastHeader = header;
            } else {
                break;
            }
        }
        if (lastHeader === null) {
            lastHeader = headers[0];
            const rect = lastHeader.getBoundingClientRect();
            const windowHeight = window.innerHeight;
            if (rect.top >= windowHeight) {
                return;
            }
        }

        // Get the anchor in the summary.
        const href = '#' + lastHeader.id;
        const a = [...document.querySelectorAll('.header-in-summary')]
            .find(element => element.getAttribute('href') === href);
        if (!a) {
            return;
        }

        a.classList.add('current-header');

        updateHeaderExpanded(a);
    }

    // Updates which header is "current" based on the threshold line.
    function reloadCurrentHeader() {
        if (disableScroll) {
            return;
        }
        updateThreshold();
        updateCurrentHeader();
    }


    // When clicking on a header in the sidebar, this adjusts the threshold so
    // that it is located next to the header. This is so that header becomes
    // "current".
    function headerThresholdClick(event) {
        // See disableScroll description why this is done.
        disableScroll = true;
        setTimeout(() => {
            disableScroll = false;
        }, 100);
        // requestAnimationFrame is used to delay the update of the "current"
        // header until after the scroll is done, and the header is in the new
        // position.
        requestAnimationFrame(() => {
            requestAnimationFrame(() => {
                // Closest is needed because if it has child elements like <code>.
                const a = event.target.closest('a');
                const href = a.getAttribute('href');
                const targetId = href.substring(1);
                const targetElement = document.getElementById(targetId);
                if (targetElement) {
                    threshold = targetElement.getBoundingClientRect().bottom;
                    updateCurrentHeader();
                }
            });
        });
    }

    // Takes the nodes from the given head and copies them over to the
    // destination, along with some filtering.
    function filterHeader(source, dest) {
        const clone = source.cloneNode(true);
        clone.querySelectorAll('mark').forEach(mark => {
            mark.replaceWith(...mark.childNodes);
        });
        dest.append(...clone.childNodes);
    }

    // Scans page for headers and adds them to the sidebar.
    document.addEventListener('DOMContentLoaded', function() {
        const activeSection = document.querySelector('#mdbook-sidebar .active');
        if (activeSection === null) {
            return;
        }

        const main = document.getElementsByTagName('main')[0];
        headers = Array.from(main.querySelectorAll('h2, h3, h4, h5, h6'))
            .filter(h => h.id !== '' && h.children.length && h.children[0].tagName === 'A');

        if (headers.length === 0) {
            return;
        }

        // Build a tree of headers in the sidebar.

        const stack = [];

        const firstLevel = parseInt(headers[0].tagName.charAt(1));
        for (let i = 1; i < firstLevel; i++) {
            const ol = document.createElement('ol');
            ol.classList.add('section');
            if (stack.length > 0) {
                stack[stack.length - 1].ol.appendChild(ol);
            }
            stack.push({level: i + 1, ol: ol});
        }

        // The level where it will start folding deeply nested headers.
        const foldLevel = 3;

        for (let i = 0; i < headers.length; i++) {
            const header = headers[i];
            const level = parseInt(header.tagName.charAt(1));

            const currentLevel = stack[stack.length - 1].level;
            if (level > currentLevel) {
                // Begin nesting to this level.
                for (let nextLevel = currentLevel + 1; nextLevel <= level; nextLevel++) {
                    const ol = document.createElement('ol');
                    ol.classList.add('section');
                    const last = stack[stack.length - 1];
                    const lastChild = last.ol.lastChild;
                    // Handle the case where jumping more than one nesting
                    // level, which doesn't have a list item to place this new
                    // list inside of.
                    if (lastChild) {
                        lastChild.appendChild(ol);
                    } else {
                        last.ol.appendChild(ol);
                    }
                    stack.push({level: nextLevel, ol: ol});
                }
            } else if (level < currentLevel) {
                while (stack.length > 1 && stack[stack.length - 1].level > level) {
                    stack.pop();
                }
            }

            const li = document.createElement('li');
            li.classList.add('header-item');
            li.classList.add('expanded');
            if (level < foldLevel) {
                li.classList.add('expanded');
            }
            const span = document.createElement('span');
            span.classList.add('chapter-link-wrapper');
            const a = document.createElement('a');
            span.appendChild(a);
            a.href = '#' + header.id;
            a.classList.add('header-in-summary');
            filterHeader(header.children[0], a);
            a.addEventListener('click', headerThresholdClick);
            const nextHeader = headers[i + 1];
            if (nextHeader !== undefined) {
                const nextLevel = parseInt(nextHeader.tagName.charAt(1));
                if (nextLevel > level && level >= foldLevel) {
                    const toggle = document.createElement('a');
                    toggle.classList.add('chapter-fold-toggle');
                    toggle.classList.add('header-toggle');
                    toggle.addEventListener('click', () => {
                        li.classList.toggle('expanded');
                    });
                    const toggleDiv = document.createElement('div');
                    toggleDiv.textContent = '❱';
                    toggle.appendChild(toggleDiv);
                    span.appendChild(toggle);
                    headerToggles.push(li);
                }
            }
            li.appendChild(span);

            const currentParent = stack[stack.length - 1];
            currentParent.ol.appendChild(li);
        }

        const onThisPage = document.createElement('div');
        onThisPage.classList.add('on-this-page');
        onThisPage.append(stack[0].ol);
        const activeItemSpan = activeSection.parentElement;
        activeItemSpan.after(onThisPage);
    });

    document.addEventListener('DOMContentLoaded', reloadCurrentHeader);
    document.addEventListener('scroll', reloadCurrentHeader, { passive: true });
})();

