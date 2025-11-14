
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CounterProgram } from "../target/types/counter_program";
import { assert } from "chai";
import { Keypair, SystemProgram, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("counter_program", () => {
    // Configure the client
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const connection = provider.connection;

    const program = anchor.workspace.counter_program as Program<CounterProgram>;

    // Setup Requirements
    // keypair decelarations
    const authority = Keypair.generate();
    const authorityPubkey = authority.publicKey;

    // PDA Decelaration
    let pda3: PublicKey;
    let bump3: number;

    before(async () => {
        // ----- Airdrop for each user Keypair -----
        const sig1 = await connection.requestAirdrop(authorityPubkey, 10 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig1, "confirmed");

        // ----- PDA Initialization -----
        [pda3, bump3] = PublicKey.findProgramAddressSync(
            [Buffer.from("counter"), authorityPubkey.toBuffer()],
            program.programId
        );

    });

    


    
    it("initialize - valid inputs", async () => {
        // Prepare arguments
        // Execute instruction
        try {
            await program.methods
                .initialize(
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey,
                    systemProgram: SystemProgram.programId
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    


    
    it("increment - valid inputs", async () => {
        // Prepare arguments
        // Execute instruction
        try {
            await program.methods
                .increment(
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    


    
    it("decrement - valid inputs", async () => {
        // Prepare arguments
        // Execute instruction
        try {
            await program.methods
                .decrement(
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    


    
    it("set - valid inputs", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("1000");
        // Execute instruction
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    it("value - minimum value", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("0");
        // Execute instruction
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
            // Expect success
            assert.ok(true);
        } catch (err) {
            assert.fail("Instruction should not have failed: " + err);
        }
    });
    
    it("set - value below minimum", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("-1");
        // Execute instruction expecting failure
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
        } catch (err) {
            assert(err.message.includes("value must be at least 0"));
        }
    });
    it("set - value is zero", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("0");
        // Execute instruction expecting failure
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
        } catch (err) {
            assert(err.message.includes("value cannot be zero"));
        }
    });
    it("set - value overflow", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("18446744073709551615");
        // Execute instruction expecting failure
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
        } catch (err) {
            assert(err.message.includes("Arithmetic overflow"));
        }
    });
    it("set - value negative value", async () => {
        // Prepare arguments
        const valueValue = new anchor.BN("-1");
        // Execute instruction expecting failure
        try {
            await program.methods
                .set(
                    valueValue
                )
                .accountsStrict({
                    counter: pda3,
                    authority: authorityPubkey
                })
                .signers([
                    authority
                ])
                .rpc();
        } catch (err) {
            assert(err.message.includes("Unsigned integer cannot be negative"));
        }
    });

})

