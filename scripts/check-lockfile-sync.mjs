import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const pkgPath = path.join(rootDir, 'package.json');
const lockPath = path.join(rootDir, 'package-lock.json');

if (!fs.existsSync(pkgPath)) {
  console.error('❌ package.json not found');
  process.exit(1);
}
if (!fs.existsSync(lockPath)) {
  console.error('❌ package-lock.json not found');
  process.exit(1);
}

const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
const lock = JSON.parse(fs.readFileSync(lockPath, 'utf-8'));
const lockRoot = lock.packages ? lock.packages[''] : {};

let mismatches = [];

const checkSection = (name) => {
  const pkgDeps = pkg[name] || {};
  const lockDeps = (lockRoot && lockRoot[name]) || {};

  for (const [dep, spec] of Object.entries(pkgDeps)) {
    if (!lockDeps[dep]) {
      mismatches.push(`${name}['${dep}']: missing in package-lock.json (expected '${spec}')`);
    } else if (lockDeps[dep] !== spec) {
      mismatches.push(`${name}['${dep}']: '${spec}' in package.json does not match '${lockDeps[dep]}' in package-lock.json`);
    }
  }

  for (const dep of Object.keys(lockDeps)) {
    if (!pkgDeps[dep]) {
      mismatches.push(`${name}['${dep}']: present in package-lock.json but removed from package.json`);
    }
  }
};

checkSection('dependencies');
checkSection('devDependencies');

if (mismatches.length > 0) {
  console.error('❌ Lockfile Sync Check Failed:');
  for (const m of mismatches) {
    console.error(`  - ${m}`);
  }
  console.error('\nPlease run `npm install` to synchronize package-lock.json with package.json.');
  process.exit(1);
}

try {
  execSync('npm ls --package-lock-only', { cwd: rootDir, stdio: 'pipe' });
} catch {
  console.error('❌ Lockfile integrity check failed (`npm ls --package-lock-only` reported errors).');
  console.error('Please run `npm install` to update your lockfile before committing.');
  process.exit(1);
}

console.log('✓ Lockfile is in sync with package.json.');
