/**
 * Auto-generate barrel export (index.ts) for ts-rs bindings.
 * Run: node src/types/bindings/gen-barrel.cjs
 */
const fs = require('fs');
const path = require('path');

const dir = __dirname + '\\bindings'
const files = fs.readdirSync(dir)
  .filter(f => f.endsWith('.ts') && f !== 'index.ts')
  .sort();

const lines = [
  '// Auto-generated barrel export. Do not edit manually.',
  '// Run: node src/types/bindings/gen-barrel.cjs',
  '',
];

for (const file of files) {
  const content = fs.readFileSync(path.join(dir, file), 'utf-8');
  // Match "export type TypeName" patterns
  const matches = [...content.matchAll(/export\s+type\s+(\w+)/g)];
  if (matches.length > 0) {
    const types = matches.map(m => m[1]).join(', ');
    const mod = file.replace('.ts', '');
    lines.push(`export type { ${types} } from './${mod}';`);
  }
}

lines.push('');
fs.writeFileSync(path.join(dir, 'index.ts'), lines.join('\n'), 'utf-8');
console.log(`Generated index.ts with ${files.length} modules.`);
