# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/c-thiel/openfga-rs/compare/v0.1.0...v0.1.1) - 2025-02-20

### Added

- Less strict version pinning
- Add caching to release workflow
- Add SBOM generation and dependency caching
- Initial commit, building lib successfully

### Fixed

- vendored-protoc flag for prost-wkt-types
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

- Remove authenticators
- add "tls-native-roots"
- Update tokio
- Optional vendored protoc
- remove protoc dependency
- Remove native tls dep
- Expose CredentialRefreshError
- Squashed commit of the following:
- Squashed commit of the following:
- *(deps)* Bump prost-types from 0.12.6 to 0.13.3
- remove prost-build dep
- Keep protobuf files for diff
- Update deps
- *(deps)* Bump serde from 1.0.203 to 1.0.210
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
