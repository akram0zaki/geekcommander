# Contributing to Geek Commander

Thank you for your interest in contributing to Geek Commander! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## How to Contribute

### Reporting Bugs

If you find a bug in the library, please create an issue on GitHub with the following information:

1. A clear, descriptive title
2. A detailed description of the issue
3. Steps to reproduce the bug
4. Expected behavior
5. Actual behavior
6. Any relevant logs or error messages
7. Your environment (OS, Rust version, Cargo version, etc.)

### Suggesting Enhancements

We welcome suggestions for enhancements to the library. Please create an issue on GitHub with the following information:

1. A clear, descriptive title
2. A detailed description of the enhancement
3. Why this enhancement would be useful
4. Any relevant examples or use cases

### Pull Requests

1. Fork the repository
2. Create a new branch for your changes
3. Make your changes
4. Add or update tests as necessary
5. Update documentation as necessary
6. Run the tests to ensure they pass
7. Submit a pull request

## Development Setup

1. Clone the repository
2. Ensure you have Rust installed (preferably via [rustup](https://rustup.rs/))
3. Build the project: `cargo build`
4. Run the tests: `cargo test`
5. For development with all features: `cargo build --all-features`

## Coding Standards

- Format code using `rustfmt`: `cargo fmt`
- Run Clippy for linting: `cargo clippy -- -D warnings`
- Write documentation comments using `///` for public APIs
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Write comprehensive tests for new functionality
- Use `cargo doc` to generate and verify documentation

## Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers in commit messages where applicable

## Versioning

This project follows [Semantic Versioning](https://semver.org/). Please ensure that version numbers are updated appropriately in the following files:

- `Cargo.toml`
- `CHANGELOG.md`

## Documentation

- Update the documentation for any changes to the API
- Add examples for new features using `///` doc comments
- Include usage examples in your documentation
- Ensure that `cargo doc` builds correctly
- Consider adding examples to the `examples/` directory for complex features

## Testing

- Add tests for new functionality using `#[cfg(test)]` modules
- Write both unit tests and integration tests where appropriate
- Use `cargo test` to run all tests
- Aim for high test coverage
- Consider using `cargo test --doc` to test documentation examples
- Use `cargo bench` for performance-critical code

## Code Quality Tools

Before submitting a pull request, please ensure:

- `cargo fmt` - Code is properly formatted
- `cargo clippy` - No linting warnings
- `cargo test` - All tests pass
- `cargo doc` - Documentation builds without warnings

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT License.

## Contact

If you have any questions or need help with contributing, please contact the project maintainer:

- Akram Zaki (azpythonprojects@gmail.com) 