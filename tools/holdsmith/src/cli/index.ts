#!/usr/bin/env node
import { Command } from 'commander';
import { readFileSync, writeFileSync, readdirSync, statSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname, relative } from 'node:path';
import { parse } from '../parser/parser.js';
import { compile, emitTypeScript } from '../compiler/compiler.js';
import { ParseError } from '../parser/errors.js';

const program = new Command();

program
  .name('holdsmith')
  .description('Scene DSL toolkit for narrative games')
  .version('0.1.0');

program
  .command('compile')
  .description('Compile .scene files to TypeScript')
  .argument('<input>', 'Input directory containing .scene files')
  .option('-o, --output <dir>', 'Output directory', './compiled')
  .option('--json', 'Output JSON instead of TypeScript')
  .action((input: string, options: { output: string; json?: boolean }) => {
    const files = findSceneFiles(input);
    
    if (files.length === 0) {
      console.error(`No .scene files found in ${input}`);
      process.exit(1);
    }
    
    console.log(`Found ${files.length} scene file(s)`);
    
    let hasErrors = false;
    const compiled: Array<{ file: string; id: string }> = [];
    
    for (const file of files) {
      try {
        const source = readFileSync(file, 'utf-8');
        const ast = parse(source, file);
        const result = compile(ast);
        
        for (const warning of result.warnings) {
          console.warn(`  Warning in ${file}: ${warning.message}`);
        }
        
        const relPath = relative(input, file);
        const outPath = join(
          options.output, 
          relPath.replace(/\.scene$/, options.json ? '.json' : '.ts')
        );
        
        const outDir = dirname(outPath);
        if (!existsSync(outDir)) {
          mkdirSync(outDir, { recursive: true });
        }
        
        const relDir = dirname(relPath);
        const subdirCount = relDir === '.' ? 0 : relDir.split('/').filter(p => p).length;
        const depth = 4 + subdirCount;
        const importPath = '../'.repeat(depth) + 'core/types.js';
        
        const output = options.json 
          ? JSON.stringify(result.scenelet, null, 2)
          : emitTypeScript(result.scenelet, importPath);
        
        writeFileSync(outPath, output);
        console.log(`  Compiled: ${relPath} -> ${relative('.', outPath)}`);
        
        compiled.push({ file: relPath, id: result.scenelet.id });
      } catch (err) {
        hasErrors = true;
        if (err instanceof ParseError) {
          console.error(`  Error in ${file}:`);
          console.error(`    ${err.message}`);
        } else {
          console.error(`  Error in ${file}: ${err}`);
        }
      }
    }
    
    if (!options.json && compiled.length > 0) {
      const indexPath = join(options.output, 'index.ts');
      const indexContent = generateIndex(compiled);
      writeFileSync(indexPath, indexContent);
      console.log(`  Generated: ${relative('.', indexPath)}`);
    }
    
    if (hasErrors) {
      console.error('\nCompilation completed with errors');
      process.exit(1);
    } else {
      console.log(`\nSuccessfully compiled ${compiled.length} scene(s)`);
    }
  });

program
  .command('validate')
  .description('Validate .scene files without compiling')
  .argument('<input>', 'Input directory containing .scene files')
  .action((input: string) => {
    const files = findSceneFiles(input);
    
    if (files.length === 0) {
      console.error(`No .scene files found in ${input}`);
      process.exit(1);
    }
    
    let hasErrors = false;
    let errorCount = 0;
    let warningCount = 0;
    
    for (const file of files) {
      try {
        const source = readFileSync(file, 'utf-8');
        const ast = parse(source, file);
        const result = compile(ast);
        
        for (const warning of result.warnings) {
          warningCount++;
          console.warn(`Warning: ${file}:${warning.line ?? 0}: ${warning.message}`);
        }
      } catch (err) {
        hasErrors = true;
        errorCount++;
        if (err instanceof ParseError) {
          console.error(`Error: ${file}:${err.span.start.line}: ${err.message}`);
        } else {
          console.error(`Error: ${file}: ${err}`);
        }
      }
    }
    
    console.log(`\nValidated ${files.length} file(s): ${errorCount} error(s), ${warningCount} warning(s)`);
    
    if (hasErrors) {
      process.exit(1);
    }
  });

program
  .command('parse')
  .description('Parse a single .scene file and output AST')
  .argument('<file>', 'Scene file to parse')
  .action((file: string) => {
    try {
      const source = readFileSync(file, 'utf-8');
      const ast = parse(source, file);
      console.log(JSON.stringify(ast, null, 2));
    } catch (err) {
      if (err instanceof ParseError) {
        console.error(`Parse error: ${err.message}`);
      } else {
        console.error(`Error: ${err}`);
      }
      process.exit(1);
    }
  });

function findSceneFiles(dir: string): string[] {
  const files: string[] = [];
  
  function walk(currentDir: string) {
    const entries = readdirSync(currentDir);
    for (const entry of entries) {
      const fullPath = join(currentDir, entry);
      const stat = statSync(fullPath);
      
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (entry.endsWith('.scene')) {
        files.push(fullPath);
      }
    }
  }
  
  walk(dir);
  return files;
}

function generateIndex(compiled: Array<{ file: string; id: string }>): string {
  const lines: string[] = [];
  
  lines.push('import type { Scenelet } from \'../../../../core/types.js\';');
  
  for (const { file, id } of compiled) {
    const importPath = './' + file.replace(/\.scene$/, '.js').replace(/\\/g, '/');
    const varName = sanitizeIdentifier(id);
    lines.push(`import { ${varName} } from '${importPath}';`);
  }
  
  lines.push('');
  
  for (const { id } of compiled) {
    const varName = sanitizeIdentifier(id);
    lines.push(`export { ${varName} };`);
  }
  
  lines.push('');
  
  const imports = compiled.map(c => sanitizeIdentifier(c.id));
  lines.push(`export const ALL_COMPILED_SCENELETS: Scenelet[] = [${imports.join(', ')}];`);
  
  return lines.join('\n');
}

function sanitizeIdentifier(id: string): string {
  return id.replace(/[^a-zA-Z0-9_]/g, '_');
}

program.parse(process.argv);
