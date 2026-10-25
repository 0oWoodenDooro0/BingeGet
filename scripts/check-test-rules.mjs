import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const testsDir = path.join(rootDir, 'tests');

// Rule: Strictly avoid asserting static copy or template labels in component tests.
// Ref: AGENTS.md -> Development & Testing Rules -> Behavioral Tests Only

const FORBIDDEN_CJK_REGEX = /expect\([^)]*\.text\(\)\)\.(?:toMatch|toContain|toBe)\([^)]*[\u4e00-\u9fa5]/;

function scanDir(dir, fileList = []) {
  if (!fs.existsSync(dir)) return fileList;
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      scanDir(fullPath, fileList);
    } else if (entry.isFile() && (entry.name.endsWith('.test.ts') || entry.name.endsWith('.spec.ts'))) {
      fileList.push(fullPath);
    }
  }
  return fileList;
}

const componentTestFiles = scanDir(path.join(testsDir, 'components'));
let violations = [];

for (const filePath of componentTestFiles) {
  const content = fs.readFileSync(filePath, 'utf-8');
  const lines = content.split('\n');

  lines.forEach((line, index) => {
    // Check if asserting static CJK copy on .text()
    if (FORBIDDEN_CJK_REGEX.test(line)) {
      const relPath = path.relative(rootDir, filePath);
      violations.push({
        file: relPath,
        line: index + 1,
        code: line.trim(),
        message: 'Forbidden static copy assertion in component test. Use behavioral assertions (presence, attributes, event emissions) instead of user-facing copy.',
      });
    }
  });
}

if (violations.length > 0) {
  console.error('\n❌ Test Rule Verification Failed:');
  console.error('Violations of AGENTS.md "Development & Testing Rules: Behavioral Tests Only":\n');
  for (const v of violations) {
    console.error(`  - ${v.file}:${v.line}`);
    console.error(`    Code: ${v.code}`);
    console.error(`    Reason: ${v.message}\n`);
  }
  process.exit(1);
}

console.log('✓ All component tests comply with behavioral assertion rules.');
