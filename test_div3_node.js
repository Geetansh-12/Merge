const { marked } = require('marked');
const fs = require('fs');
let input = "b_]*aedern>[!_[er(n\\`!)#n_)nc\\er-\\r**b#a(-re\\)` \\!`c]!d)]";
console.log(marked.parse(input));
