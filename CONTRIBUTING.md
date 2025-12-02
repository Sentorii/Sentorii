# Contributing to Sentorii 🦊

## Setting Up Your Development Environment
Sentorii makes use of devcontainer to set up the required toolchain.

## Development Workflow

We use the **GitFlow** methodology to develop Sentorii itself. This means all new development happens on branches off of `develop`.

1.  **Create an Issue:** All work should start with a GitHub Issue that describes the bug or feature.
2.  **Create a Feature Branch:** Branch off the `develop` branch.
    ```bash
    git checkout develop
    git pull origin develop
    git checkout -b feat/123-my-new-feature
    ```
3.  **Write Code & Tests:** Make your changes. Remember to add or update tests to cover your changes! Our project has two types of tests:
    * **Unit tests**: Fast, in-memory tests that have no external dependencies.
    * **Integration tests**: Slower tests that require external dependencies (like `git`) and are marked with a feature flag.
4.  **Follow Coding Standards:** To make quality checks easy and consistent, we use the `just` command runner.
    <br>**Before submitting a Pull Request, please run the master check command from the root of the repository:**
    ```bash
    # This single command will format, lint, and run the complete test suite.
    just check
    ```
5.  **Commit Your Changes:** We use the [Conventional Commits](https://www.conventionalcommits.org/) standard for our commit messages.
6.  **Submit a Pull Request:** Push your branch to your fork and open a pull request against the `develop` branch of the main repository. Please fill out the PR template.

## Coding Standards

*   **Formatting:** All code must be formatted with `rustfmt`.
*   **Linting:** All code must pass `cargo clippy --workspace -- -D warnings`.
*   **Error Handling:** We use `anyhow` for the main binary and `thiserror` for library crates. `unwrap()` and `expect()` are not permitted in application code.
*   **Documentation:** All public APIs in our library crates must have comprehensive doc-tests.