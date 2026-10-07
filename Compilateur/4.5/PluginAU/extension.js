const vscode = require('vscode');

const KEYWORDS_CONTROL = new Set([
  'fn', 'main', 'if', 'while', 'loop', 'for', 'break', 'let', 'reg',
  'in', 'out', 'true', 'false', 'cutall', 'set', 'setmode'
]);
const OPCODES = new Set([
  'ADD', 'SUB', 'OR', 'AND', 'XOR', 'AND1B', 'FLAG',
  'LATCH', 'RESET', 'JMP', 'NOP', 'STOP'
]);
const OPERATORS = new Set([
  '==', '!=', '<=', '>=', '<', '>', '+', '-', '*', '/', '%',
  '&', '|', '^', '&&', '||', '!'
]);

let diagnosticCollection;

function activate(context) {
  diagnosticCollection = vscode.languages.createDiagnosticCollection('msl5');
  context.subscriptions.push(diagnosticCollection);

  context.subscriptions.push(
    vscode.workspace.onDidChangeTextDocument(event => {
      if (event.document.languageId === 'msl5') {
        analyzeMSL5(event.document);
      }
    })
  );

  context.subscriptions.push(
    vscode.workspace.onDidOpenTextDocument(document => {
      if (document.languageId === 'msl5') {
        analyzeMSL5(document);
      }
    })
  );

  context.subscriptions.push(
    vscode.languages.registerHoverProvider('msl5', {
      provideHover(document, position, token) {
        const range = document.getWordRangeAtPosition(position);
        if (!range) return null;
        const word = document.getText(range);
        const documentation = getOpcodeDocumentation(word);
        if (documentation) {
          return new vscode.Hover(new vscode.MarkdownString(documentation));
        }
        return null;
      }
    })
  );

  context.subscriptions.push(
    vscode.languages.registerCompletionItemProvider('msl5', {
      provideCompletionItems(document, position, token, context) {
        return getCompletionItems();
      }
    }, '')
  );

  vscode.workspace.textDocuments.forEach(doc => {
    if (doc.languageId === 'msl5') {
      analyzeMSL5(doc);
    }
  });
}

function getOpcodeDocumentation(word) {
  const docs = {
    'ADD': '**ADD** - Additionne deux valeurs\n\n```\nADD d1, d2\n```\nÉquivalent: `let x = a + b`',
    'SUB': '**SUB** - Soustrait deux valeurs\n\n```\nSUB d1, d2\n```\nÉquivalent: `let x = a - b`',
    'OR': '**OR** - OU logique bit à bit\n\n```\nOR d1, d2\n```\nÉquivalent: `let x = a | b`',
    'AND': '**AND** - ET logique bit à bit\n\n```\nAND d1, d2\n```\nÉquivalent: `let x = a & b`',
    'XOR': '**XOR** - OU exclusif bit à bit\n\n```\nXOR d1, d2\n```\nÉquivalent: `let x = a ^ b`',
    'AND1B': '**AND1B** - ET logique sur 1 bit\n\n```\nAND1B d1, d2\n```\nUtilisé pour les comparaisons `==`',
    'FLAG': '**FLAG** - Flag de comparaison (égalité)\n\n```\nFLAG\n```\nUtilisé après AND1B dans un `if`/`while`',
    'LATCH': '**LATCH** - Verrouille le registre\n\n```\nLATCH\n```\nToute instruction bloquée jusqu\'au `RESET`',
    'RESET': '**RESET** - Déverrouille le registre\n\n```\nRESET\n```',
    'JMP': '**JMP** - Saut à une adresse\n\n```\nJMP adresse\n```',
    'NOP': '**NOP** - Pas d\'opération (délai 1 cycle)\n\n```\nNOP\n```',
    'STOP': '**STOP** - Arrête le programme\n\n```\nSTOP\n```',
    'BREAK': '**BREAK** - Sort de la boucle courante\n\n```\nBREAK\n```\nUtilisable dans `while`, `loop`, `for`',
    'fn': '**fn** - Déclare une fonction\n\n```\nfn main() {\n  ...\n}\n```',
    'main': '**main** - Fonction principale (point d\'entrée)',
    'if': '**if** - Condition\n\n```\nif x == y { ... }\nif x > y { ... }\nif x < y { ... }\nif In(1) == true { ... }\n```',
    'while': '**while** - Boucle conditionnelle\n\n```\nwhile x == y { ... }\nwhile x > y { ... }\nwhile 1 == 1 { ... }\n```',
    'loop': '**loop** - Boucle répétée N fois\n\n```\nloop 10 { ... }\nloop { ... }  // infini\n```',
    'for': '**for** - Boucle FOR\n\n```\nfor x in 0..10 { ... }\nfor 5 { ... }\n```',
    'let': '**let** - Déclaration de variable\n\n```\nlet x = 42\nlet x = a + b\nlet x = a - b\nlet x = a & b\nlet x = a | b\nlet x = a ^ b\n```',
    'reg': '**reg** - Lecture de registre\n\n```\nreg read x\n```',
    'In': '**In(pin)** - Lecture d\'une pin GPIO\n\n```\nIn(1)  // lire pin 1\n```\nPins: 1, 2, 3, 4',
    'Out': '**Out(pin)** - Écriture sur une pin GPIO\n\n```\nOut(1)              // set (impulsion)\nOut(1).Set(0)       // set niveau bas\nOut(1).Set(1)       // set niveau haut\nOut(1).Set(1).SetMode(2)  // mode impulsion\nOut(0).CutAll()     // coupe toutes les sorties\n```'
  };
  return docs[word.toUpperCase()] || docs[word];
}

function getCompletionItems() {
  const items = [];

  const controlStructures = [
    {
      label: 'fn main()',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'fn main() {\n\t$0\n}',
      documentation: 'Structure principale du programme'
    },
    {
      label: 'while ==',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'while $1 == $2 {\n\t$0\n}',
      documentation: 'Boucle conditionnelle (égalité)'
    },
    {
      label: 'while >',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'while $1 > $2 {\n\t$0\n}',
      documentation: 'Boucle conditionnelle (supérieur)'
    },
    {
      label: 'while <',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'while $1 < $2 {\n\t$0\n}',
      documentation: 'Boucle conditionnelle (inférieur)'
    },
    {
      label: 'if ==',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'if $1 == $2 {\n\t$0\n}',
      documentation: 'Condition égalité'
    },
    {
      label: 'if >',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'if $1 > $2 {\n\t$0\n}',
      documentation: 'Condition supérieur'
    },
    {
      label: 'if <',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'if $1 < $2 {\n\t$0\n}',
      documentation: 'Condition inférieur'
    },
    {
      label: 'LOOP',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'loop $1 {\n\t$0\n}',
      documentation: 'Boucle répétée N fois'
    },
    {
      label: 'FOR range',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'for $1 in $2..$3 {\n\t$0\n}',
      documentation: 'Boucle FOR avec plage'
    },
    {
      label: 'if In()',
      kind: vscode.CompletionItemKind.Snippet,
      insertText: 'if In($1) == $2 {\n\t$0\n}',
      documentation: 'Condition sur pin GPIO'
    }
  ];

  const opcodeSnippets = [
    { label: 'ADD', insertText: 'ADD $1, $2', documentation: 'Addition' },
    { label: 'SUB', insertText: 'SUB $1, $2', documentation: 'Soustraction' },
    { label: 'OR', insertText: 'OR $1, $2', documentation: 'OU logique' },
    { label: 'AND', insertText: 'AND $1, $2', documentation: 'ET logique' },
    { label: 'XOR', insertText: 'XOR $1, $2', documentation: 'OU exclusif' },
    { label: 'AND1B', insertText: 'AND1B $1, $2', documentation: 'ET sur 1 bit' },
    { label: 'FLAG', insertText: 'FLAG', documentation: 'Définir flag' },
    { label: 'LATCH', insertText: 'LATCH', documentation: 'Verrouiller registre' },
    { label: 'RESET', insertText: 'RESET', documentation: 'Déverrouiller registre' },
    { label: 'JMP', insertText: 'JMP $1', documentation: 'Saut' },
    { label: 'NOP', insertText: 'NOP', documentation: 'Pas d\'opération' },
    { label: 'STOP', insertText: 'STOP', documentation: 'Arrêter le programme' },
    { label: 'BREAK', insertText: 'BREAK', documentation: 'Sortir de boucle' }
  ];

  const variableGpioSnippets = [
    { label: 'LET', insertText: 'let ${1:x} = $0', documentation: 'Déclaration de variable' },
    { label: 'LET expr', insertText: 'let ${1:x} = $2 ${3|+,-,|,&,^|} $4', documentation: 'Variable avec opération' },
    { label: 'REG READ', insertText: 'reg read $0', documentation: 'Lecture de registre' },
    { label: 'In()', insertText: 'In($0)', documentation: 'Lire pin GPIO' },
    { label: 'Out().Set().SetMode()', insertText: 'Out(${1:pin}).Set(${2:0}).SetMode(${3:2})', documentation: 'Sortie GPIO complète' },
    { label: 'Out().Set()', insertText: 'Out(${1:pin}).Set(${2:0})', documentation: 'Sortie GPIO avec Set' },
    { label: 'Out()', insertText: 'Out(${1:pin})', documentation: 'Sortie GPIO (impulsion)' },
    { label: 'Out(0).CutAll()', insertText: 'Out(0).CutAll()', documentation: 'Couper toutes les sorties' }
  ];

  controlStructures.forEach(item => {
    const completion = new vscode.CompletionItem(item.label, item.kind);
    completion.insertText = new vscode.SnippetString(item.insertText);
    completion.documentation = item.documentation;
    completion.sortText = '1_' + item.label;
    items.push(completion);
  });

  opcodeSnippets.forEach(item => {
    const completion = new vscode.CompletionItem(item.label, vscode.CompletionItemKind.Function);
    completion.insertText = new vscode.SnippetString(item.insertText);
    completion.documentation = item.documentation;
    completion.sortText = '2_' + item.label;
    items.push(completion);
  });

  variableGpioSnippets.forEach(item => {
    const completion = new vscode.CompletionItem(item.label, vscode.CompletionItemKind.Snippet);
    completion.insertText = new vscode.SnippetString(item.insertText);
    completion.documentation = item.documentation;
    completion.sortText = '3_' + item.label;
    items.push(completion);
  });

  return items;
}

function analyzeMSL5(document) {
  const diagnostics = [];
  const text = document.getText();
  const lines = text.split('\n');
  let braceStack = [];
  let inWhile = [];
  let inLatch = false;
  let latchLine = -1;
  let braceLineMap = [];

  for (let lineNum = 0; lineNum < lines.length; lineNum++) {
    const line = lines[lineNum];
    const trimmed = line.trim();

    if (trimmed.startsWith('//') || trimmed.startsWith('#')) {
      continue;
    }

    for (let i = 0; i < line.length; i++) {
      if (line[i] === '{') {
        braceStack.push({ line: lineNum, char: i });
        const upper = trimmed.toUpperCase();
        if (upper.startsWith('WHILE') || upper.startsWith('LOOP') || upper.startsWith('FOR')) {
          inWhile.push(lineNum);
        }
      } else if (line[i] === '}') {
        if (braceStack.length > 0) {
          braceStack.pop();
        }
        if (inWhile.length > 0) {
          inWhile.pop();
        }
        if (inLatch && braceStack.length === 0) {
          inLatch = false;
        }
      }
    }

    const upperTrimmed = trimmed.toUpperCase();

    if (upperTrimmed.includes('BREAK') && inWhile.length === 0) {
      const breakPos = line.toUpperCase().indexOf('BREAK');
      const actualPos = line.indexOf(trimmed.substring(0, trimmed.length));
      diagnostics.push(
        new vscode.Diagnostic(
          new vscode.Range(lineNum, breakPos, lineNum, breakPos + 5),
          'BREAK interdit en dehors d\'une boucle (while/loop/for)',
          vscode.DiagnosticSeverity.Error
        )
      );
    }

    if (upperTrimmed === 'LATCH' || upperTrimmed.startsWith('LATCH')) {
      inLatch = true;
      latchLine = lineNum;
    }

    if (upperTrimmed === 'RESET' || upperTrimmed.startsWith('RESET')) {
      inLatch = false;
    }

    if (inLatch && !upperTrimmed.includes('RESET') && trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')) {
      const tokens = trimmed.split(/\s+/);
      const firstToken = tokens[0].toUpperCase();

      if (OPCODES.has(firstToken) && firstToken !== 'NOP') {
        diagnostics.push(
          new vscode.Diagnostic(
            new vscode.Range(lineNum, 0, lineNum, line.length),
            `Registre verrouillé (LATCH ligne ${latchLine + 1}) - Opération bloquée jusqu\'au RESET`,
            vscode.DiagnosticSeverity.Warning
          )
        );
      }
    }

    const tokens = trimmed.split(/[\s,]+/).filter(t => t);
    for (let i = 0; i < tokens.length; i++) {
      const token = tokens[i].toUpperCase();
      const col = line.indexOf(tokens[i]);

      if (['ADD', 'SUB', 'OR', 'AND', 'XOR', 'AND1B'].includes(token)) {
        if (i + 2 >= tokens.length) {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, col + tokens[i].length),
              `${tokens[i]} nécessite 2 arguments (d1, d2)`,
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (token === 'FLAG' || token === 'JMP') {
        if (i + 1 >= tokens.length) {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, col + tokens[i].length),
              `${tokens[i]} nécessite 1 argument`,
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (token === 'LET') {
        if (tokens.length < 4 || tokens[2].toUpperCase() !== '=') {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, line.length),
              'Syntaxe: let x = valeur  ou  let x = a + b',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (token === 'REG') {
        if (i + 1 >= tokens.length || tokens[i + 1].toUpperCase() !== 'READ') {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, col + tokens[i].length),
              'Syntaxe: reg read variable',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (token === 'LOOP') {
        if (tokens.length > 2) {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, line.length),
              'Syntaxe: loop [nombre] { ... }',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (token === 'FOR') {
        const hasIn = tokens.some(t => t.toUpperCase() === 'IN');
        if (tokens.length < 2) {
          diagnostics.push(
            new vscode.Diagnostic(
              new vscode.Range(lineNum, col, lineNum, line.length),
              'Syntaxe: for var in debut..fin { ... }  ou  for nombre { ... }',
              vscode.DiagnosticSeverity.Error
            )
          );
        }
      }

      if (!KEYWORDS_CONTROL.has(token) && !OPCODES.has(token) &&
          !OPERATORS.has(token) && !/^\d+$/.test(token) &&
          token !== '{' && token !== '}' && token !== '(' && token !== ')' &&
          token !== '==' && token !== '!=' && token !== '..' &&
          !token.startsWith('.') && token !== '=') {

        if (/^(0x[0-9a-fA-F]+|[0-9]+)$/.test(token)) {
          continue;
        }

        if (![',', ';', '{', '}', '(', ')', '==', '!=', '<', '>', '<=', '>=', '..'].includes(token)) {
          if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(token)) {
            // identificateur valide
          }
        }
      }
    }
  }

  if (braceStack.length > 0) {
    braceStack.forEach(brace => {
      diagnostics.push(
        new vscode.Diagnostic(
          new vscode.Range(brace.line, brace.char, brace.line, brace.char + 1),
          'Accolade { non fermée',
          vscode.DiagnosticSeverity.Error
        )
      );
    });
  }

  diagnosticCollection.set(document.uri, diagnostics);
}

function deactivate() {
  if (diagnosticCollection) {
    diagnosticCollection.dispose();
  }
}

module.exports = {
  activate,
  deactivate
};
