/**
 * Monaco Editor Monarch Syntax Definition & Theme for LLVM IR
 */

export function registerLlvmLanguage(monaco) {
  monaco.languages.register({ id: 'llvm' });

  monaco.languages.setMonarchTokensProvider('llvm', {
    defaultToken: '',
    tokenPostfix: '.ll',

    keywords: [
      'define', 'declare', 'global', 'constant', 'internal', 'external', 'private',
      'linkonce', 'linkonce_odr', 'weak', 'weak_odr', 'appending', 'common', 'available_externally',
      'default', 'hidden', 'protected', 'unnamed_addr', 'local_unnamed_addr',
      'target', 'datalayout', 'triple', 'source_filename', 'module', 'asm',
      'sideeffect', 'alignstack', 'inteldialect', 'uselistorder', 'uselistorder_bb',
      'type', 'opaque', 'attributes', 'syncscope', 'personality'
    ],

    terminators: [
      'ret', 'br', 'switch', 'indirectbr', 'invoke', 'resume', 'unreachable', 'catchswitch', 'catchret', 'cleanupret'
    ],

    instructions: [
      'alloca', 'load', 'store', 'fence', 'cmpxchg', 'atomicrmw', 'getelementptr',
      'add', 'fadd', 'sub', 'fsub', 'mul', 'fmul', 'udiv', 'sdiv', 'fdiv', 'urem', 'srem', 'frem',
      'shl', 'lshr', 'ashr', 'and', 'or', 'xor',
      'extractelement', 'insertelement', 'shufflevector',
      'extractvalue', 'insertvalue',
      'trunc', 'zext', 'sext', 'fptrunc', 'fpext', 'fptoui', 'fptosi', 'uitofp', 'sitofp',
      'ptrtoint', 'inttoptr', 'bitcast', 'addrspacecast',
      'icmp', 'fcmp', 'phi', 'select', 'freeze', 'call', 'va_arg', 'landingpad', 'catchpad', 'cleanuppad'
    ],

    typeKeywords: [
      'void', 'half', 'bfloat', 'float', 'double', 'x86_fp80', 'fp128', 'ppc_fp128',
      'label', 'metadata', 'x86_mmx', 'x86_amx', 'token', 'ptr'
    ],

    predicates: [
      'eq', 'ne', 'ugt', 'uge', 'ult', 'ule', 'sgt', 'sge', 'slt', 'sle',
      'false', 'oeq', 'ogt', 'oge', 'olt', 'ole', 'one', 'ord', 'ueq', 'une', 'uno', 'true'
    ],

    modifiers: [
      'inbounds', 'exact', 'nuw', 'nsw', 'fast', 'nnan', 'ninf', 'nsz', 'arcp',
      'contract', 'afn', 'reassoc', 'volatile', 'atomic', 'unordered', 'monotonic',
      'acquire', 'release', 'acq_rel', 'seq_cst', 'align', 'tail', 'musttail', 'notail'
    ],

    tokenizer: {
      root: [
        // Comments
        [/;.*$/, 'comment'],

        // Strings
        [/"([^"\\]|\\.)*"/, 'string'],

        // Basic block labels (e.g. entry:, loop.body:)
        [/^[a-zA-Z$._0-9]+:\s*(;.*)?$/, 'type.identifier'],
        [/label\s+%[a-zA-Z$._0-9]+/, 'type.identifier'],

        // Global variables and function names (@main, @printf)
        [/@([a-zA-Z$._0-9]+|"[^"]+")/, 'variable.predefined'],

        // Local variables and SSA registers (%0, %res, %i.0)
        [/%([a-zA-Z$._0-9]+|"[^"]+")/, 'variable'],

        // Metadata (!dbg, !0, !"string")
        [/!([a-zA-Z$._0-9]+|"[^"]+"|(?=\d+)\d+)/, 'annotation'],

        // Integer types (i1, i8, i32, i64, i128)
        [/\bi\d+\b/, 'type'],

        // Built-in keywords
        [/[a-zA-Z_]\w*/, {
          cases: {
            '@keywords': 'keyword',
            '@terminators': 'keyword.flow',
            '@instructions': 'keyword.control',
            '@typeKeywords': 'type',
            '@predicates': 'operator',
            '@modifiers': 'attribute',
            '@default': 'identifier'
          }
        }],

        // Numbers (integers, hex, floats)
        [/-?0x[0-9a-fA-F]+/, 'number.hex'],
        [/-?\d+(\.\d+)?([eE][+-]?\d+)?/, 'number'],

        // Delimiters
        [/[{}()\[\]]/, '@brackets'],
        [/[<>]/, 'delimiter'],
        [/[,]/, 'delimiter'],
        [/[=]/, 'operator']
      ]
    }
  });

  // Language configuration (brackets, comments)
  monaco.languages.setLanguageConfiguration('llvm', {
    comments: {
      lineComment: ';'
    },
    brackets: [
      ['{', '}'],
      ['[', ']'],
      ['(', ')'],
      ['<', '>']
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '<', close: '>' },
      { open: '"', close: '"' }
    ]
  });

  // Define custom dark theme matching LLVM IR Studio palette
  monaco.editor.defineTheme('llvm-studio-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'comment', foreground: '64748b', fontStyle: 'italic' },
      { token: 'keyword', foreground: '38bdf8', fontStyle: 'bold' },
      { token: 'keyword.flow', foreground: 'f43f5e', fontStyle: 'bold' },
      { token: 'keyword.control', foreground: '00e5ff' },
      { token: 'type', foreground: 'a78bfa' },
      { token: 'type.identifier', foreground: 'facc15', fontStyle: 'bold' },
      { token: 'variable', foreground: 'e2e8f0' },
      { token: 'variable.predefined', foreground: 'fb923c', fontStyle: 'bold' },
      { token: 'annotation', foreground: '94a3b8' },
      { token: 'string', foreground: '4ade80' },
      { token: 'number', foreground: 'f472b6' },
      { token: 'operator', foreground: '38bdf8' },
      { token: 'attribute', foreground: '93c5fd' }
    ],
    colors: {
      'editor.background': '#0f172a',
      'editor.foreground': '#f8fafc',
      'editorLineNumber.foreground': '#475569',
      'editorLineNumber.activeForeground': '#38bdf8',
      'editor.lineHighlightBackground': '#1e293b55',
      'editor.selectionBackground': '#38bdf833',
      'editorCursor.foreground': '#00e5ff',
      'editorWhitespace.foreground': '#334155',
      'editorWidget.background': '#1e293b',
      'editorWidget.border': '#334155'
    }
  });
}
