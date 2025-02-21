import { Keypair, PublicKey } from '@solana/web3.js';
import { argon2id } from 'hash-wasm';
import { encryptPrivateKey } from './cli/src/commands/crypto';
import nacl from 'tweetnacl';
import { randomBytes } from 'crypto';
import { Buffer } from 'buffer';

// Program ID'yi tanımlayalım
const KEY_REGISTRY_PROGRAM_ID = new PublicKey('...'); // Gerçek program ID'nizi buraya ekleyin

// registerUserOnChain fonksiyonunu import edelim veya tanımlayalım
async function registerUserOnChain(params: {
  ecPubkey: Uint8Array;
  zklSig: Uint8Array;
  solanaSig: Uint8Array;
  solanaPubkey: PublicKey;
}) {
  // Implementasyon buraya gelecek
}

export async function initializeUser(password: string) {
  // 1. Generate EC keypair
  const ecKeypair = nacl.box.keyPair();
  
  // 2. Derive encryption key
  const salt = randomBytes(16);
  const encKey = await argon2id({
    password,
    salt: salt.toString('hex'), // Hash-wasm salt'ı string olarak bekler
    iterations: 3,
    parallelism: 1,
    memorySize: 4096,
    hashLength: 32,
  });

  // 3. Encrypt private key
  const encryptedPrivateKey = encryptPrivateKey(
    ecKeypair.secretKey,
    encKey
  );

  // 4. Generate Solana PDA
  const solanaKeypair = Keypair.generate();
  const [pda] = await PublicKey.findProgramAddress(
    [solanaKeypair.publicKey.toBuffer(), Buffer.from([0])],
    KEY_REGISTRY_PROGRAM_ID
  );

  // 5. Sign messages
  const message = Buffer.from(`ZKLAccount:${pda.toString()}`);
  const zklSig = nacl.sign.detached(message, ecKeypair.secretKey);
  const solanaSig = nacl.sign.detached(message, solanaKeypair.secretKey);

  // 6. Submit transaction
  await registerUserOnChain({
    ecPubkey: ecKeypair.publicKey,
    zklSig,
    solanaSig,
    solanaPubkey: solanaKeypair.publicKey,
  });
} 