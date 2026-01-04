import type { SourceLocation, SourceSpan } from './ast.js';

export type TokenType =
  | 'FRONTMATTER_DELIM'
  | 'PASSAGE_HEADER'
  | 'CHOICE_MARKER'
  | 'EFFECT_MARKER'
  | 'ARROW'
  | 'LBRACE'
  | 'RBRACE'
  | 'LBRACKET'
  | 'RBRACKET'
  | 'COMMA'
  | 'COLON'
  | 'EQUALS'
  | 'PLUS_EQUALS'
  | 'MINUS_EQUALS'
  | 'BANG'
  | 'DOT'
  | 'COMPARISON'
  | 'NUMBER'
  | 'STRING'
  | 'IDENTIFIER'
  | 'TEXT'
  | 'NEWLINE'
  | 'INDENT'
  | 'DEDENT'
  | 'EOF'
  | 'ERROR';

export interface Token {
  readonly type: TokenType;
  readonly value: string;
  readonly span: SourceSpan;
}

export interface LexerState {
  source: string;
  filename: string;
  pos: number;
  line: number;
  column: number;
  indentStack: number[];
  inFrontmatter: boolean;
  atLineStart: boolean;
  inProseContext: boolean; // After passage header, before choice/effect markers
}

const COMPARISON_OPS = ['>=', '<=', '==', '!=', '>', '<'];

export function createLexer(source: string, filename: string = '<input>'): LexerState {
  return {
    source,
    filename,
    pos: 0,
    line: 1,
    column: 1,
    indentStack: [0],
    inFrontmatter: false,
    atLineStart: true,
    inProseContext: false,
  };
}

function currentLocation(state: LexerState): SourceLocation {
  return { line: state.line, column: state.column, offset: state.pos };
}

function makeSpan(start: SourceLocation, end: SourceLocation, filename: string): SourceSpan {
  return { start, end, source: filename };
}

function peek(state: LexerState, offset: number = 0): string {
  return state.source[state.pos + offset] ?? '';
}

function peekString(state: LexerState, length: number): string {
  return state.source.slice(state.pos, state.pos + length);
}

function advance(state: LexerState, count: number = 1): string {
  const chars = state.source.slice(state.pos, state.pos + count);
  for (const ch of chars) {
    if (ch === '\n') {
      state.line++;
      state.column = 1;
    } else {
      state.column++;
    }
    state.pos++;
  }
  return chars;
}

function skipWhitespaceOnLine(state: LexerState): void {
  while (peek(state) === ' ' || peek(state) === '\t') {
    advance(state);
  }
}

function isAtEnd(state: LexerState): boolean {
  return state.pos >= state.source.length;
}

function isAlpha(ch: string): boolean {
  return /[a-zA-Z_]/.test(ch);
}

function isAlphaNumeric(ch: string): boolean {
  return /[a-zA-Z0-9_]/.test(ch);
}

function isDigit(ch: string): boolean {
  return /[0-9]/.test(ch);
}

function makeToken(state: LexerState, type: TokenType, value: string, start: SourceLocation): Token {
  return {
    type,
    value,
    span: makeSpan(start, currentLocation(state), state.filename),
  };
}

function scanNumber(state: LexerState): Token {
  const start = currentLocation(state);
  let value = '';
  
  if (peek(state) === '-') {
    value += advance(state);
  }
  
  while (isDigit(peek(state))) {
    value += advance(state);
  }
  
  if (peek(state) === '.' && isDigit(peek(state, 1))) {
    value += advance(state);
    while (isDigit(peek(state))) {
      value += advance(state);
    }
  }
  
  return makeToken(state, 'NUMBER', value, start);
}

function scanIdentifier(state: LexerState): Token {
  const start = currentLocation(state);
  let value = '';
  
  while (isAlphaNumeric(peek(state))) {
    value += advance(state);
  }
  
  return makeToken(state, 'IDENTIFIER', value, start);
}

function scanString(state: LexerState): Token {
  const start = currentLocation(state);
  const quote = advance(state);
  let value = '';
  
  while (!isAtEnd(state) && peek(state) !== quote && peek(state) !== '\n') {
    if (peek(state) === '\\' && peek(state, 1) === quote) {
      advance(state);
      value += advance(state);
    } else {
      value += advance(state);
    }
  }
  
  if (peek(state) === quote) {
    advance(state);
  }
  
  return makeToken(state, 'STRING', value, start);
}

function scanTextUntilSpecial(state: LexerState): Token {
  const start = currentLocation(state);
  let value = '';
  
  while (!isAtEnd(state)) {
    const ch = peek(state);
    
    if (ch === '\n') break;
    if (ch === '*' && state.atLineStart) break;
    if (ch === '~' && state.atLineStart) break;
    if (ch === '-' && peek(state, 1) === '>') break;
    if (ch === '{') break;
    if (ch === '[') break;
    if (ch === ']') break;
    
    value += advance(state);
    state.atLineStart = false;
  }
  
  return makeToken(state, 'TEXT', value.trim(), start);
}

function scanFrontmatterLine(state: LexerState): Token {
  const start = currentLocation(state);
  let value = '';
  
  while (!isAtEnd(state) && peek(state) !== '\n') {
    value += advance(state);
  }
  
  return makeToken(state, 'TEXT', value, start);
}

function scanProseLine(state: LexerState): Token {
  const start = currentLocation(state);
  let value = '';
  
  while (!isAtEnd(state) && peek(state) !== '\n') {
    value += advance(state);
  }
  
  return makeToken(state, 'TEXT', value.trim(), start);
}

function isProseLineStart(state: LexerState): boolean {
  const ch = peek(state);
  const nextCh = peek(state, 1);
  const next2Ch = peek(state, 2);
  
  if (ch === '*') return false;
  if (ch === '~') return false;
  if (ch === '-' && nextCh === '>') return false;
  if (ch === '-' && nextCh === '-' && next2Ch === '-') return false;
  if (ch === '=' && nextCh === '=' && next2Ch === '=') return false;
  if (ch === '/' && nextCh === '/') return false;
  if (ch === '\n') return false;
  if (ch === '') return false;
  
  return true;
}

function measureIndent(state: LexerState): number {
  let indent = 0;
  let tempPos = state.pos;
  
  while (tempPos < state.source.length) {
    const ch = state.source[tempPos];
    if (ch === ' ') {
      indent++;
      tempPos++;
    } else if (ch === '\t') {
      indent += 2;
      tempPos++;
    } else {
      break;
    }
  }
  
  return indent;
}

export function* tokenize(state: LexerState): Generator<Token, void, undefined> {
  while (!isAtEnd(state)) {
    const start = currentLocation(state);
    
    if (peekString(state, 3) === '---') {
      advance(state, 3);
      state.inFrontmatter = !state.inFrontmatter;
      state.inProseContext = false;
      yield makeToken(state, 'FRONTMATTER_DELIM', '---', start);
      
      if (peek(state) === '\n') {
        advance(state);
        state.atLineStart = true;
      }
      continue;
    }
    
    if (state.inFrontmatter) {
      if (peek(state) === '\n') {
        advance(state);
        state.atLineStart = true;
        yield makeToken(state, 'NEWLINE', '\n', start);
        continue;
      }
      
      const line = scanFrontmatterLine(state);
      if (line.value.length > 0) {
        yield line;
      }
      continue;
    }
    
    if (peek(state) === '\n') {
      advance(state);
      state.atLineStart = true;
      yield makeToken(state, 'NEWLINE', '\n', start);
      continue;
    }
    
    if (state.atLineStart) {
      const indent = measureIndent(state);
      const currentIndent = state.indentStack[state.indentStack.length - 1] ?? 0;
      
      if (indent > currentIndent) {
        state.indentStack.push(indent);
        skipWhitespaceOnLine(state);
        state.inProseContext = false;
        yield makeToken(state, 'INDENT', '', start);
        state.atLineStart = false;
        continue;
      } else if (indent < currentIndent) {
        while (state.indentStack.length > 1 && 
               (state.indentStack[state.indentStack.length - 1] ?? 0) > indent) {
          state.indentStack.pop();
          yield makeToken(state, 'DEDENT', '', start);
        }
        skipWhitespaceOnLine(state);
        state.atLineStart = false;
        continue;
      } else {
        skipWhitespaceOnLine(state);
        state.atLineStart = false;
      }
    }
    
    if (peek(state) === ' ' || peek(state) === '\t') {
      skipWhitespaceOnLine(state);
      continue;
    }
    
    if (peekString(state, 2) === '//') {
      while (!isAtEnd(state) && peek(state) !== '\n') {
        advance(state);
      }
      continue;
    }
    
    if (peekString(state, 3) === '===') {
      advance(state, 3);
      skipWhitespaceOnLine(state);
      
      let name = '';
      while (isAlphaNumeric(peek(state))) {
        name += advance(state);
      }
      
      state.inProseContext = true;
      yield makeToken(state, 'PASSAGE_HEADER', name, start);
      continue;
    }
    
    if (state.inProseContext && isProseLineStart(state)) {
      const proseLine = scanProseLine(state);
      if (proseLine.value.length > 0) {
        yield proseLine;
      }
      continue;
    }
    
    if (peek(state) === '*') {
      state.inProseContext = false;
      advance(state);
      yield makeToken(state, 'CHOICE_MARKER', '*', start);
      continue;
    }
    
    if (peek(state) === '~') {
      state.inProseContext = false;
      advance(state);
      yield makeToken(state, 'EFFECT_MARKER', '~', start);
      continue;
    }
    
    if (peekString(state, 2) === '->') {
      state.inProseContext = false;
      advance(state, 2);
      yield makeToken(state, 'ARROW', '->', start);
      continue;
    }
    
    let foundComparison = false;
    for (const op of COMPARISON_OPS) {
      if (peekString(state, op.length) === op) {
        advance(state, op.length);
        yield makeToken(state, 'COMPARISON', op, start);
        foundComparison = true;
        break;
      }
    }
    if (foundComparison) continue;
    
    if (peekString(state, 2) === '+=') {
      advance(state, 2);
      yield makeToken(state, 'PLUS_EQUALS', '+=', start);
      continue;
    }
    
    if (peekString(state, 2) === '-=') {
      advance(state, 2);
      yield makeToken(state, 'MINUS_EQUALS', '-=', start);
      continue;
    }
    
    const singleChars: Record<string, TokenType> = {
      '[': 'LBRACKET',
      ']': 'RBRACKET',
      '{': 'LBRACE',
      '}': 'RBRACE',
      ',': 'COMMA',
      ':': 'COLON',
      '=': 'EQUALS',
      '!': 'BANG',
      '.': 'DOT',
    };
    
    if (peek(state) in singleChars) {
      const ch = advance(state);
      yield makeToken(state, singleChars[ch]!, ch, start);
      continue;
    }
    
    if (isDigit(peek(state)) || (peek(state) === '-' && isDigit(peek(state, 1)))) {
      yield scanNumber(state);
      continue;
    }
    
    if (peek(state) === '"') {
      yield scanString(state);
      continue;
    }
    
    if (isAlpha(peek(state))) {
      yield scanIdentifier(state);
      continue;
    }
    
    const text = scanTextUntilSpecial(state);
    if (text.value.length > 0) {
      yield text;
    } else {
      advance(state);
    }
  }
  
  while (state.indentStack.length > 1) {
    state.indentStack.pop();
    yield makeToken(state, 'DEDENT', '', currentLocation(state));
  }
  
  yield makeToken(state, 'EOF', '', currentLocation(state));
}

export function lex(source: string, filename?: string): Token[] {
  const state = createLexer(source, filename);
  return [...tokenize(state)];
}
