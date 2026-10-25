import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const srcDir = path.join(rootDir, 'src');

const errors = [];

function walkDir(dir) {
  const results = [];
  const list = fs.readdirSync(dir);
  for (const file of list) {
    const fullPath = path.join(dir, file);
    const stat = fs.statSync(fullPath);
    if (stat && stat.isDirectory()) {
      results.push(...walkDir(fullPath));
    } else if (file.endsWith('.ts') || file.endsWith('.vue')) {
      results.push(fullPath);
    }
  }
  return results;
}

if (fs.existsSync(srcDir)) {
  const files = walkDir(srcDir);
  for (const filePath of files) {
    const relPath = path.relative(rootDir, filePath);
    const content = fs.readFileSync(filePath, 'utf-8');

    // 1. Check for banned domain synonym "線路" (should use "頻道")
    if (content.includes('線路')) {
      errors.push(`${relPath} contains prohibited domain synonym '線路'. Use canonical term '頻道' per GLOSSARY.md.`);
    }

    // 2. Check for hardcoded scraping site domains in frontend code
    // Referer and site scraping logic must be routed through SiteExtractor in backend per AGENTS.md
    const bannedDomains = ['chinaq.fun', 'chinaq.at', 'bfikuncdn.com'];
    for (const domain of bannedDomains) {
      if (content.includes(domain)) {
        errors.push(`${relPath} contains hardcoded site domain '${domain}'. Site URLs and referer logic belong to SiteExtractor in backend per AGENTS.md.`);
      }
    }
  }
} else {
  errors.push('src directory not found');
}

if (errors.length > 0) {
  console.error('❌ Domain & Architecture Guardrail Check Failed:');
  for (const err of errors) {
    console.error(`  - ${err}`);
  }
  process.exit(1);
} else {
  console.log('✓ Domain & architecture guardrails verified: canonical terminology and extractor decoupling.');
}
