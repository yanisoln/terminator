#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read the workspace Cargo.toml
const workspaceCargoPath = path.join(__dirname, '../../Cargo.toml');
const cargoContent = fs.readFileSync(workspaceCargoPath, 'utf8');

// Extract version from Cargo.toml
const versionMatch = cargoContent.match(/^version\s*=\s*"([^"]+)"/m);
if (!versionMatch) {
    console.error('Could not find version in workspace Cargo.toml');
    process.exit(1);
}

const version = versionMatch[1];
console.log(`Found version: ${version}`);

// Read package.json
const packagePath = path.join(__dirname, 'package.json');
const packageContent = JSON.parse(fs.readFileSync(packagePath, 'utf8'));

// Update version and optionalDependencies
packageContent.version = version;

// Update optionalDependencies to use the same version
if (packageContent.optionalDependencies) {
    for (const dep in packageContent.optionalDependencies) {
        if (dep.startsWith('terminator.js-')) {
            packageContent.optionalDependencies[dep] = version;
        }
    }
}

// Write back to package.json
fs.writeFileSync(packagePath, JSON.stringify(packageContent, null, 2) + '\n');

console.log(`Updated package.json version to: ${version}`); 