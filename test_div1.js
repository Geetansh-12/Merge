const fs = require('fs');
fs.writeFileSync('test_div1.txt', '_-e_*_[');
console.log('Node:', JSON.stringify(require('marked').parse('_-e_*_[')));
const out = require('child_process').execSync('target\\\\release\\\\marked-rs test_div1.txt').toString();
console.log('Rust:', JSON.stringify(out));
