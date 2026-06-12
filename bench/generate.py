#!/usr/bin/env python3
"""Generate benchmark input files of specific sizes."""
import os
import random

def generate_markdown(target_bytes: int) -> str:
    sections = []
    current = 0
    level = 1

    while current < target_bytes:
        # Heading
        h = f"{'#' * level} Section {len(sections)+1}\n\n"
        sections.append(h)
        current += len(h)
        level = (level % 3) + 1

        # Paragraph
        words = ['The', 'quick', 'brown', 'fox', 'jumps', 'over',
                'the', 'lazy', 'dog', 'with', '**bold**', '*italic*',
                '`code`', 'text', 'and', 'more', 'content']
        para = ' '.join(random.choices(words, k=40)) + '\n\n'
        sections.append(para)
        current += len(para)

        # Code block
        code = f'```python\ndef func_{len(sections)}():\n    return {len(sections)}\n```\n\n'
        sections.append(code)
        current += len(code)

        # List
        items = [f'- Item {i}: {"word " * random.randint(3,8)}\n'
                for i in range(random.randint(3, 6))]
        lst = ''.join(items) + '\n'
        sections.append(lst)
        current += len(lst)

        # Table
        table = ('| Column A | Column B | Column C |\n'
                '|----------|----------|----------|\n'
                f'| data {len(sections)} | value | result |\n\n')
        sections.append(table)
        current += len(table)

        # Blockquote
        bq = f'> This is a blockquote with **emphasis** and [a link](https://example.com).\n\n'
        sections.append(bq)
        current += len(bq)

    return ''.join(sections)[:target_bytes]

os.makedirs('bench/input', exist_ok=True)

for name, size in [('small', 10_000), ('medium', 100_000), ('large', 1_000_000)]:
    content = generate_markdown(size)
    path = f'bench/input/{name}.md'
    with open(path, 'w') as f:
        f.write(content)
    print(f'Generated {path}: {len(content):,} bytes')
