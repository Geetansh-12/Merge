const fs = require('fs');
const input = "`e]    *\\ed\\! )\n[ee*[  _[  d_     _)\n> _](e *  e (] `";
fs.writeFileSync('test_codespan_multiline.txt', input);
const output = require('child_process').execSync('target\\\\release\\\\marked-rs test_codespan_multiline.txt').toString();
console.log('Rust:', JSON.stringify(output));
console.log('Node:', JSON.stringify(require('marked').parse(input)));
