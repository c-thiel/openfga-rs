# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/c-thiel/openfga-rs/compare/v0.1.0...v0.2.0) - 2024-09-29

### Added

- Add caching to release workflow
- Add SBOM generation and dependency caching
- Initial commit, building lib successfully

### Fixed

- Cargo config in the right place
- Fix check workflow
- Set Dependabot to auto rebase
- Remove debugging from release workflow
- Add release-plz configuration to allow dirty publishing
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

- Merge commit 'cea99dbcc52c94d384eecfb1e1ef0487fd959ce1'
- Update tonic
- Add new_initialized
- fmt
- Better docs
- Add interceptors
- Keep protobuf files for diff
- *(deps)* Bump serde from 1.0.202 to 1.0.203
- *(deps)* Bump serde from 1.0.201 to 1.0.202
- *(deps)* Bump prost-build from 0.12.4 to 0.12.6
- *(deps)* Bump prost-types from 0.12.4 to 0.12.6
- *(deps)* Bump prost from 0.12.4 to 0.12.6
- *(deps)* Bump serde from 1.0.198 to 1.0.201
- *(deps)* Bump serde from 1.0.197 to 1.0.198
- *(deps)* Bump prost-wkt-types from 0.5.0 to 0.5.1
- *(deps)* Bump prost-wkt-build from 0.5.0 to 0.5.1
- *(deps)* Bump prost-wkt from 0.5.0 to 0.5.1
- Add Getting Started to readme
- *(deps)* Bump prost-build from 0.12.3 to 0.12.4
- Remove branch filter for triggering Check workflow
