/**
 * CodeMirror 6 setup for .scene files.
 *
 * This file initializes CodeMirror with syntax highlighting for the
 * Holdsmith scene format.
 */

import { EditorView, basicSetup } from 'codemirror';
import { EditorState } from '@codemirror/state';
import { StreamLanguage } from '@codemirror/language';
import { oneDark } from '@codemirror/theme-one-dark';

// Custom language mode for .scene files
const sceneLanguage = StreamLanguage.define({
    name: 'scene',

    startState() {
        return {
            inFrontmatter: false,
            frontmatterDone: false,
        };
    },

    token(stream, state) {
        // Handle frontmatter (YAML between --- markers)
        if (stream.sol() && stream.match(/^---\s*$/)) {
            if (!state.frontmatterDone) {
                state.inFrontmatter = !state.inFrontmatter;
                if (!state.inFrontmatter) {
                    state.frontmatterDone = true;
                }
            }
            return 'meta';
        }

        if (state.inFrontmatter) {
            // YAML key
            if (stream.match(/^[a-zA-Z_][a-zA-Z0-9_]*(?=\s*:)/)) {
                return 'propertyName';
            }
            // YAML value
            if (stream.match(/^:\s*/)) {
                return 'punctuation';
            }
            // String values
            if (stream.match(/"[^"]*"|'[^']*'/)) {
                return 'string';
            }
            // Numbers
            if (stream.match(/\b\d+\b/)) {
                return 'number';
            }
            stream.next();
            return null;
        }

        // Passage header: === passage_name ===
        if (stream.sol() && stream.match(/^===\s*/)) {
            stream.match(/[^\s=]+/);
            stream.match(/\s*===\s*$/);
            return 'heading';
        }

        // Choice: * [choice text]
        if (stream.sol() && stream.match(/^\*\s*/)) {
            return 'keyword';
        }

        // Choice text in brackets
        if (stream.match(/\[[^\]]*\]/)) {
            return 'string';
        }

        // Condition: if(...) or unless(...)
        if (stream.match(/\b(if|unless)\s*\(/)) {
            // Read until matching paren
            let depth = 1;
            while (depth > 0 && !stream.eol()) {
                const ch = stream.next();
                if (ch === '(') depth++;
                if (ch === ')') depth--;
            }
            return 'keyword';
        }

        // Effect: ~ effect
        if (stream.sol() && stream.match(/^~\s*/)) {
            return 'operator';
        }

        // Effect content
        if (stream.match(/\b(set|add|sub|give|take|require)\b/)) {
            return 'keyword';
        }

        // Target: -> passage_name or -> END
        if (stream.match(/->\s*/)) {
            stream.match(/[A-Z_][A-Za-z0-9_]*/);
            return 'link';
        }

        // Resource/flag references: $resource or @flag
        if (stream.match(/\$[a-zA-Z_][a-zA-Z0-9_]*/)) {
            return 'variableName';
        }
        if (stream.match(/@[a-zA-Z_][a-zA-Z0-9_]*/)) {
            return 'variableName special';
        }

        // Comments
        if (stream.match(/\/\/.*/)) {
            return 'comment';
        }

        // Skip other characters
        stream.next();
        return null;
    },
});

// Store editor instances by element
const editors = new WeakMap();

// Track if we're updating from Rust (to avoid feedback loops)
let updatingFromRust = false;

/**
 * Initialize CodeMirror on an element.
 * @param {HTMLElement} element - The container element
 * @param {string} content - Initial content
 * @param {function} callback - Called when content changes
 */
window.initCodeMirror = function(element, content, callback) {
    // Check if already initialized
    if (editors.has(element)) {
        return;
    }

    const state = EditorState.create({
        doc: content,
        extensions: [
            basicSetup,
            oneDark,
            sceneLanguage,
            EditorView.updateListener.of((update) => {
                if (update.docChanged && !updatingFromRust) {
                    callback(update.state.doc.toString());
                }
            }),
        ],
    });

    const view = new EditorView({
        state,
        parent: element,
    });

    editors.set(element, view);
};

/**
 * Update the content of a CodeMirror instance.
 * @param {HTMLElement} element - The container element
 * @param {string} content - New content
 */
window.updateCodeMirrorContent = function(element, content) {
    const view = editors.get(element);
    if (!view) return;

    // Check if content actually changed
    if (view.state.doc.toString() === content) {
        return;
    }

    updatingFromRust = true;
    view.dispatch({
        changes: {
            from: 0,
            to: view.state.doc.length,
            insert: content,
        },
    });
    updatingFromRust = false;
};

/**
 * Set diagnostics in the editor.
 * @param {HTMLElement} element - The container element
 * @param {string} diagnosticsJson - JSON array of diagnostics
 */
window.setCodeMirrorDiagnostics = function(element, diagnosticsJson) {
    // TODO: Implement diagnostic markers (underlines, gutter icons)
    // This requires the @codemirror/lint extension
    console.log('Diagnostics:', diagnosticsJson);
};
