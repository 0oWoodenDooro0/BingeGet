import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

let errors = [];

// 1. Check index.html for <meta name="color-scheme" content="dark" />
const indexPath = path.join(rootDir, 'index.html');
if (fs.existsSync(indexPath)) {
  const indexContent = fs.readFileSync(indexPath, 'utf-8');
  if (!/<meta\s+[^>]*name=["']color-scheme["'][^>]*content=["']dark["']/i.test(indexContent)) {
    errors.push('index.html is missing <meta name="color-scheme" content="dark" />');
  }
} else {
  errors.push('index.html not found');
}

// 2. Check main.css for color-scheme: dark
const cssPath = path.join(rootDir, 'src/styles/main.css');
if (fs.existsSync(cssPath)) {
  const cssContent = fs.readFileSync(cssPath, 'utf-8');
  if (!/color-scheme\s*:\s*dark/i.test(cssContent)) {
    errors.push('src/styles/main.css is missing color-scheme: dark definition');
  }
} else {
  errors.push('src/styles/main.css not found');
}

// 3. Check Vue components for <option> dark styling and placeholder conventions
const componentsDir = path.join(rootDir, 'src/components');
if (fs.existsSync(componentsDir)) {
  const files = fs.readdirSync(componentsDir).filter(f => f.endsWith('.vue'));
  for (const file of files) {
    const fullPath = path.join(componentsDir, file);
    const content = fs.readFileSync(fullPath, 'utf-8');

    // 3a. <option> styling
    const optionMatches = content.match(/<option[^>]*>/g) || [];
    for (const tag of optionMatches) {
      if (!/class=["'][^"']*(bg-slate-|dark:)[^"']*["']/.test(tag)) {
        errors.push(`${file} contains <option> without explicit dark background class (e.g. bg-slate-900): ${tag}`);
      }
    }

    // 3b. Placeholder convention: avoid verbose prefixes like "例如：", "例如:", "e.g."
    const placeholderMatches = content.match(/placeholder=["'][^"']*["']/g) || [];
    for (const tag of placeholderMatches) {
      if (/placeholder=["'](例如[:：]|e\.g\.)/i.test(tag)) {
        errors.push(`${file} contains verbose placeholder prefix (avoid '例如：' / 'e.g.'): ${tag}`);
      }
    }
  }
}

if (errors.length > 0) {
  let currentBranch = '';
  try {
    currentBranch = execSync('git rev-parse --abbrev-ref HEAD', { encoding: 'utf-8' }).trim();
  } catch {}
  const isMaster = currentBranch === 'master' || currentBranch === 'master-retro' || process.env.GITHUB_REF === 'refs/heads/master';

  if (isMaster && process.env.STRICT_GUARDRAILS !== 'true') {
    console.warn('⚠️  UI & Theme Guardrail Notice (Pending PR merge on master):');
    for (const err of errors) {
      console.warn(`  - ${err}`);
    }
  } else {
    console.error('❌ UI & Theme Guardrail Check Failed:');
    for (const err of errors) {
      console.error(`  - ${err}`);
    }
    process.exit(1);
  }
} else {
  console.log('✓ UI and theme guardrails verified: color-scheme, option styling, and placeholder conventions.');
}
