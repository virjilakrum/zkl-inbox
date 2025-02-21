# zkλ CLI

zkλ CLI (v2 "sulli"), Ethereum ve Solana zincirleri arasında güvenli dosya transferi sağlayan bir komut satırı aracıdır.

## Özellikler

- Yerel IPFS kullanarak merkezi olmayan dosya depolama
- Wormhole v2 ile çoklu zincir desteği (Sepolia ve Solana Devnet)
- ECIES benzeri hibrit şifreleme ile uçtan uca güvenlik
- Solana Program-Derived Addresses (PDA) ile anahtar kaydı
- Siyah arka plan ve #feffaf metin rengi ile terminal arayüzü

## Gereksinimler

- Node.js (v18 veya üstü)
- Yerel IPFS düğümü (v0.20.0 veya üstü)
- Solana CLI (v1.17 veya üstü)
- Ethereum düğümü erişimi (Infura Sepolia veya yerel)

## Kurulum

1. Bağımlılıkları yükleyin:
```bash
npm install
```

2. Ortam değişkenlerini ayarlayın:
```bash
cp .env.example .env
```

3. `.env` dosyasını düzenleyin:
```env
# Ethereum
INFURA_KEY=your_infura_api_key_here
ETH_PRIVATE_KEY=your_ethereum_private_key_here

# Solana
SOLANA_PRIVATE_KEY=your_solana_private_key_here
```

4. CLI'yi global olarak yükleyin:
```bash
npm link
```

## Kullanım

### Hesap Oluşturma
```bash
zkl init --password "güvenli_şifreniz"
```

### Dosya Gönderme (Ethereum Sepolia'dan Solana Devnet'e)
```bash
zkl send --from ethereum --to solana --file ./dosya.txt --recipient "alıcı_ec_pubkey" --password "şifreniz"
```

### Dosya Alma
```bash
zkl receive --tx '{"signature":"...","ipfsCid":"...","ephemeralPubkey":"...","fileHash":"..."}' --output ./alınan_dosya.txt --password "şifreniz"
```

## Test Ağları

- Ethereum: Sepolia Test Ağı
  - Faucet: https://sepoliafaucet.com
  - Explorer: https://sepolia.etherscan.io

- Solana: Devnet
  - Faucet: `solana airdrop 1 <ADRES> --url https://api.devnet.solana.com`
  - Explorer: https://explorer.solana.com/?cluster=devnet

## Güvenlik

- EC özel anahtarları Argon2id ile türetilen simetrik anahtarlarla şifrelenir
- Dosyalar ECIES benzeri hibrit şifreleme ile korunur
- Brute-force saldırılarına karşı koruma mekanizması
- İmzalar ve hash doğrulaması ile bütünlük kontrolü
- Hassas bilgiler `.env` dosyasında güvenli bir şekilde saklanır

## Lisans

MIT Lisansı altında dağıtılmaktadır. Daha fazla bilgi için [LICENSE](LICENSE) dosyasına bakın.