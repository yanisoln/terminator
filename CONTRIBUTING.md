# Contributing to Terminator

This document explains how to publish new versions of Terminator components.

## Release Process

### TypeScript SDK

1. Update the version in `ts-sdk/package.json`:
   ```json
   {
     "version": "1.0.6"  // Increment this version number
   }
   ```

2. Build the TypeScript SDK:
   ```bash
   cd ts-sdk
   npm run build
   ```

3. Publish to npm:
   ```bash
   npm publish
   ```

### Python SDK

1. Update the version in `python-sdk/pyproject.toml`:
   ```toml
   [project]
   version = "0.1.3"  # Increment this version number
   ```

2. Build the distribution files:
   ```bash
   cd python-sdk
   python -m build
   ```

3. Publish to PyPI:
   ```bash
   python -m twine upload dist/*
   ```
   You'll need to enter your PyPI credentials when prompted.

### Server Release

The server release process is automated through GitHub Actions. To create a new release:

1. Create and push a new version tag:
   ```bash
   git tag v1.0.0  # Use appropriate version number
   git push origin v1.0.0
   ```

This will automatically:
- Build the Windows server executable
- Create a GitHub release
- Package the executable as `terminator-server-windows-x86_64.zip`
- Upload the package to the GitHub release

Alternatively, you can manually trigger the release process:
1. Go to the GitHub Actions tab
2. Select the "Build & Release Windows Server" workflow
3. Click "Run workflow"

## Versioning Guidelines

- Follow semantic versioning (MAJOR.MINOR.PATCH)
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes

## Release Checklist

Before publishing any component:
1. Update version numbers in all relevant files
2. Ensure all tests pass
3. Update documentation if necessary
4. Create a git commit with your changes
5. Follow the release process for each component
6. Verify the release on the respective package registries (npm, PyPI, GitHub Releases) 