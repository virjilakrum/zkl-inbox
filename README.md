```bash
Building...
$ warning: unused variable: `index`
  --> src/lib.rs:13:9
   |
13 |         index: u32, // PDA seed'inde client-side'da kullanılacak
   |         ^^^^^ help: if this is intentional, prefix it with an underscore: `_index`
   |
   = note: `#[warn(unused_variables)]` on by default
 
warning: `zkl-1` (lib) generated 1 warning (run `cargo fix --lib -p zkl-1` to apply 1 suggestion)
Build successful. Completed in 5.91s.
```

Deploying... This could take a while depending on the program size and network conditions.
$ Deployment successful. Completed in 1m.
https://explorer.solana.com/tx/3BCWsedbnByza36UAMmd6WmAkQYEpwxgvAQN5L1bQ3E2UZomTyke76eh8Q3QU8D6BSrz9rjRKBhuemsZVEKFPG1W?cluster=devnet
