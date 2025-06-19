# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wezterm-parallel - An interactive setup assistant that helps developers configure Claude Code projects with best practices, appropriate prompts, and standardized project structures.

## Commands

### Root Level Commands
```bash
npm run setup          # Run the interactive setup assistant
npm run setup:dry-run  # Preview mode without making changes
npm run setup:dev      # Development mode (run directly with ts-node)
npm run build          # Build TypeScript files
npm run test           # Run all tests with Jest
npm run lint           # Run ESLint with auto-fix
npm run type-check     # Run TypeScript type checking
```

### Scripts Directory Commands (from scripts/)
```bash
cd scripts
npm run build         # Compile TypeScript to JavaScript
npm run dev           # Run directly with ts-node
npm run test          # Run Jest tests for lib/**/*.test.ts
npm run lint          # ESLint with auto-fix
npm run type-check    # TypeScript validation only
```

### Running Single Tests
```bash
# Run a specific test file
npm test -- lib/fileManager.test.ts

# Run tests matching a pattern
npm test -- --testNamePattern="should create backup"

# Run tests with coverage for a specific file
npm test -- --coverage lib/templateProcessor.test.ts
```

## Architecture

### Setup Assistant Flow
```
[CLI Entry] → [Prompt Selection] → [User Input] → [Template Processing] → [File Generation]
     ↓              ↓                    ↓                ↓                      ↓
setup-assistant  promptSelector      inquirer      templateProcessor      fileManager
```

### Core Components

**scripts/setup-assistant.ts**
- Main entry point that orchestrates the setup flow
- Handles command-line arguments and mode selection
- Integrates all components to create the final project structure

**scripts/lib/promptSelector.ts**
- Analyzes user requirements through interactive questions
- Recommends appropriate development prompts based on project type
- Maps user choices to specific prompt templates

**scripts/lib/templateProcessor.ts**
- Processes Mustache templates with user-provided variables
- Handles template loading and variable substitution
- Generates customized content for project files

**scripts/lib/fileManager.ts**
- Manages file operations with backup capabilities
- Creates directory structures and writes processed templates
- Handles dry-run mode for preview without changes

**scripts/lib/validator.ts**
- Validates user inputs (project names, emails, URLs)
- Ensures data integrity before template processing
- Provides validation rules for different input types

### Template System

The project uses Mustache templates stored in `prompts/templates/`:
- Templates use `{{variable}}` syntax for substitution
- Variables are collected through the interactive CLI
- Processed templates become project files (README.md, CLAUDE.md, etc.)

### Development Workflow Integration

This starter kit enforces specific development practices:
1. **Git Worktree Usage**: Features must be developed in isolated worktrees
2. **Progress Tracking**: PROGRESS.md and DEVELOPMENT_ROADMAP.md must be updated
3. **Test-First Development**: Tests must be written before implementation
4. **Quality Checks**: All code must pass lint, type-check, and tests before commit

### Prompt Selection Logic

The assistant recommends prompts based on:
- Project type (startup, enterprise, open-source, etc.)
- Team size and development methodology
- Required features and compliance needs
- Development philosophy preferences

Each prompt type (`basic`, `enterprise`, `opensource`, `startup`) includes:
- Customized CLAUDE.md with specific guidelines
- Appropriate development workflows
- Relevant documentation templates
- Security and compliance considerations