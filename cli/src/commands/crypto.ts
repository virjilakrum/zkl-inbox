import { createCipheriv, randomBytes } from 'crypto';

export function encryptPrivateKey(
  privateKey: Uint8Array,
  encryptionKey: string
): string {
  const iv = randomBytes(16);
  const cipher = createCipheriv('aes-256-gcm', 
    Buffer.from(encryptionKey, 'hex'), 
    iv
  );
  
  const encrypted = Buffer.concat([
    cipher.update(privateKey),
    cipher.final()
  ]);
  
  return iv.toString('hex') + ':' + encrypted.toString('hex');
}

// Şifreleme fonksiyonlarını içeren modül
export * from './crypto'; 