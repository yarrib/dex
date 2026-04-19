# Changelog

All notable changes to dex are documented here.

## [Unreleased]

### Bug Fixes

- **context-map**: Populate tasks from scaffolded dex.toml (#48) ([`a25d62d`](https://github.com/yarrib/dex/commit/a25d62d3aa99f3359451d0c25085c443e728cd9b))

### Documentation

- Add SCOPE.md — product scope guardrails and decision filter (#44) ([`7062c81`](https://github.com/yarrib/dex/commit/7062c8110a8089779338bc94e50c366510cae062))
- Add PRD for dex-in-browser WASM feature (#45) ([`7b2cd91`](https://github.com/yarrib/dex/commit/7b2cd91e1df060418ebe2bdf71bab8fbe05beaeb))
- Add scaffolding differentiation PRD (#46) ([`a1dda0c`](https://github.com/yarrib/dex/commit/a1dda0c1ab88c9fee27ef43d8f091cf7bd88ad24))
- Add PRD for org-validated skills & MCP server catalog (#52) ([`c04db4f`](https://github.com/yarrib/dex/commit/c04db4f60a79037e6fcb1d2b7a315fa25d071ae9))

### Features

- **cli**: Add dex templates list/show (#40) ([`0752c04`](https://github.com/yarrib/dex/commit/0752c04b9dc8546ea307d6a07fb50a76a185b73f))
- **templates**: Add databricks-app-streamlit template (#41) ([`6e3f963`](https://github.com/yarrib/dex/commit/6e3f9637d9c088f8c4edb6f9bd49464c82278a7f))
- **templates**: Add databricks-app-streamlit template (#42) ([`2239358`](https://github.com/yarrib/dex/commit/22393585b8702fb9b3fae6aa10583aad5ff78671))
- **templates**: Add dabs-dashboard template (#43) ([`2eeb5a0`](https://github.com/yarrib/dex/commit/2eeb5a054719ac42872159db089bc3b7a5495777))
- Add next.js template, notebook trait, and context-map generation (#47) ([`95dba9b`](https://github.com/yarrib/dex/commit/95dba9be5309684aeb031421c1040cf1dbc63c8a))
- **scaffold**: Add post-scaffold activation hook (on_success) (#49) ([`d712d89`](https://github.com/yarrib/dex/commit/d712d896a1cc207b5f4108bb055cfb3e47c91e26))
- **templates**: Add conditional variable visibility via `when` field (#50) ([`376c137`](https://github.com/yarrib/dex/commit/376c137f163d82b0818fe3f2bfcbadc594c27511))
- **mcp**: Complete MCP v0.2 — integration tests and variable annotation (#51) ([`723c342`](https://github.com/yarrib/dex/commit/723c342d180f0080fa012199955962f5154e28c1))
- **agent**: Batteries-included, assistant-agnostic agent scaffolding (#53) ([`7f7b8e0`](https://github.com/yarrib/dex/commit/7f7b8e0474e26484e135c457e7acbe0823adeed6))

## [0.2.0] — 2026-04-02

### Bug Fixes

- **release**: Use cross for Linux musl targets ([`bb22702`](https://github.com/yarrib/dex/commit/bb227029828d164b138a89b1e1f7b4924af6373e))

### Chores

- Bump version to v0.2.0 (#39) ([`1078c43`](https://github.com/yarrib/dex/commit/1078c43543b5b791d78cdb59222e17c35463410e))

### Documentation

- Add PRD for AI-ready scaffolding (context map, traits, WASM) (#34) ([`6cbcd49`](https://github.com/yarrib/dex/commit/6cbcd49ef677c2a524019af955ae74aeb773602d))
- Add PRD for Snowflake templates (#35) ([`b8fa631`](https://github.com/yarrib/dex/commit/b8fa6316e16403f98eb0578be354efce9129f904))

### Features

- **skills**: Add dex skills system — agent skill pack management (#36) ([`2f1e593`](https://github.com/yarrib/dex/commit/2f1e593e2ff08249a1b8b397af7622e5084c0f47))
- **mcp**: Implement scaffold_agent tool and add .mcp.json (#37) ([`3035344`](https://github.com/yarrib/dex/commit/3035344e7063c234bec62bca2ffe9d1b115c75f7))
- **devcontainer**: Ai-dev-kit integration with profile-based skill setup (#38) ([`4535cd0`](https://github.com/yarrib/dex/commit/4535cd05e9d32a3228f3a0ef450e72471b2e5634))

## [0.1.1] — 2026-04-01

### Bug Fixes

- **release**: Align install.sh artifact names with release.yml and add linux aarch64 target (#30) ([`a378387`](https://github.com/yarrib/dex/commit/a378387df248fe7c905e5be001ad617b457854e0))
- **release**: Support workflow_dispatch and fix first-release changelog (#31) ([`5da5907`](https://github.com/yarrib/dex/commit/5da5907f1ba574bb3906d91ca4f3b1902553520d))
- **release**: Support workflow_dispatch and fix first-release changelog ([`b604a2d`](https://github.com/yarrib/dex/commit/b604a2d1996c915bc1f37ffd58ae150ab128f7a1))

### Chores

- Remove Python layer and expand Rust test coverage (#22) ([`f085abb`](https://github.com/yarrib/dex/commit/f085abb8a81d0b68485129b93632bca7478430ff))

### Documentation

- Migrate from MkDocs to mdBook (#23) ([`8f177a0`](https://github.com/yarrib/dex/commit/8f177a014412abd6b4523f796e9633ddc43591b4))
- Add changelog.md placeholder for mdBook build (#24) ([`1084403`](https://github.com/yarrib/dex/commit/10844036b12f96dcf4d2187cca052ef4378ae0b2))
- Rewrite all docs for Rust binary architecture (#25) ([`e3efd53`](https://github.com/yarrib/dex/commit/e3efd53907298c081ed4f9dbd18be0c7fd25d434))

### Features

- **templates**: Inline variables format, order field, and standards pre-fill (#20) ([`7719579`](https://github.com/yarrib/dex/commit/77195797fa78b5242d945be4c1a1b0f36feb437c))
- Port dex to pure Rust single binary (#21) ([`30690b9`](https://github.com/yarrib/dex/commit/30690b9633c05da429f7e01eb1ec5dcb0a2b28ef))
- Add web-based project scaffolding app (#26) ([`a6ab3a6`](https://github.com/yarrib/dex/commit/a6ab3a605669d72d46436be5b841991f04f32154))
- **cli**: Add dex run <task> command (#28) ([`bacd088`](https://github.com/yarrib/dex/commit/bacd088bffd2102325b33e2944b87a8ff6ecb050))
- **templates**: Add python-package template (#29) ([`dfdc0e9`](https://github.com/yarrib/dex/commit/dfdc0e90d1bd040c3433e8aa8ecdaff0f6d845de))

### Testing

- **core**: Add regression tests for embedded template variable and file loading (#27) ([`bf5f8b5`](https://github.com/yarrib/dex/commit/bf5f8b53942e498e7555a273491ed43fd2201a17))

## [0.1.0] — 2026-03-10

### Bug Fixes

- **release**: Fix bump-version idempotency and replace git-cliff Docker action ([`b5d760e`](https://github.com/yarrib/dex/commit/b5d760e8b5a39c6edf6c03a7e23fcd4c285e826d))
- **release**: Remove redundant version stamp step in build jobs ([`18f3db6`](https://github.com/yarrib/dex/commit/18f3db6176ef39f9146f4bbbe263273f3c66a2fd))
- **docs**: Resolve gh-pages deploy alias conflict (#18) ([`439426a`](https://github.com/yarrib/dex/commit/439426a45e6e0aa4ef07bd327f4741138ba765eb))
- **docs**: Add mike set-default to create root redirect (#19) ([`d8715ff`](https://github.com/yarrib/dex/commit/d8715ff07f221c05956cdeb4954a5812e67eba66))
- **docs**: Use latest as version name, deploy numbered versions on tags ([`0c7c4f8`](https://github.com/yarrib/dex/commit/0c7c4f812d251b4f89353eb3a7f58aec3ab8283a))
- **docs**: Suppress MkDocs 2.0 compatibility warning ([`0dcf505`](https://github.com/yarrib/dex/commit/0dcf50529e8d6ec07c44b9c7247081b11d6e13b8))
- **docs**: Move NO_MKDOCS_2_WARNING to job level, delete version before deploy ([`bfeb2a5`](https://github.com/yarrib/dex/commit/bfeb2a55ca567aa6b21727b6e93a45d3f47f8dc5))

### Chores

- **docs**: Add workflow_dispatch to docs deploy workflow ([`0e067c2`](https://github.com/yarrib/dex/commit/0e067c2e0da86e913d5d893238eef01320e60570))
- **release**: Remove auto-version workflow, add release guide (#13) ([`1f9f2e9`](https://github.com/yarrib/dex/commit/1f9f2e9837be7cd813ef3df2ab70cdba9ed8cb16))
- Release v0.1.0 (#17) ([`7b9ade0`](https://github.com/yarrib/dex/commit/7b9ade03d7be3d2f6391fa2cb69712decebd861b))

### Documentation

- Add quickstart, mcp serve page, versioning guide, and template … (#11) ([`d7b6caf`](https://github.com/yarrib/dex/commit/d7b6caf5248af05323f000bed9dbf07f25b7358b))


