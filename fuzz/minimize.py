#!/usr/bin/env python3
import subprocess
import sys
import os

if len(sys.argv) < 2:
    print("Usage: minimize.py <input_file>")
    sys.exit(1)

with open(sys.argv[1], 'r', encoding='utf-8') as f:
    original_input = f.read()

# Make sure we have the built executable
if not os.path.exists("target/release/marked-rs.exe") and not os.path.exists("target/release/marked-rs"):
    subprocess.run(["cargo", "build", "--release"], check=True)

executable = "target/release/marked-rs.exe" if os.name == 'nt' else "target/release/marked-rs"

def get_outputs(test_input):
    try:
        rust_proc = subprocess.run([executable], input=test_input, text=True, capture_output=True, check=True)
        rust_out = rust_proc.stdout
    except subprocess.CalledProcessError:
        rust_out = "PANIC"
    
    node_script = """
import { marked } from 'marked';
let input = '';
process.stdin.on('data', d => input += d);
process.stdin.on('end', () => process.stdout.write(marked.parse(input)));
"""
    node_proc = subprocess.run(["node", "--input-type=module", "-e", node_script], input=test_input, text=True, capture_output=True)
    node_out = node_proc.stdout
    
    return rust_out, node_out

def is_diverging(test_input):
    r, n = get_outputs(test_input)
    return r != n

if not is_diverging(original_input):
    print("The provided input does NOT diverge. Nothing to minimize.")
    sys.exit(0)

print(f"Original length: {len(original_input)} chars")
current_input = original_input

# Greedy minimization
changed = True
while changed:
    changed = False
    for i in range(len(current_input)):
        # Try removing character at index i
        test_input = current_input[:i] + current_input[i+1:]
        if is_diverging(test_input):
            current_input = test_input
            changed = True
            print(f"Reduced to {len(current_input)} chars: {repr(current_input)}")
            break

print("\n--- MINIMIZED INPUT ---")
print(repr(current_input))
print("-----------------------")
r, n = get_outputs(current_input)
print("RUST: ", repr(r))
print("NODE: ", repr(n))
