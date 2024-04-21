import { AnchorProvider, BN, Wallet, utils, web3 } from "@coral-xyz/anchor";
import { splTokenProgram } from "@coral-xyz/spl-token";
import {
  initializeAccount,
  initializeMint,
  mintTo,
  transferLamports,
} from "./utils";

export const ZERO = new BN(0);
export const ONE = new BN(1);
export const TEN = new BN(10);
export const HUNDRED = new BN(100);
export const THOUSAND = new BN(1000);
export const MILLION = new BN(1000000);
export const BILLION = new BN(1000000000);
export const PRECISION = BILLION;

export class Trader {
  public readonly spl: ReturnType<typeof splTokenProgram>;

  constructor(
    public readonly connection: web3.Connection,
    public readonly keypair = new web3.Keypair()
  ) {
    this.spl = splTokenProgram({
      provider: new AnchorProvider(this.connection, new Wallet(this.keypair), {
        commitment: "confirmed",
      }),
    });
  }

  tokenAccount = (mint: web3.PublicKey) => {
    return utils.token.associatedAddress({
      mint,
      owner: this.keypair.publicKey,
    });
  };
}

export function getTokenPreset(decimals: number) {
  const precision = TEN.pow(new BN(decimals));
  return {
    mint: new web3.Keypair(),
    decimals,
    supply: BILLION.mul(precision), // 1_000_000_000
    amount: {
      init: MILLION.mul(precision), // 1_000_000,
      deposit: HUNDRED.mul(new BN(2)).mul(precision), // 200,
    },
  };
}

export default class Texture {
  public readonly spl: ReturnType<typeof splTokenProgram>;
  public fee = new BN(2500000);
  public tax = new BN(1500000);

  public platformConfig = new web3.Keypair();
  // Referrer
  public referrer = new web3.Keypair();

  public poolAB = new web3.Keypair();
  public poolBC = new web3.Keypair();
  public taxman = new web3.Keypair();

  // Tokens
  public A = getTokenPreset(6);
  public B = getTokenPreset(6);
  public C = getTokenPreset(9);

  // Traders
  public readonly Alice: Trader;
  public readonly Bob: Trader;

  constructor(public readonly provider: AnchorProvider) {
    this.spl = splTokenProgram({ provider });
    this.Alice = new Trader(this.provider.connection);
    this.Bob = new Trader(this.provider.connection);
  }

  async init() {
    // Init token A
    await initializeMint(this.A.decimals, this.A.mint, this.spl);
    await initializeAccount(this.A.mint.publicKey, this.spl);
    await mintTo(this.A.supply, this.A.mint.publicKey, this.spl);
    // Init token B
    await initializeMint(this.B.decimals, this.B.mint, this.spl);
    await initializeAccount(this.B.mint.publicKey, this.spl);
    await mintTo(this.B.supply, this.B.mint.publicKey, this.spl);
    // Init token C
    await initializeMint(this.C.decimals, this.C.mint, this.spl);
    await initializeAccount(this.C.mint.publicKey, this.spl);
    await mintTo(this.C.supply, this.C.mint.publicKey, this.spl);

    // Faucet to Alice
    await transferLamports(
      2 * web3.LAMPORTS_PER_SOL,
      this.Alice.keypair.publicKey,
      this.provider
    );
    await initializeAccount(this.A.mint.publicKey, this.Alice.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.A.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.A.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Alice.tokenAccount(this.A.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();
    await initializeAccount(this.B.mint.publicKey, this.Alice.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.B.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.B.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Alice.tokenAccount(this.B.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();
    await initializeAccount(this.C.mint.publicKey, this.Alice.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.C.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.C.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Alice.tokenAccount(this.C.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();

    // Faucet to Bob
    await transferLamports(
      2 * web3.LAMPORTS_PER_SOL,
      this.Bob.keypair.publicKey,
      this.provider
    );
    await initializeAccount(this.A.mint.publicKey, this.Bob.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.A.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.A.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Bob.tokenAccount(this.A.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();
    await initializeAccount(this.B.mint.publicKey, this.Bob.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.B.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.B.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Bob.tokenAccount(this.B.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();
    await initializeAccount(this.C.mint.publicKey, this.Bob.spl);
    await this.spl.methods
      .transfer(TEN.pow(new BN(this.C.decimals)).mul(THOUSAND)) // 1k
      .accounts({
        source: utils.token.associatedAddress({
          mint: this.C.mint.publicKey,
          owner: this.provider.publicKey,
        }),
        destination: this.Bob.tokenAccount(this.C.mint.publicKey),
        authority: this.provider.publicKey,
      })
      .rpc();
  }
}
