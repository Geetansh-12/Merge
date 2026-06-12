const patterns = [
    () => '# Hello\n\nParagraph with **bold** and *italic*.',
    () => Array(Math.floor(Math.random()*5)+1).fill('*').join('') + 
          'text' + 
          Array(Math.floor(Math.random()*5)+1).fill('*').join(''),
    () => Array(Math.floor(Math.random()*8)+1).fill('>').join(' ') + ' text',
    () => '| a | b |\n|---|---|\n| c | d |',
    () => '[' + 'x'.repeat(Math.floor(Math.random()*20)) + ']()',
    () => '\`'.repeat(Math.floor(Math.random()*3)+1) + 
          'code' + 
          '\`'.repeat(Math.floor(Math.random()*3)+1),
    () => {
        const chars = 'abcde #*_`[]()!->\\n\\r';
        return Array.from({length: Math.floor(Math.random()*300)+1},
            () => chars[Math.floor(Math.random()*chars.length)]).join('');
    }
];
const fn = patterns[Math.floor(Math.random()*patterns.length)];
process.stdout.write(fn());
