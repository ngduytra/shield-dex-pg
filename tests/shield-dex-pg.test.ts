import * as anchor from "@coral-xyz/anchor";
import { BN, IdlTypes, Program, utils, web3 } from "@coral-xyz/anchor";
import { ShieldDexPg } from "../target/types/shield_dex_pg";
import { expect } from "chai";

import Texture, { BILLION, HUNDRED, ONE, TEN, ZERO } from "./texture";
import { getLamports, getReferrerAddress } from "./utils";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

describe("shield-dex-pg", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.ShieldDexPg as Program<ShieldDexPg>;
  const texture = new Texture(provider);

  const STATE: Record<string, IdlTypes<ShieldDexPg>["PoolState"]> = {
    Uninitialized: { uninitialized: {} },
    Initialized: { initialized: {} },
    Paused: { paused: {} },
    Canceled: { canceled: {} },
  };

  const [escrowAB] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), texture.poolAB.publicKey.toBuffer()],
    program.programId
  );
  const [lpMintAB] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), texture.poolAB.publicKey.toBuffer()],
    program.programId
  );

  const [escrowBC] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), texture.poolBC.publicKey.toBuffer()],
    program.programId
  );
  const [lpMintBC] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("lp_mint"), texture.poolBC.publicKey.toBuffer()],
    program.programId
  );

  /**
   * Create three tokens A, B, C with all 1 billion supply. Their decimals are 6, 6, 9 respectively
   * There are 3 actors, Deployer, Alice, and Bob. Because the Deployer first minted all tokens to his balance, he need to faucet for Alice (1000 A, 1000 B, 1000 C), and Bob (1000 A, 1000 B, 1000 C). He also faucet for both 1 SOL.
   */
  before(async () => {
    await texture.init();
  });

  /**
   * Verify
   * A = 1_000_000_000 * 10^6
   * B = 1_000_000_000 * 10^6
   * B = 1_000_000_000 * 10^9
   */
  it("verify the mint texture", async () => {
    const A = await texture.spl.account.mint.fetch(texture.A.mint.publicKey);
    const B = await texture.spl.account.mint.fetch(texture.B.mint.publicKey);
    const C = await texture.spl.account.mint.fetch(texture.C.mint.publicKey);

    expect(A.decimals).equals(texture.A.decimals);
    expect(B.decimals).equals(texture.B.decimals);
    expect(C.decimals).equals(texture.C.decimals);
    expect(TEN.pow(new BN(texture.A.decimals)).mul(BILLION).eq(A.supply)).to.be
      .true;
    expect(TEN.pow(new BN(texture.B.decimals)).mul(BILLION).eq(B.supply)).to.be
      .true;
    expect(TEN.pow(new BN(texture.C.decimals)).mul(BILLION).eq(C.supply)).to.be
      .true;
  });

  it("create platform config", async () => {
    await program.methods
      .createPlatformConfig(new BN(2500000))
      .accounts({
        owner: provider.publicKey,
        platformConfig: texture.platformConfig.publicKey,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([texture.platformConfig])
      .rpc();

    const { tax } = await program.account.platformConfig.fetch(
      texture.platformConfig.publicKey
    );

    expect(tax.eq(new BN(2500000))).to.be.true;
  });

  it("initialized a pool of (A,B)", async () => {
    const platform = await program.account.platformConfig.fetch(
      texture.platformConfig.publicKey
    );
    await program.methods
      .initialize(
        texture.A.amount.init,
        texture.B.amount.init,
        // new BN(1000000000000),
        ZERO,
        ONE,
        ZERO
      )
      .accounts({
        authority: provider.publicKey,
        platformConfig: texture.platformConfig.publicKey,
        pool: texture.poolAB.publicKey,
        mintA: texture.A.mint.publicKey,
        srcA: utils.token.associatedAddress({
          owner: provider.publicKey,
          mint: texture.A.mint.publicKey,
        }),
        treasuryA: utils.token.associatedAddress({
          owner: escrowAB,
          mint: texture.A.mint.publicKey,
        }),
        mintB: texture.B.mint.publicKey,
        srcB: utils.token.associatedAddress({
          owner: provider.publicKey,
          mint: texture.B.mint.publicKey,
        }),
        treasuryB: utils.token.associatedAddress({
          owner: escrowAB,
          mint: texture.B.mint.publicKey,
        }),
        lpMint: lpMintAB,
        dstLp: utils.token.associatedAddress({
          owner: provider.publicKey,
          mint: lpMintAB,
        }),
        escrow: escrowAB,
        taxman: provider.publicKey,
        tokenProgram: utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([texture.poolAB])
      .rpc();

    const { authority, mintA, mintB, lpFee, tax, state } =
      await program.account.pool.fetch(texture.poolAB.publicKey);
    const { amount: a } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        owner: escrowAB,
        mint: texture.A.mint.publicKey,
      })
    );
    const { amount: b } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        owner: escrowAB,
        mint: texture.B.mint.publicKey,
      })
    );
    const { amount: lp } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        owner: provider.publicKey,
        mint: lpMintAB,
      })
    );

    const balance = await getLamports(texture.taxman.publicKey, provider);

    expect(authority).deep.equal(provider.publicKey);
    expect(mintA).deep.equal(texture.A.mint.publicKey);
    expect(mintB).deep.equal(texture.B.mint.publicKey);
    expect(lpFee.eq(ZERO)).to.be.true;
    expect(state).deep.equal(STATE.Initialized);
    expect(a.eq(texture.A.amount.init)).to.be.true;
    expect(b.eq(texture.B.amount.init)).to.be.true;
    expect(lp.toString()).equal("1000000000000");
  });

  it("create referrer", async () => {
    const [referrer] = await getReferrerAddress(
      provider.publicKey,
      program.programId
    );

    await program.methods
      .createReferrer(texture.Bob.keypair.publicKey)
      .accounts({
        authority: provider.publicKey,
        pool: texture.poolAB.publicKey,
        referrer: referrer,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const { owner } = await program.account.referrer.fetch(referrer);

    console.log("owner.toBase58(): ", owner.toBase58());

    expect(owner).deep.eq(texture.Bob.keypair.publicKey);
  });

  it("Bob adds liquidity in the pool of (A,B)", async () => {
    await program.methods
      .addLiquidity(texture.A.amount.deposit, texture.B.amount.deposit)
      .accounts({
        authority: texture.Bob.keypair.publicKey,
        pool: texture.poolAB.publicKey,
        mintA: texture.A.mint.publicKey,
        srcA: texture.Bob.tokenAccount(texture.A.mint.publicKey),
        treasuryA: utils.token.associatedAddress({
          mint: texture.A.mint.publicKey,
          owner: escrowAB,
        }),
        mintB: texture.B.mint.publicKey,
        srcB: texture.Bob.tokenAccount(texture.B.mint.publicKey),
        treasuryB: utils.token.associatedAddress({
          mint: texture.B.mint.publicKey,
          owner: escrowAB,
        }),
        lpMint: lpMintAB,
        dstLp: texture.Bob.tokenAccount(lpMintAB),
        escrow: escrowAB,
        tokenProgram: utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([texture.Bob.keypair])
      .rpc();
    await program.methods
      .addLiquidity(texture.A.amount.deposit, texture.B.amount.deposit)
      .accounts({
        authority: texture.Bob.keypair.publicKey,
        pool: texture.poolAB.publicKey,
        mintA: texture.A.mint.publicKey,
        srcA: texture.Bob.tokenAccount(texture.A.mint.publicKey),
        treasuryA: utils.token.associatedAddress({
          mint: texture.A.mint.publicKey,
          owner: escrowAB,
        }),
        mintB: texture.B.mint.publicKey,
        srcB: texture.Bob.tokenAccount(texture.B.mint.publicKey),
        treasuryB: utils.token.associatedAddress({
          mint: texture.B.mint.publicKey,
          owner: escrowAB,
        }),
        lpMint: lpMintAB,
        dstLp: texture.Bob.tokenAccount(lpMintAB),
        escrow: escrowAB,
        tokenProgram: utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([texture.Bob.keypair])
      .rpc();

    const { amount } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(lpMintAB)
    );

    expect(amount.toString()).equal("400000000");
  });

  it("Bob removes liquidity out the pool of (A,B)", async () => {
    // Prev states
    const { amount: prevLP } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(lpMintAB)
    );
    const { amount: prevA } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(texture.A.mint.publicKey)
    );
    const { amount: prevB } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(texture.B.mint.publicKey)
    );

    const { amount: vaultA } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        mint: texture.A.mint.publicKey,
        owner: escrowAB,
      })
    );

    const { amount: vaultB } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        mint: texture.B.mint.publicKey,
        owner: escrowAB,
      })
    );

    // Remove liquidity
    const lp = HUNDRED.mul(new BN(1000000));

    await program.methods
      .removeLiquidity(lp)
      .accounts({
        authority: texture.Bob.keypair.publicKey,
        pool: texture.poolAB.publicKey,
        mintA: texture.A.mint.publicKey,
        treasuryA: utils.token.associatedAddress({
          mint: texture.A.mint.publicKey,
          owner: escrowAB,
        }),
        dstA: texture.Bob.tokenAccount(texture.A.mint.publicKey),
        mintB: texture.B.mint.publicKey,
        treasuryB: utils.token.associatedAddress({
          mint: texture.B.mint.publicKey,
          owner: escrowAB,
        }),
        dstB: texture.Bob.tokenAccount(texture.B.mint.publicKey),
        lpMint: lpMintAB,
        srcLp: texture.Bob.tokenAccount(lpMintAB),
        escrow: escrowAB,
        tokenProgram: utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([texture.Bob.keypair])
      .rpc({ skipPreflight: true });
    // Next states
    const { amount: nextLP } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(lpMintAB)
    );
    const { amount: nextA } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(texture.A.mint.publicKey)
    );
    const { amount: nextB } = await texture.spl.account.account.fetch(
      texture.Bob.tokenAccount(texture.B.mint.publicKey)
    );

    expect(prevLP.sub(nextLP).eq(lp)).to.be.true;
    expect(nextA.sub(prevA).toString()).equal("100000000");
    expect(nextB.sub(prevB).toString()).equal("100000000");
  });

  // /**
  //  * Alice swap 100 * 10^6 A to ? B
  //  */
  it("Alice swaps A to B", async () => {
    const poolAA = await program.account.pool.fetch(texture.poolAB.publicKey);
    console.log("poolAA", poolAA.tax.toBase58());
    console.log(
      "texture.platformConfig.publicKey: ",
      texture.platformConfig.publicKey.toBase58()
    );
    // Previous state
    const { amount: prevA } = await texture.spl.account.account.fetch(
      texture.Alice.tokenAccount(texture.A.mint.publicKey)
    );
    const { amount: prevB } = await texture.spl.account.account.fetch(
      texture.Alice.tokenAccount(texture.B.mint.publicKey)
    );
    // const prevTax = ZERO;
    const { amount: prevTax } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        mint: texture.A.mint.publicKey,
        owner: new web3.PublicKey(
          "CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1"
        ),
      })
    );
    // Swap
    const bidAmount = TEN.pow(new BN(texture.A.decimals)).mul(HUNDRED);
    const limit = new BN(0); // Free slipagge rate
    await program.methods
      .swap(bidAmount, limit)
      .accounts({
        authority: texture.Alice.keypair.publicKey,
        pool: texture.poolAB.publicKey,
        platformConfig: texture.platformConfig.publicKey,
        bidMint: texture.A.mint.publicKey,
        bidSrc: texture.Alice.tokenAccount(texture.A.mint.publicKey),
        bidTreasury: utils.token.associatedAddress({
          mint: texture.A.mint.publicKey,
          owner: escrowAB,
        }),
        askMint: texture.B.mint.publicKey,
        askTreasury: utils.token.associatedAddress({
          mint: texture.B.mint.publicKey,
          owner: escrowAB,
        }),
        askDst: texture.Alice.tokenAccount(texture.B.mint.publicKey),
        escrow: escrowAB,
        taxman: new web3.PublicKey(
          "CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1"
        ),
        taxDst: utils.token.associatedAddress({
          mint: texture.A.mint.publicKey,
          owner: new web3.PublicKey(
            "CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1"
          ),
        }),
        tokenProgram: utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([texture.Alice.keypair])
      .rpc();
    // Next state
    const { amount: nextA } = await texture.spl.account.account.fetch(
      texture.Alice.tokenAccount(texture.A.mint.publicKey)
    );
    const { amount: nextB } = await texture.spl.account.account.fetch(
      texture.Alice.tokenAccount(texture.B.mint.publicKey)
    );
    const { amount: nextTax } = await texture.spl.account.account.fetch(
      utils.token.associatedAddress({
        mint: texture.A.mint.publicKey,
        owner: new web3.PublicKey(
          "CVkbpNdrD1hb6TDwiyaoEyrDUft4T7aM5PQifmtCnGb1"
        ),
      })
    );

    console.log("prevTax, nextTax: ", prevTax.toNumber(), nextTax.toNumber());

    expect(prevA.sub(nextA).eq(bidAmount)).to.be.true;
    expect(nextB.sub(prevB).toString()).equal("99740054");
    expect(nextTax.sub(prevTax).toString()).equal("250000");
  });

  // it("update fee in the pool of (A,B)", async () => {
  //   await program.methods
  //     .updateFee(texture.fee)
  //     .accounts({
  //       authority: provider.publicKey,
  //       pool: texture.poolAB.publicKey,
  //     })
  //     .rpc({ skipPreflight: true });

  //   const { lpFee } = await program.account.pool.fetch(
  //     texture.poolAB.publicKey
  //   );

  //   expect(lpFee.eq(texture.fee)).to.be.true;
  // });

  // // it("update tax in the pool of (A,B)", async () => {
  // //   await program.methods
  // //     .updateTax(texture.tax)
  // //     .accounts({
  // //       authority: provider.publicKey,
  // //       pool: texture.poolAB.publicKey,
  // //     })
  // //     .rpc({ skipPreflight: true });

  // //   const { tax } = await program.account.pool.fetch(texture.poolAB.publicKey);

  // //   expect(tax.eq(texture.tax)).to.be.true;
  // // });

  // it("pause the pool of (A,B)", async () => {
  //   await program.methods
  //     .pause()
  //     .accounts({
  //       authority: texture.Alice.keypair.publicKey,
  //       pool: texture.poolAB.publicKey,
  //     })
  //     .signers([texture.Alice.keypair])
  //     .rpc({ skipPreflight: true });

  //   const { state } = await program.account.pool.fetch(
  //     texture.poolAB.publicKey
  //   );

  //   expect(state).deep.equal(STATE.Paused);
  // });

  // it("resume the pool of (A,B)", async () => {
  //   await program.methods
  //     .resume()
  //     .accounts({
  //       authority: texture.Alice.keypair.publicKey,
  //       pool: texture.poolAB.publicKey,
  //     })
  //     .signers([texture.Alice.keypair])
  //     .rpc({ skipPreflight: true });

  //   const { state } = await program.account.pool.fetch(
  //     texture.poolAB.publicKey
  //   );

  //   expect(state).deep.equal(STATE.Initialized);
  // });

  // it("initialized a pool of (B,C)", async () => {
  //   await program.methods
  //     .initialize(
  //       texture.B.amount.init,
  //       texture.C.amount.init,
  //       ZERO,
  //       ONE,
  //       texture.fee,
  //       texture.tax
  //     )
  //     .accounts({
  //       authority: provider.publicKey,
  //       pool: texture.poolBC.publicKey,
  //       mintA: texture.B.mint.publicKey,
  //       srcA: utils.token.associatedAddress({
  //         owner: provider.publicKey,
  //         mint: texture.B.mint.publicKey,
  //       }),
  //       treasuryA: utils.token.associatedAddress({
  //         owner: escrowBC,
  //         mint: texture.B.mint.publicKey,
  //       }),
  //       mintB: texture.C.mint.publicKey,
  //       srcB: utils.token.associatedAddress({
  //         owner: provider.publicKey,
  //         mint: texture.C.mint.publicKey,
  //       }),
  //       treasuryB: utils.token.associatedAddress({
  //         owner: escrowBC,
  //         mint: texture.C.mint.publicKey,
  //       }),
  //       lpMint: lpMintBC,
  //       dstLp: utils.token.associatedAddress({
  //         owner: provider.publicKey,
  //         mint: lpMintBC,
  //       }),
  //       escrow: escrowBC,
  //       taxman: texture.taxman.publicKey,
  //       tokenProgram: utils.token.TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: utils.token.ASSOCIATED_PROGRAM_ID,
  //       systemProgram: web3.SystemProgram.programId,
  //       rent: web3.SYSVAR_RENT_PUBKEY,
  //     })
  //     .signers([texture.poolBC])
  //     .rpc({ skipPreflight: true });

  //   const { authority, lpMint, mintA, mintB, lpFee, tax, state } =
  //     await program.account.pool.fetch(texture.poolBC.publicKey);
  //   const { amount: a } = await texture.spl.account.account.fetch(
  //     utils.token.associatedAddress({
  //       owner: escrowBC,
  //       mint: texture.B.mint.publicKey,
  //     })
  //   );
  //   const { amount: b } = await texture.spl.account.account.fetch(
  //     utils.token.associatedAddress({
  //       owner: escrowBC,
  //       mint: texture.C.mint.publicKey,
  //     })
  //   );
  //   const { amount: lp } = await texture.spl.account.account.fetch(
  //     utils.token.associatedAddress({
  //       owner: provider.publicKey,
  //       mint: lpMintBC,
  //     })
  //   );

  //   expect(authority).deep.equal(provider.publicKey);
  //   expect(lpMint).deep.equal(lpMintBC);
  //   expect(mintA).deep.equal(texture.B.mint.publicKey);
  //   expect(mintB).deep.equal(texture.C.mint.publicKey);
  //   expect(lpFee.eq(texture.fee)).to.be.true;
  //   // expect(tax.eq(texture.tax)).to.be.true;
  //   // expect(taxman).deep.equal(texture.taxman.publicKey);
  //   expect(state).deep.equal(STATE.Initialized);
  //   expect(a.eq(texture.B.amount.init)).to.be.true;
  //   expect(b.eq(texture.C.amount.init)).to.be.true;
  //   expect(lp.toString()).equal("31622776601683");
  // });
});
