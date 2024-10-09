# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/c-thiel/openfga-rs/compare/v0.1.0...v0.2.0) - 2024-10-09

### Added

- Add caching to release workflow
- Add SBOM generation and dependency caching
- Initial commit, building lib successfully

### Fixed

- Cargo config in the right place
- Make proto dir env var
- include in Cargo.toml
- Improve error handling in build.rs
- fixup! fix: Debugging proto dir in release workflow
- Remove generated code
- Debugging proto dir in release workflow
- Add checkout to udeps job
- Remove include from config.toml
- Add github_token explicitly to buf setup action
- Include proto dir

### Other

- Expose CredentialRefreshError
- Squashed commit of the following:
- *(deps)* Bump prost-types from 0.12.6 to 0.13.3
- remove prost-build dep
- Keep protobuf files for diff
- Update deps
- Add Getting Started to readme
