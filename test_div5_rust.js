const input = "_\\n)!(na>  c`e]    *\\ed\\! )\\n[ee*[  _[  d_     _)\\n> _](e *  e (] `";
require('fs').writeFileSync('test_div5_rust.txt', input);
const out = require('child_process').execSync('target\\\\release\\\\marked-rs test_div5_rust.txt').toString();
console.log('Rust:', JSON.stringify(out));
