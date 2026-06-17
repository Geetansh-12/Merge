import { marked } from 'marked';
import { execFileSync } from 'child_process';
import fs from 'fs';

const duration = parseInt(process.argv[2] || '60', 10);
const exePath = process.platform === 'win32' ? 'target/release/marked-rs.exe' : 'target/release/marked-rs';

if (!fs.existsSync(exePath)) {
    console.error(`Executable not found at ${exePath}. Did you run cargo build --release?`);
    process.exit(1);
}

const pathologicalPatterns = [
    () => '[[[[[[[[[foo]]]]]]]]]',
    () => '*_*_*_*_*_*_*_',
    () => '<div\nclass="foo"\n>\n&amp;&lt;&gt;&quot;',
    () => '\\ '.repeat(50),
    () => '```'.repeat(20),
    () => '> '.repeat(50) + 'foo',
    () => '- '.repeat(20) + 'foo',
    () => '[' + 'x'.repeat(20) + '](<foo\\nbar>)',
    () => {
        const chars = 'abcde #*_`[]()!->\\n\\r\t&;"\'+=~|';
        return Array.from({length: Math.floor(Math.random()*100)+1},
            () => chars[Math.floor(Math.random()*chars.length)]).join('');
    }
];

let runs = 0;
let divergences = 0;
let panics = 0;
let mismatches = [];

console.log(`Running differential fuzz for ${duration}s...`);
const start = Date.now();

while (Date.now() - start < duration * 1000) {
    const fn = pathologicalPatterns[Math.floor(Math.random() * pathologicalPatterns.length)];
    const input = fn();

    let rustOut;
    try {
        rustOut = execFileSync(exePath, [], {
            input: input,
            encoding: 'utf8',
            stdio: ['pipe', 'pipe', 'ignore'],
            timeout: 1000
        });
    } catch (e) {
        panics++;
        console.log(`PANIC on run ${runs + 1}`);
        console.log(`Input: ${JSON.stringify(input)}`);
        runs++;
        continue;
    }

    const nodeOut = marked.parse(input);

    if (rustOut !== nodeOut) {
        divergences++;
        // MINIMIZER: try removing 1 character at a time to shrink the divergence
        let minInput = input;
        let shrunk = true;
        while (shrunk && minInput.length > 2) {
            shrunk = false;
            for (let i = 0; i < minInput.length; i++) {
                let testInput = minInput.substring(0, i) + minInput.substring(i + 1);
                try {
                    let rustTest = execFileSync(exePath, [], { input: testInput, encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'], timeout: 1000 });
                    let nodeTest = marked.parse(testInput);
                    if (rustTest !== nodeTest) {
                        minInput = testInput;
                        shrunk = true;
                        break;
                    }
                } catch (e) {}
            }
        }
        mismatches.push({
            input: minInput,
            rust: execFileSync(exePath, [], { input: minInput, encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'], timeout: 1000 }),
            node: marked.parse(minInput)
        });
    } else {
        // MUTATION: randomly insert delimiter chars into successful input
        if (input.length < 50 && pathologicalPatterns.length < 1000) {
            const chars = '*_`[]()!->\\n';
            const charToInsert = chars[Math.floor(Math.random()*chars.length)];
            const pos = Math.floor(Math.random() * (input.length + 1));
            const mutant = input.substring(0, pos) + charToInsert + input.substring(pos);
            pathologicalPatterns.push(() => mutant);
        }
    }

    runs++;
    if (runs % 500 === 0) {
        const elapsed = (Date.now() - start) / 1000;
        const runsPerSec = (runs / elapsed).toFixed(2);
        console.log(`Progress: ${runs} runs, ${divergences} divergences, ${elapsed.toFixed(1)}s elapsed (${runsPerSec} runs/s)`);
    }
}

const totalTime = (Date.now() - start) / 1000;

console.log(`\n=== FINAL RESULTS ===`);
console.log(`Duration:    ${totalTime.toFixed(2)}s`);
console.log(`Total runs:  ${runs}`);
console.log(`Speed:       ${(runs / totalTime).toFixed(2)} runs/second`);
console.log(`Panics:      ${panics}`);
console.log(`Divergences: ${divergences}`);
console.log(`Finished:    ${new Date().toISOString()}`);

if (divergences === 0 && panics === 0) {
    console.log(`STATUS: PASS - Zero divergences, zero panics`);
} else {
    console.log(`STATUS: FAIL`);
    if (mismatches.length > 0) {
        fs.writeFileSync('fuzz/divergences.json', JSON.stringify(mismatches.slice(0, 50), null, 2));
        console.log(`Saved up to 50 divergences in fuzz/divergences.json`);
    }
}
