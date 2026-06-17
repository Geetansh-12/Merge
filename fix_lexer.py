import sys

with open('src/lexer.rs', 'r', encoding='utf-8') as f:
    c = f.read()

c = c.replace('\\n', '\n')
c = c.replace('\\r', '\r')
c = c.replace('\\t', '\t')
c = c.replace('\\"', '"')
c = c.replace('\\\\', '\\')

with open('src/lexer.rs', 'w', encoding='utf-8') as f:
    f.write(c)
