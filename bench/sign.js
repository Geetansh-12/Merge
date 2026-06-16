const fs = require('fs');
const openpgp = require('openpgp');

async function sign() {
    const { privateKey, publicKey } = await openpgp.generateKey({
        type: 'ecc', curve: 'curve25519',
        userIDs: [{ name: 'Port Mortem', email: 'admin@port-mortem.org' }]
    });
    fs.writeFileSync('tests/pubkey.asc', publicKey);

    const text = fs.readFileSync('tests/original/specs/commonmark_spec.json', 'utf8');
    const privateKeyObj = await openpgp.readPrivateKey({ armoredKey: privateKey });

    const cleartextMessage = await openpgp.createCleartextMessage({ text: text });
    const signed = await openpgp.sign({
        message: cleartextMessage,
        signingKeys: privateKeyObj
    });

    fs.writeFileSync('tests/spec.json.asc', signed);
    console.log('Signed!');
}
sign().catch(console.error);
