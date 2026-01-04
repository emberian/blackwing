import type { SourceSpan } from './ast.js';

export enum ParseErrorCode {
  UNEXPECTED_TOKEN = 'UNEXPECTED_TOKEN',
  INVALID_FRONTMATTER = 'INVALID_FRONTMATTER',
  INVALID_CONDITION = 'INVALID_CONDITION',
  INVALID_EFFECT = 'INVALID_EFFECT',
  MISSING_PASSAGE = 'MISSING_PASSAGE',
  DUPLICATE_PASSAGE = 'DUPLICATE_PASSAGE',
  INVALID_NAVIGATION = 'INVALID_NAVIGATION',
}

export class ParseError extends Error {
  constructor(
    public readonly code: ParseErrorCode,
    message: string,
    public readonly span: SourceSpan
  ) {
    super(`${message} at ${span.source ?? '<input>'}:${span.start.line}:${span.start.column}`);
    this.name = 'ParseError';
  }
}

export function formatParseError(error: ParseError): string {
  return `[${error.code}] ${error.message}`;
}
