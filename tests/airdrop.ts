import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Airdrop } from "../target/types/airdrop";
import { TOKEN_PROGRAM_ID, Token, createMint, createAccount, mintTo, getAccount, getAssociatedTokenAddress, createAssociatedTokenAccount, ASSOCIATED_TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram, Transaction, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import * as assert from "assert";

describe("airdrop", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Airdrop as Program<Airdrop>;
    const provider = anchor.AnchorProvider.env();

    let mint = null;
    let initializerTokenAccount = null;

    const mintAuthority = anchor.web3.Keypair.generate();
    const initializerMainAccount = anchor.web3.Keypair.generate();

    it("Initialize", async () => {
        // TODO: Rewrite in anchor.Spl (https://project-serum.github.io/anchor/ts/)
        let token = anchor.Spl.token(provider);

        await provider.connection.confirmTransaction(
            await provider.connection.requestAirdrop(
                initializerMainAccount.publicKey,
                10000000000
            ),
            "confirmed"
        );

        mint = await createMint(
            provider.connection,
            provider.wallet.payer,
            mintAuthority.publicKey,
            null,
            0
        );

        initializerTokenAccount = await createAccount(
            provider.connection,
            provider.wallet.payer,
            mint,
            initializerMainAccount.publicKey
        );

        await mintTo(
            provider.connection,
            provider.wallet.payer,
            mint,
            initializerTokenAccount,
            mintAuthority,
            10000
        );
    })

    it("Simple transfer", async () => {
        let takerMainAccount = anchor.web3.Keypair.generate();

        let takerTokenAccount = await createAccount(
            provider.connection,
            provider.wallet.payer,
            mint,
            takerMainAccount.publicKey
        );

        await program.methods.transferSimple(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                to: takerTokenAccount,
            })
            .signers([initializerMainAccount])
            .rpc();

        let _takerTokenAccount = await getAccount(provider.connection, takerTokenAccount);
        assert.equal(Number(_takerTokenAccount.amount), 42);
    });

    it("ATA transfer (account exists)", async () => {
        let takerMainAccount = anchor.web3.Keypair.generate();

        let takerATA = await getAssociatedTokenAddress(
            mint,
            takerMainAccount.publicKey
        );

        // Anchor's TS lib does not have and IDL definition for ATA program, so
        // for now just using a function straight from the @solana/spl-token
        await createAssociatedTokenAccount(
            provider.connection,
            provider.wallet.payer,
            mint,
            takerMainAccount.publicKey
        );

        await program.methods.transferAta(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                toMain: takerMainAccount.publicKey,
                toAta: takerATA,
                mint: mint
            })
            .signers([initializerMainAccount])
            .rpc();

        let _takerATA = await getAccount(provider.connection, takerATA);
        assert.equal(Number(_takerATA.amount), 42);
    });

    it("ATA transfer (account not exists)", async () => {
        let takerMainAccount = anchor.web3.Keypair.generate();

        let takerATA = await getAssociatedTokenAddress(
            mint,
            takerMainAccount.publicKey
        );

        await program.methods.transferAta(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                toMain: takerMainAccount.publicKey,
                toAta: takerATA,
                mint: mint
            })
            .signers([initializerMainAccount])
            .rpc();

        let _takerATA = await getAccount(provider.connection, takerATA);
        assert.equal(Number(_takerATA.amount), 42);
    });

    it ("Airdrop (accounts exist)", async () => {
        let takerMainAccount1 = anchor.web3.Keypair.generate();
        let takerMainAccount2 = anchor.web3.Keypair.generate();

        let takerATA1 = await getAssociatedTokenAddress(
            mint,
            takerMainAccount1.publicKey
        );

        let takerATA2 = await getAssociatedTokenAddress(
            mint,
            takerMainAccount2.publicKey
        );

        // Anchor's TS lib does not have and IDL definition for ATA program, so
        // for now just using a function straight from the @solana/spl-token
        await createAssociatedTokenAccount(
            provider.connection,
            provider.wallet.payer,
            mint,
            takerMainAccount1.publicKey
        );

        await createAssociatedTokenAccount(
            provider.connection,
            provider.wallet.payer,
            mint,
            takerMainAccount2.publicKey
        );

        await program.methods.airdrop(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                mint: mint
            })
            .remainingAccounts([
                {
                    pubkey: takerMainAccount1.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA1,
                    isWritable: true, isSigner: false
                },
                {
                    pubkey: takerMainAccount2.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA2,
                    isWritable: true, isSigner: false
                }
            ])
            .signers([initializerMainAccount])
            .rpc();

        let _takerATA1 = await getAccount(provider.connection, takerATA1);
        assert.equal(Number(_takerATA1.amount), 42);

        let _takerATA2 = await getAccount(provider.connection, takerATA2);
        assert.equal(Number(_takerATA2.amount), 42);
    });

    it ("Airdrop (accounts not exist)", async () => {
        let takerMainAccount1 = anchor.web3.Keypair.generate();
        let takerMainAccount2 = anchor.web3.Keypair.generate();

        let takerATA1 = await getAssociatedTokenAddress(
            mint,
            takerMainAccount1.publicKey
        );

        let takerATA2 = await getAssociatedTokenAddress(
            mint,
            takerMainAccount2.publicKey
        );

        await program.methods.airdrop(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                mint: mint
            })
            .remainingAccounts([
                {
                    pubkey: takerMainAccount1.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA1,
                    isWritable: true, isSigner: false
                },
                {
                    pubkey: takerMainAccount2.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA2,
                    isWritable: true, isSigner: false
                }
            ])
            .signers([initializerMainAccount])
            .rpc();

        let _takerATA1 = await getAccount(provider.connection, takerATA1);
        assert.equal(Number(_takerATA1.amount), 42);

        let _takerATA2 = await getAccount(provider.connection, takerATA2);
        assert.equal(Number(_takerATA2.amount), 42);
    });

    it ("Airdrop (duplicates)", async () => {
        let takerMainAccount = anchor.web3.Keypair.generate();

        let takerATA = await getAssociatedTokenAddress(
            mint,
            takerMainAccount.publicKey
        );

        await program.methods.airdrop(new anchor.BN(42))
            .accounts({
                initializer: initializerMainAccount.publicKey,
                from: initializerTokenAccount,
                mint: mint
            })
            .remainingAccounts([
                {
                    pubkey: takerMainAccount.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerMainAccount.publicKey,
                    isWritable: false, isSigner: false
                },
                {
                    pubkey: takerATA,
                    isWritable: true, isSigner: false
                }
            ])
            .signers([initializerMainAccount])
            .rpc();

        // Check that value is still 42, so tokens were dropped only once
        let _takerATA = await getAccount(provider.connection, takerATA);
        assert.equal(Number(_takerATA.amount), 42);
    });
});
