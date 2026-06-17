node -e "require('fs').writeFileSync('fuzz/input.txt', '-\\\\b!(na]\\\\c\\\\])\\\\\\\\re*\`(d[b\`\\\\>!de(dn[)d)!\`\\\\rr>*_*[nd_e[\`e\\\\ ')"
Get-Content fuzz/input.txt | .\target\release\marked-rs.exe > test_out4.txt
node -e "console.log(JSON.stringify(require('fs').readFileSync('test_out4.txt', 'utf8')))"
