import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Airdrop } from "../target/types/airdrop";
import { TOKEN_PROGRAM_ID, Token, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { PublicKey, SystemProgram, Transaction, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import * as assert from "assert";

describe("airdrop", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Airdrop as Program<Airdrop>;
  const provider = anchor.AnchorProvider.env();

  let mint = null;
  let initializerTokenAccount = null;
  let takerTokenAccountA = null;
  let takerTokenAccountB = null;

  const payer = anchor.web3.Keypair.generate();
  const mintAuthority = anchor.web3.Keypair.generate();
  const initializerMainAccount = anchor.web3.Keypair.generate();
  const takerMainAccountA = anchor.web3.Keypair.generate();
  const takerMainAccountB = anchor.web3.Keypair.generate();

  console.log(Token);

  it("Is initialized!", async () => {
    // Add your test here.
    //const tx = await program.methods.initialize().rpc();
    //console.log("Your transaction signature", tx);

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(payer.publicKey, 10000000000),
      "confirmed"
    );

    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(initializerMainAccount.publicKey, 10000000000),
      "confirmed"
    );

    mint = await createMint(
      provider.connection,
      payer,
      mintAuthority.publicKey,
      null,
      0
    );

    initializerTokenAccount = await createAccount(
        provider.connection,
        payer,
        mint,
        initializerMainAccount.publicKey
    );

    takerTokenAccountA = await createAccount(
        provider.connection,
        payer,
        mint,
        takerMainAccountA.publicKey
    );
    // takerTokenAccountB = await mint.createAccount(takerMainAccountB.publicKey);

    await mintTo(
        provider.connection,
        payer,
        mint,
        initializerTokenAccount,
        mintAuthority,
        1000
    );

    let _initializerTokenAccount = await getAccount(provider.connection, initializerTokenAccount);
    assert.equal(Number(_initializerTokenAccount.amount), 1000);

  });

  it("Transfer", async () => {
    let _takerTokenAccountA = await getAccount(provider.connection, takerTokenAccountA);
    assert.equal(Number(_takerTokenAccountA.amount), 0);

    await program.methods.initialize(new anchor.BN(42), mint)
        .accounts({
            initializer: initializerMainAccount.publicKey,
            from: initializerTokenAccount,
            to: takerTokenAccountA,
            to2: takerMainAccountB.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID
        })
        .signers([initializerMainAccount])
        .rpc();

    _takerTokenAccountA = await getAccount(provider.connection, takerTokenAccountA);
    assert.equal(Number(_takerTokenAccountA.amount), 42);
  });
});
