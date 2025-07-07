# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- CI/CD pipeline with GitHub Actions
- Automated testing and quality checks
- Security auditing and dependency management
- Documentation generation and deployment
- Code coverage reporting
- Multi-platform release automation

### Changed
- Improved error handling and logging
- Enhanced test coverage (251 tests)

### Fixed
- End-to-end integration test timeout issues
- Compiler warnings resolution

## [0.3.0] - 2025-07-07

### Added
- Issue #44: Unified logging system migration (131 log calls migrated)
- Structured logging with LogContext and metadata
- Component-based logging separation (system, ipc, config, performance, dashboard, error_recovery)
- Enhanced error tracing and operational visibility

### Changed
- Migrated from traditional logging to structured logging system
- Improved log quality with entity tracking and metadata
- Enhanced debugging capabilities with contextual information

### Fixed
- All compiler warnings resolved
- Test stability improvements (251/251 tests passing)
- Enhanced system reliability

## [0.2.0] - 2025-07-04

### Added
- Issue #43: Comprehensive error handling improvements
- Issue #41: Integration test quality assurance
- Safe error handling macros and helper functions
- Improved process communication error handling
- Enhanced file operation safety

### Changed
- Removed dangerous unwrap() calls from codebase
- Improved error recovery mechanisms
- Enhanced system stability and fault tolerance

### Fixed
- Integration test failures (39/39 tests now passing)
- File synchronization edge cases
- macOS path normalization issues
- Performance monitoring accuracy

## [0.1.0] - 2025-06-29

### Added
- Issue #10: Claude Code automatic startup functionality (Phases 1-6)
- Binary detection and configuration management
- Health monitoring and logging systems
- Workspace integration with process management
- Comprehensive integration tests (22 new tests)

### Changed
- Enhanced workspace management with Claude Code integration
- Improved process lifecycle management
- Better resource management and monitoring

## [0.0.3] - 2025-06-27

### Added
- Issue #17: Claude Code multi-process coordination system
- Issue #18: WezTerm pane splitting and layout templates
- Process coordination with load balancing
- Task distribution and dependency management
- File synchronization and conflict resolution
- YAML template system with dynamic layouts

### Changed
- Enhanced process management architecture
- Improved multi-process communication
- Better task management and distribution

## [0.0.2] - 2025-06-25

### Added
- Basic workspace management system
- Process management with monitoring
- WebSocket dashboard for real-time updates
- Task management with kanban board UI
- WezTerm Lua integration (3,239 lines)
- Comprehensive test suite (127 tests)

### Changed
- Improved architecture with proper separation of concerns
- Enhanced IPC communication system
- Better configuration management

## [0.0.1] - 2025-06-20

### Added
- Initial project setup and architecture
- Basic Rust framework structure
- Core modules: workspace, process, config
- Unix Domain Socket IPC implementation
- Basic WezTerm integration
- Project documentation and planning

### Technical Details
- Rust backend: 19,335 lines of code
- Lua integration: 7,175 lines of code
- Testing framework: 251 tests (100% passing)
- Multi-platform support: Linux, macOS, Windows
- Performance optimized: <2s startup time, <30MB memory usage

## Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for information on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.