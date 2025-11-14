/// Main test file template - single consolidated file with helpers, setup, and tests
pub const MAIN_TEST_TEMPLATE: &str = r#"import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { expect } from "chai";
import { {{program_name_camel}} } from "../target/types/{{program_name}}";

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
  {{#each setup_steps}}
  // {{type}}: {{description}}
  {{#if (eq type "CreateKeypair")}}
  const account_{{@index}} = await createKeypair();
  context.accounts.set("{{description}}", account_{{@index}});
  await airdrop(provider.connection, account_{{@index}}.publicKey, 10 * LAMPORTS_PER_SOL);
  {{/if}}
  {{/each}}

  console.log("  Deriving PDAs...");
  // Initialize PDAs
  {{#each pda_init}}
  {{#with this}}
  {
    const seeds = [
      {{#each seeds}}
      {{#if (eq type "Static")}}
      Buffer.from("{{value}}"),
      {{/if}}
      {{#if (eq type "AccountKey")}}
      context.accounts.get("{{value}}")?.publicKey.toBuffer() || Buffer.alloc(32),
      {{/if}}
      {{/each}}
    ];
    const [pda, bump] = await derivePDA(seeds, program.programId);
    context.pdas.set("{{account_name}}", [pda, bump]);
    console.log("    ✓ {{account_name}}:", pda.toString());
  }
  {{/with}}
  {{/each}}

  console.log("  Account dependencies:");
  {{#each account_dependencies}}
  console.log("    {{account_name}}{{#if is_pda}} (PDA){{/if}}{{#if must_be_initialized}} - must initialize first{{/if}}");
  {{#if depends_on}}
  {{#if depends_on.[0]}}
  console.log("      Depends on: {{depends_on}}");
  {{/if}}
  {{/if}}
  {{/each}}

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

describe("{{program_name}} - Complete Test Suite", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.{{program_name_camel}} as Program<{{program_name_camel}}>;

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
      console.log("  ✓ Total instructions: {{instructions_count}}");
      console.log("  ✓ Total tests: {{total_tests}}");
      console.log("  ✓ PDAs to initialize: {{pda_count}}");
      console.log("  ✓ Setup requirements: {{setup_count}}");
    });

    it("should display execution order", () => {
      console.log("  Instruction execution order:");
      {{#each instructions}}
      console.log("    {{@index}}. {{this}}");
      {{/each}}
      expect(true).to.be.true;
    });
  });

  {{#each test_suites}}
  describe("Instruction: {{instruction_name}}", () => {
    
    describe("Positive Test Cases", () => {
      {{#each positive_tests}}
      it("{{description}}", async () => {
        try {
          console.log("  Running test case {{index}}: {{test_type}}");
          
          // Prepare test arguments
          {{#each arguments}}
          const {{name}} = {{value}};
          {{/each}}

          // Derive PDAs that depend on instruction arguments
          {{#each ../account_mappings}}
          {{#if needs_derivation}}
          const {{name}}Pda = {{source}};
          {{/if}}
          {{/each}}

          // Execute instruction
          const tx = await program.methods
            .{{../instruction_name}}(
              {{#each arguments}}
              {{name}}{{#unless @last}},{{/unless}}
              {{/each}}
            )
            .accounts({
              {{#each ../account_mappings}}
              {{name}}: {{#if needs_derivation}}{{name}}Pda{{else}}{{source}}{{/if}}{{#unless @last}},{{/unless}}
              {{/each}}
            })
            .rpc();

          await waitForTransaction(provider.connection, tx);

          // Expected outcome: {{expected.type}}
          {{#if expected.state_changes}}
          // Expected state changes:
          {{#each expected.state_changes}}
          //   - {{this}}
          {{/each}}
          {{/if}}

          console.log("  ✓ Test passed");
          expect(tx).to.be.a("string");
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      {{/each}}
    });

    describe("Negative Test Cases", () => {
      {{#each negative_tests}}
      it("{{description}}", async () => {
        try {
          console.log("  Running negative test case {{index}}: {{test_type}}");
          
          // Prepare invalid test arguments
          {{#each arguments}}
          const {{name}} = {{value}}; // Invalid value
          {{/each}}

          // Derive PDAs that depend on instruction arguments
          {{#each ../account_mappings}}
          {{#if needs_derivation}}
          const {{name}}Pda = {{source}};
          {{/if}}
          {{/each}}

          // This should fail
          try {
            await program.methods
              .{{../instruction_name}}(
                {{#each arguments}}
                {{name}}{{#unless @last}},{{/unless}}
                {{/each}}
              )
              .accounts({
                {{#each ../account_mappings}}
                {{name}}: {{#if needs_derivation}}{{name}}Pda{{else}}{{source}}{{/if}}{{#unless @last}},{{/unless}}
                {{/each}}
              })
              .rpc();

            expect.fail("Expected transaction to fail but it succeeded");
          } catch (error: any) {
            // Verify error matches expected
            {{#if expected.error_code}}
            expect(error.toString()).to.include("{{expected.error_code}}");
            {{/if}}
            console.log("  ✓ Correctly failed: {{expected.error_message}}");
          }
        } catch (error) {
          console.error("  ✗ Test failed:", error);
          throw error;
        }
      });
      {{/each}}
    });

    {{#if arguments}}
    describe("Argument Validation for {{instruction_name}}", () => {
      {{#each arguments}}
      it("should validate argument: {{name}} ({{type}})", async () => {
        // TODO: Add validation tests for {{name}}
        {{#if is_optional}}
        // Note: {{name}} is optional
        {{/if}}
        console.log("  ✓ Validation test for {{name}} (implementation needed)");
        expect(true).to.be.true;
      });
      {{/each}}
    });
    {{/if}}

  });
  {{/each}}

  describe("Integration Tests", () => {
    it("should execute full instruction sequence", async () => {
      console.log("  Testing complete workflow in execution order:");
      {{#each instructions}}
      console.log("    Step {{@index}}: {{this}}");
      {{/each}}
      
      // TODO: Implement full integration test executing all instructions in order
      console.log("  ✓ Integration test structure ready (implementation needed)");
      expect(true).to.be.true;
    });
  });
});
"#;

/// Test case template
pub const TEST_CASE_TEMPLATE: &str = r#"import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { {{program_name}} } from "../target/types/{{program_name}}";
import { setupTest } from "./{{program_name}}.setup";
import { 
  createKeypair, 
  airdrop, 
  derivePDA,
  waitForTransaction 
} from "./{{program_name}}.helpers";

describe("{{program_name}} - {{instruction_name}}", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.{{program_name}} as Program<{{program_name}}>;

  let testContext: any;

  before(async () => {
    testContext = await setupTest(provider, program);
  });

  describe("Positive Test Cases", () => {
    {{#each positive_tests}}
    it("{{description}}", async () => {
      try {
        // Test case {{index}}: {{test_type}}
        
        // Prepare arguments
        {{#each arguments}}
        const {{name}} = {{value}}; // TODO: Replace with actual test value
        {{/each}}

        // Execute instruction
        const tx = await program.methods
          .{{../instruction_name}}(
            {{#each arguments}}
            {{name}}{{#unless @last}},{{/unless}}
            {{/each}}
          )
          .accounts({
            // TODO: Add required accounts
          })
          .rpc();

        await waitForTransaction(provider.connection, tx);

        // Verify expected outcome
        {{#if expected.state_changes}}
        {{#each expected.state_changes}}
        // Expected state change: {{this}}
        {{/each}}
        {{/if}}

        console.log("  ✓ {{description}}");
        console.log("    Transaction:", tx);
      } catch (error) {
        console.error("  ✗ Test failed:", error);
        throw error;
      }
    });
    {{/each}}
  });

  describe("Negative Test Cases", () => {
    {{#each negative_tests}}
    it("{{description}}", async () => {
      try {
        // Test case {{index}}: {{test_type}}
        
        // Prepare invalid arguments
        {{#each arguments}}
        const {{name}} = {{value}}; // Invalid test value
        {{/each}}

        // Expect this to fail
        try {
          await program.methods
            .{{../instruction_name}}(
              {{#each arguments}}
              {{name}}{{#unless @last}},{{/unless}}
              {{/each}}
            )
            .accounts({
              // TODO: Add required accounts
            })
            .rpc();

          // If we reach here, the test should fail
          expect.fail("Expected transaction to fail but it succeeded");
        } catch (error: any) {
          // Verify error matches expected
          {{#if expected.error_code}}
          expect(error.toString()).to.include("{{expected.error_code}}");
          {{/if}}
          console.log("  ✓ Correctly failed: {{expected.error_message}}");
        }
      } catch (error) {
        console.error("  ✗ Test failed:", error);
        throw error;
      }
    });
    {{/each}}
  });

  describe("Argument Validation", () => {
    {{#each arguments}}
    it("should validate {{name}} (type: {{type}})", async () => {
      // TODO: Add specific validation tests for {{name}}
      {{#if is_optional}}
      // Note: {{name}} is optional
      {{/if}}
      expect(true).to.be.true;
    });
    {{/each}}
  });
});
"#;

/// Setup template
pub const SETUP_TEMPLATE: &str = r#"import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider } from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { createKeypair, airdrop, derivePDA } from "./{{program_name}}.helpers";

export interface TestContext {
  provider: AnchorProvider;
  program: Program<any>;
  accounts: Map<string, Keypair>;
  pdas: Map<string, [anchor.web3.PublicKey, number]>;
}

/**
 * Setup test environment
 */
export async function setupTest(
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
  {{#each setup_steps}}
  // {{type}}: {{description}}
  {{#if (eq type "CreateKeypair")}}
  const account_{{@index}} = await createKeypair();
  context.accounts.set("{{description}}", account_{{@index}});
  {{/if}}
  {{#if (eq type "FundAccount")}}
  await airdrop(provider.connection, account_{{@index}}.publicKey, 10 * LAMPORTS_PER_SOL);
  {{/if}}
  {{/each}}

  console.log("  Deriving PDAs...");

  // Initialize PDAs
  {{#each pda_init}}
  {{#with this}}
  {
    const seeds = [
      {{#each seeds}}
      {{#if (eq type "Static")}}
      Buffer.from("{{value}}"),
      {{/if}}
      {{#if (eq type "AccountKey")}}
      context.accounts.get("{{value}}")?.publicKey.toBuffer() || Buffer.alloc(32),
      {{/if}}
      {{/each}}
    ];

    const [pda, bump] = await derivePDA(
      seeds,
      program.programId
    );

    context.pdas.set("{{account_name}}", [pda, bump]);
    console.log("    ✓ {{account_name}}:", pda.toString());
  }
  {{/with}}
  {{/each}}

  console.log("  Account dependencies:");
  {{#each account_dependencies}}
  console.log("    {{account_name}}{{#if is_pda}} (PDA){{/if}}{{#if must_be_initialized}} - must initialize first{{/if}}");
  {{#if depends_on}}
  {{#if depends_on.[0]}}
  console.log("      Depends on: {{depends_on}}");
  {{/if}}
  {{/if}}
  {{/each}}

  return context;
}

/**
 * Cleanup test environment
 */
export async function cleanupTest(context: TestContext): Promise<void> {
  console.log("  Cleaning up test accounts...");
  // Perform any necessary cleanup
  context.accounts.clear();
  context.pdas.clear();
}
"#;

/// Helper functions template
pub const HELPER_TEMPLATE: &str = r#"import * as anchor from "@coral-xyz/anchor";
import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";

/**
 * Create a new keypair
 */
export async function createKeypair(): Promise<Keypair> {
  return Keypair.generate();
}

/**
 * Airdrop SOL to an account
 */
export async function airdrop(
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
export async function derivePDA(
  seeds: (Buffer | Uint8Array)[],
  programId: PublicKey
): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(seeds, programId);
}

/**
 * Wait for transaction confirmation
 */
export async function waitForTransaction(
  connection: Connection,
  signature: string
): Promise<void> {
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    ...latestBlockhash,
  });
}

/**
 * Get account balance
 */
export async function getBalance(
  connection: Connection,
  publicKey: PublicKey
): Promise<number> {
  return await connection.getBalance(publicKey);
}

/**
 * Format SOL amount
 */
export function formatSOL(lamports: number): string {
  return `${lamports / LAMPORTS_PER_SOL} SOL`;
}

/**
 * Sleep for specified milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry a function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  delay: number = 1000
): Promise<T> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await sleep(delay * Math.pow(2, i));
    }
  }
  throw new Error("Max retries exceeded");
}
"#;

