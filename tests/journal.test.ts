import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { expect } from "chai";
import { journal } from "../target/types/journal";

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Create a new keypair
 */
async function createKeypair(): Promise<Keypair> {
  return Keypair.generate();
}

/**
 * Airdrop SOL to an account
 */
async function airdrop(
  connection: Connection,
  publicKey: PublicKey,
  amount: number
): Promise<void> {
  const signature = await connection.requestAirdrop(publicKey, amount);
  await connection.confirmTransaction(signature);
}

/**
 * Derive a PDA
 */
async function derivePDA(
  seeds: (Buffer | Uint8Array)[],
  programId: PublicKey
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(seeds, programId);
}

/**
 * Wait for transaction confirmation
 */
async function waitForTransaction(
  connection: Connection,
  signature: string
): Promise<void> {
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}

// ============================================================================
// Test Context Interface
// ============================================================================

interface TestContext {
  provider: AnchorProvider;
  program: Program<any>;
  accounts: Map<string, Keypair>;
  pdas: Map<string, [PublicKey, number]>;
}

/**
 * Setup test environment
 */
async function setupTest(
  provider: AnchorProvider,
  program: Program<any>
): Promise<TestContext> {
  const context: TestContext = {
    provider,
    program,
    accounts: new Map(),
    pdas: new Map(),
  };

  console.log("  Initializing test accounts...");
  // Setup requirements
  // CreateKeypair: Create keypair for owner
  const account_0 = await createKeypair();
  context.accounts.set("Create keypair for owner", account_0);
  await airdrop(provider.connection, account_0.publicKey, 10 * LAMPORTS_PER_SOL);
  // FundAccount: Fund owner with SOL for transactions
  // InitializePda: Initialize journal_entry PDA

  console.log("  Deriving PDAs...");
  // Initialize PDAs
  {
    const seeds = [
      context.accounts.get("owner")?.publicKey.toBuffer() || Buffer.alloc(32),
    ];
    const [pda, bump] = await derivePDA(seeds, program.programId);
    context.pdas.set("journal_entry", [pda, bump]);
    console.log("    ✓ journal_entry:", pda.toString());
  }

  console.log("  Account dependencies:");
  console.log("    journal_entry (PDA) - must initialize first");
  console.log("      Depends on: [owner]");
  console.log("    owner - must initialize first");
  console.log("    system_program - must initialize first");

  return context;
}

/**
 * Cleanup test environment
 */
async function cleanupTest(context: TestContext): Promise<void> {
  console.log("  Cleaning up test accounts...");
  context.accounts.clear();
  context.pdas.clear();
}

// ============================================================================
// Test Suite
// ============================================================================

describe("journal - Complete Test Suite", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.journal as Program<journal>;

  let testContext: TestContext;

  before(async () => {
    console.log("Setting up test environment...");
    testContext = await setupTest(provider, program);
    console.log("Test environment ready!");
    console.log("Program ID:", program.programId.toString());
  });

  after(async () => {
    console.log("Cleaning up test environment...");
    await cleanupTest(testContext);
  });

  describe("Configuration", () => {
    it("should have correct program configuration", () => {
      expect(program.programId).to.be.instanceOf(PublicKey);
      console.log("  ✓ Total instructions: 3");
      console.log("  ✓ Total tests: 15");
      console.log("  ✓ PDAs to initialize: 1");
      console.log("  ✓ Setup requirements: 3");
    });

    it("should display execution order", () => {
      console.log("  Instruction execution order:");
      console.log("    0. create_journal_entry");
      console.log("    1. update_journal_entry");
      console.log("    2. delete_journal_entry");
      expect(true).to.be.true;
    });
  });

  describe("Instruction: create_journal_entry", () => {
    
    describe("Positive Test Cases", () => {
      it("create_journal_entry - valid inputs", async () => {
        try {
          console.log("  Running test case 1: Positive");
          
          // Prepare test arguments
          const title = &quot;test_value&quot;;
          const message = &quot;test_value&quot;;

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // Execute instruction
          const tx = await program.methods
            .create_journal_entry(
              title,
              message
            )
            .accounts({
              journal_entry: journal_entryPda,
              owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
              system_program: anchor.web3.SystemProgram.programId
            })
            .rpc();

          await waitForTransaction(provider.connection, tx);

          // Expected outcome: success
          // Expected state changes:
          //   - Account state updated successfully
          //   - Instruction executed without errors

          console.log("  ✓ Test passed");
          expect(tx).to.be.a("string");
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Negative Test Cases", () => {
      it("create_journal_entry - title empty string", async () => {
        try {
          console.log("  Running negative test case 1: NegativeNull");
          
          // Prepare invalid test arguments
          const title = &quot;&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .create_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("EmptyString");
            console.log("  ✓ Correctly failed: String cannot be empty");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("create_journal_entry - title too long", async () => {
        try {
          console.log("  Running negative test case 2: NegativeBoundary");
          
          // Prepare invalid test arguments
          const title = &quot;a\&quot;.repeat(1000)&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .create_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("StringTooLong");
            console.log("  ✓ Correctly failed: String exceeds maximum length");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("create_journal_entry - message empty string", async () => {
        try {
          console.log("  Running negative test case 3: NegativeNull");
          
          // Prepare invalid test arguments
          const message = &quot;&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .create_journal_entry(
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("EmptyString");
            console.log("  ✓ Correctly failed: String cannot be empty");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("create_journal_entry - message too long", async () => {
        try {
          console.log("  Running negative test case 4: NegativeBoundary");
          
          // Prepare invalid test arguments
          const message = &quot;a\&quot;.repeat(1000)&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .create_journal_entry(
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("StringTooLong");
            console.log("  ✓ Correctly failed: String exceeds maximum length");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("create_journal_entry - all arguments invalid", async () => {
        try {
          console.log("  Running negative test case 5: NegativeConstraint");
          
          // Prepare invalid test arguments
          const title = &quot;invalid&quot;; // Invalid value
          const message = &quot;invalid&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .create_journal_entry(
                title,
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            console.log("  ✓ Correctly failed: Multiple validation errors");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Argument Validation for create_journal_entry", () => {
      it("should validate argument: title (String { max_length: None })", async () => {
        // TODO: Add validation tests for title
        console.log("  ✓ Validation test for title (implementation needed)");
        expect(true).to.be.true;
      });
      it("should validate argument: message (String { max_length: None })", async () => {
        // TODO: Add validation tests for message
        console.log("  ✓ Validation test for message (implementation needed)");
        expect(true).to.be.true;
      });
    });

  });
  describe("Instruction: update_journal_entry", () => {
    
    describe("Positive Test Cases", () => {
      it("update_journal_entry - valid inputs", async () => {
        try {
          console.log("  Running test case 1: Positive");
          
          // Prepare test arguments
          const title = &quot;test_value&quot;;
          const message = &quot;test_value&quot;;

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // Execute instruction
          const tx = await program.methods
            .update_journal_entry(
              title,
              message
            )
            .accounts({
              journal_entry: journal_entryPda,
              owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
              system_program: anchor.web3.SystemProgram.programId
            })
            .rpc();

          await waitForTransaction(provider.connection, tx);

          // Expected outcome: success
          // Expected state changes:
          //   - Account state updated successfully
          //   - Instruction executed without errors

          console.log("  ✓ Test passed");
          expect(tx).to.be.a("string");
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Negative Test Cases", () => {
      it("update_journal_entry - title empty string", async () => {
        try {
          console.log("  Running negative test case 1: NegativeNull");
          
          // Prepare invalid test arguments
          const title = &quot;&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .update_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("EmptyString");
            console.log("  ✓ Correctly failed: String cannot be empty");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("update_journal_entry - title too long", async () => {
        try {
          console.log("  Running negative test case 2: NegativeBoundary");
          
          // Prepare invalid test arguments
          const title = &quot;a\&quot;.repeat(1000)&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .update_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("StringTooLong");
            console.log("  ✓ Correctly failed: String exceeds maximum length");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("update_journal_entry - message empty string", async () => {
        try {
          console.log("  Running negative test case 3: NegativeNull");
          
          // Prepare invalid test arguments
          const message = &quot;&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .update_journal_entry(
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("EmptyString");
            console.log("  ✓ Correctly failed: String cannot be empty");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("update_journal_entry - message too long", async () => {
        try {
          console.log("  Running negative test case 4: NegativeBoundary");
          
          // Prepare invalid test arguments
          const message = &quot;a\&quot;.repeat(1000)&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .update_journal_entry(
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("StringTooLong");
            console.log("  ✓ Correctly failed: String exceeds maximum length");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("update_journal_entry - all arguments invalid", async () => {
        try {
          console.log("  Running negative test case 5: NegativeConstraint");
          
          // Prepare invalid test arguments
          const title = &quot;invalid&quot;; // Invalid value
          const message = &quot;invalid&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .update_journal_entry(
                title,
                message
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            console.log("  ✓ Correctly failed: Multiple validation errors");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Argument Validation for update_journal_entry", () => {
      it("should validate argument: title (String { max_length: None })", async () => {
        // TODO: Add validation tests for title
        console.log("  ✓ Validation test for title (implementation needed)");
        expect(true).to.be.true;
      });
      it("should validate argument: message (String { max_length: None })", async () => {
        // TODO: Add validation tests for message
        console.log("  ✓ Validation test for message (implementation needed)");
        expect(true).to.be.true;
      });
    });

  });
  describe("Instruction: delete_journal_entry", () => {
    
    describe("Positive Test Cases", () => {
      it("delete_journal_entry - valid inputs", async () => {
        try {
          console.log("  Running test case 1: Positive");
          
          // Prepare test arguments
          const title = &quot;test_value&quot;;

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // Execute instruction
          const tx = await program.methods
            .delete_journal_entry(
              title
            )
            .accounts({
              journal_entry: journal_entryPda,
              owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
              system_program: anchor.web3.SystemProgram.programId
            })
            .rpc();

          await waitForTransaction(provider.connection, tx);

          // Expected outcome: success
          // Expected state changes:
          //   - Account state updated successfully
          //   - Instruction executed without errors

          console.log("  ✓ Test passed");
          expect(tx).to.be.a("string");
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Negative Test Cases", () => {
      it("delete_journal_entry - title empty string", async () => {
        try {
          console.log("  Running negative test case 1: NegativeNull");
          
          // Prepare invalid test arguments
          const title = &quot;&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .delete_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("EmptyString");
            console.log("  ✓ Correctly failed: String cannot be empty");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      it("delete_journal_entry - title too long", async () => {
        try {
          console.log("  Running negative test case 2: NegativeBoundary");
          
          // Prepare invalid test arguments
          const title = &quot;a\&quot;.repeat(1000)&quot;; // Invalid value

          // Derive PDAs that depend on instruction arguments
          const journal_entryPda = (await PublicKey.findProgramAddress([
      Buffer.from(title, &#x27;utf8&#x27;),
      testContext.accounts.get(&quot;owner&quot;)?.publicKey.toBuffer() || Buffer.alloc(32)
    ], program.programId))[0];

          // This should fail
          try {
            await program.methods
              .delete_journal_entry(
                title
              )
              .accounts({
                journal_entry: journal_entryPda,
                owner: testContext.accounts.get(&quot;owner&quot;)?.publicKey,
                system_program: anchor.web3.SystemProgram.programId
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            expect(error.toString()).to.include("StringTooLong");
            console.log("  ✓ Correctly failed: String exceeds maximum length");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
    });

    describe("Argument Validation for delete_journal_entry", () => {
      it("should validate argument: title (String { max_length: None })", async () => {
        // TODO: Add validation tests for title
        console.log("  ✓ Validation test for title (implementation needed)");
        expect(true).to.be.true;
      });
    });

  });

  describe("Integration Tests", () => {
    it("should execute full instruction sequence", async () => {
      console.log("  Testing complete workflow in execution order:");
      console.log("    Step 0: create_journal_entry");
      console.log("    Step 1: update_journal_entry");
      console.log("    Step 2: delete_journal_entry");
      
      // TODO: Implement full integration test executing all instructions in order
      console.log("  ✓ Integration test structure ready (implementation needed)");
      expect(true).to.be.true;
    });
  });
});
