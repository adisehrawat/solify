
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Counter } from "../target/types/counter";
import { assert } from "chai";
import { Keypair, SystemProgram, PublicKey, LAMPORTS_PER_SOL, Keypair } from "@solana/web3.js";

describe("counter_program", () => {
    // Configure the client
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.counter_program as Program<Counter>;

    // Setup Requirements
    // keypair decelarations

    // PDA Decelaration
    let pda1: PublicKey;
    let bump1: number;

    before(async () => {
        // ----- Airdrop for each user Keypair -----

        // ----- PDA Initialization -----
        [pda1, bump1] = PublicKey.findProgramAddressSync(
            [/* TODO seeds for pda1 */],
            program.programId
        );
    });


})

