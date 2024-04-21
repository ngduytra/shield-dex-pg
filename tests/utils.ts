import { web3, BN, utils, Provider } from "@coral-xyz/anchor";
import { splTokenProgram } from "@coral-xyz/spl-token";

export const REFERRER_SEED = Buffer.from("referrer");

export const asyncWait = (s: number) =>
  new Promise((resolve) => setTimeout(resolve, s * 1000));

export const getCurrentTimestamp = async (
  connection: web3.Connection
): Promise<number> => {
  const { data } =
    (await connection.getAccountInfo(web3.SYSVAR_CLOCK_PUBKEY)) || {};
  if (!data) throw new Error("Cannot read clock data");
  const bn = new BN(data.subarray(32, 40), "le");
  return bn.toNumber();
};

export const initializeMint = async (
  decimals: number,
  token: web3.Keypair,
  splProgram: ReturnType<typeof splTokenProgram>
) => {
  const provider = splProgram.provider;
  if (!provider.publicKey || !provider.sendAndConfirm)
    throw new Error("Invalid wallet");
  await splProgram.methods
    .initializeMint(decimals, provider.publicKey, provider.publicKey)
    .accounts({
      mint: token.publicKey,
      rent: web3.SYSVAR_RENT_PUBKEY,
    })
    .preInstructions([await splProgram.account.mint.createInstruction(token)])
    .signers([token])
    .rpc({ maxRetries: 5 });
};

export async function getReferrerAddress(
  authority: web3.PublicKey,
  programId: web3.PublicKey
): Promise<[web3.PublicKey, number]> {
  const [address, bump] = web3.PublicKey.findProgramAddressSync(
    [REFERRER_SEED, authority.toBuffer()],
    programId
  );

  return [address, bump];
}

export const initializeAccount = async (
  mint: web3.PublicKey,
  splProgram: ReturnType<typeof splTokenProgram>
) => {
  const provider = splProgram.provider;
  if (!provider.publicKey || !provider.sendAndConfirm)
    throw new Error("Invalid wallet");
  const account = utils.token.associatedAddress({
    mint,
    owner: provider.publicKey,
  });
  const ix = new web3.TransactionInstruction({
    keys: [
      {
        pubkey: provider.publicKey,
        isSigner: true,
        isWritable: true,
      },
      {
        pubkey: account,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: provider.publicKey,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: mint,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: utils.token.TOKEN_PROGRAM_ID,
        isSigner: false,
        isWritable: false,
      },
      {
        pubkey: web3.SYSVAR_RENT_PUBKEY,
        isSigner: false,
        isWritable: false,
      },
    ],
    programId: utils.token.ASSOCIATED_PROGRAM_ID,
    data: Buffer.from([]),
  });
  const tx = new web3.Transaction().add(ix);
  return await provider.sendAndConfirm(tx, undefined, { maxRetries: 5 });
};

export const mintTo = async (
  amount: BN,
  mint: web3.PublicKey,
  splProgram: ReturnType<typeof splTokenProgram>
) => {
  const provider = splProgram.provider;
  if (!provider.publicKey) throw new Error("Invalid wallet");
  const account = utils.token.associatedAddress({
    mint: mint,
    owner: provider.publicKey,
  });
  await splProgram.methods
    .mintTo(amount)
    .accounts({
      mint,
      account,
      owner: provider.publicKey,
    })
    .rpc({ maxRetries: 5 });
};

export const transferLamports = async (
  lamports: number,
  dstAddress: web3.PublicKey,
  provider: Provider
) => {
  if (!provider.publicKey || !provider.sendAndConfirm)
    throw new Error("Invalid wallet");
  const ix = web3.SystemProgram.transfer({
    fromPubkey: provider.publicKey,
    toPubkey: dstAddress,
    lamports: Number(lamports),
  });
  const tx = new web3.Transaction().add(ix);
  return await provider.sendAndConfirm(tx, undefined, { maxRetries: 5 });
};

export const getLamports = async (
  address: web3.PublicKey,
  provider: Provider
) => {
  if (!provider.publicKey || !provider.sendAndConfirm)
    throw new Error("Invalid wallet");

  return await provider.connection.getBalance(address);
};
