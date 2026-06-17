import sys

with open('submission_bundle_v3.txt', 'r', encoding='utf-8') as f:
    lines = f.readlines()

in_lexer = False
lexer_lines = []

for line in lines:
    if line.strip() == "FILE: D:/post_mortem/src/lexer.rs":
        in_lexer = True
        continue
    if in_lexer and line.startswith("FILE: "):
        break
    if in_lexer:
        if not line.startswith("======="):
            lexer_lines.append(line)

with open('src/lexer.rs', 'w', encoding='utf-8') as f:
    f.write(''.join(lexer_lines).lstrip())
