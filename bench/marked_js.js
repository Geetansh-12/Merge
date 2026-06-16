const fs = require('fs');
const marked = require('marked');
const text = fs.readFileSync(process.argv[2], 'utf8');
marked.parse(text);
