const fs = require('fs');
let c = fs.readFileSync('src/lexer.rs', 'utf8');

c = c.replace(/        let text = code_lines\.join\("\\n"\);\r?\n        let text = if text\.is_empty\(\) \{ text \} else \{ text \+ "\\n" \};\r?\n        Some\(Token::CodeBlock \{\r?\n            lang: None,\r?\n            text,\r?\n        \}\)\r?\n    \}\r?\n\r?\n    fn parse_blockquote\(\&mut self\) -> Option<Token> \{\r?\n        let line = \&self\.lines\[self\.line_idx\];\r?\n                return None;\r?\n            \}\r?\n        \}\r?\n/g, '');

fs.writeFileSync('src/lexer.rs', c);
