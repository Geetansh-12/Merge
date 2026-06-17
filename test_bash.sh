echo "<p>-\b!(na]\c\])\re*<code>(d[b</code>&gt;!de(dn[)d)!<code>\rr&gt;*_*[nd_e[</code>e\ </p>" | od -c
echo "<p>-\b!(na]\c\])\re*<code>(d[b</code>&gt;!de(dn[)d)!<code>\rr&gt;*_*[nd_e[</code>e\ </p>" > test_echo_out.txt
node -e "console.log(JSON.stringify(require('fs').readFileSync('test_echo_out.txt', 'utf8')))"
