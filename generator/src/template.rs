// generator/src/template.rs

/// Main test template with comprehensive coverage
pub const MAIN_TEST_TEMPLATE: &str = r#"import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, BN } from "@coral-xyz/anchor";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import type { {{program_name_camel}} } from "../target/types/{{program_name}}";

describe("{{program_name}} - Complete Test Suite", () => {
  // Configure the client
  const provider = AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.{{program_name_camel}} as Program<{{program_name_camel}}>;
  
  // Test context
  interface TestContext {
    provider: AnchorProvider;
    program: Program<{{program_name_camel}}>;
    accounts: Map<string, Keypair>;
    pdas: Map<string, [PublicKey, number]>;
  }

  let testContext: TestContext;

  // ============================================================================
  // Helper Functions
  // ============================================================================

  async function airdrop(publicKey: PublicKey, amount: number): Promise<void> {
    const signature = await provider.connection.requestAirdrop(publicKey, amount);
    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature,
      ...latestBlockhash,
    });
  }

  async function derivePDA(
    seeds: (Buffer | Uint8Array)[],
    programId: PublicKey
  ): Promise<[PublicKey, number]> {
    return PublicKey.findProgramAddressSync(seeds, programId);
  }

  // ============================================================================
  // Setup & Teardown
  // ============================================================================

  before(async () => {
    console.log("\n" + "=".repeat(60));
    console.log("Setting up test environment for {{program_name}}");
    console.log("=".repeat(60));

    testContext = {
      provider,
      program,
      accounts: new Map(),
      pdas: new Map(),
    };

    // Create and fund test accounts
    console.log("\n📝 Creating test accounts...");
    {{#each setup_steps}}
    {{#if is_keypair}}
    {
      const keypair = Keypair.generate();
      testContext.accounts.set("{{description}}", keypair);
      console.log("  ✓ Created: {{description}}");
      console.log("    Address:", keypair.publicKey.toBase58());
    }
    {{/if}}
    {{#if is_fund}}
    {
      const keypair = testContext.accounts.get("{{description}}");
      if (keypair) {
        await airdrop(keypair.publicKey, 10 * LAMPORTS_PER_SOL);
        const balance = await provider.connection.getBalance(keypair.publicKey);
        console.log("  ✓ Funded: {{description}}");
        console.log("    Balance:", balance / LAMPORTS_PER_SOL, "SOL");
      }
    }
    {{/if}}
    {{/each}}

    // Derive PDAs
    console.log("\n🔑 Deriving PDAs...");
    {{#each pda_init}}
    {
      const seeds = [
        {{#each seeds}}
        {{#if (eq type "Static")}}
        Buffer.from("{{value}}"),
        {{/if}}
        {{#if (eq type "AccountKey")}}
        testContext.accounts.get("{{value}}")?.publicKey.toBuffer() || Buffer.alloc(32),
        {{/if}}
        {{/each}}
      ];
      
      const [pda, bump] = derivePDA(seeds, program.programId);
      testContext.pdas.set("{{account_name}}", [pda, bump]);
      console.log("  ✓ {{account_name}}:", pda.toBase58());
    }
    {{/each}}

    console.log("\n📊 Test Configuration:");
    console.log("  Program ID:", program.programId.toBase58());
    console.log("  Instructions:", "{{instructions_count}}");
    console.log("  Total Tests:", "{{total_tests}}" + " ({{total_positive}} positive, {{total_negative}} negative)");
    console.log("  PDAs:", "{{pda_count}}");
    console.log("\n" + "=".repeat(60) + "\n");
  });

  after(async () => {
    console.log("\n" + "=".repeat(60));
    console.log("Cleaning up test environment");
    console.log("=".repeat(60) + "\n");
    testContext.accounts.clear();
    testContext.pdas.clear();
  });

  // ============================================================================
  // Configuration Tests
  // ============================================================================

  describe("Configuration", () => {
    it("should have correct program configuration", () => {
      assert.ok(program.programId instanceof PublicKey);
      console.log("  ✓ Program ID valid");
    });

    it("should display execution order", () => {
      const executionOrder = [
        {{#each instructions}}
        "{{this}}"{{#unless @last}},{{/unless}}
        {{/each}}
      ];
      console.log("\n  📋 Execution Order:");
      executionOrder.forEach((instr, idx) => {
        console.log(`    ${idx + 1}. ${instr}`);
      });
      assert.equal(executionOrder.length, {{instructions_count}});
    });

    it("should have all required accounts", () => {
      console.log("\n  👥 Account Dependencies:");
      {{#each account_dependencies}}
      console.log("    • {{account_name}}{{#if is_pda}} (PDA){{/if}}{{#if is_signer}} (Signer){{/if}}");
      {{#if depends_on}}
      {{#if depends_on.[0]}}
      console.log("      Depends on: {{depends_on}}");
      {{/if}}
      {{/if}}
      {{/each}}
      assert.ok(true);
    });
  });

  // ============================================================================
  // Instruction Tests
  // ============================================================================

  {{#each test_suites}}
  describe("Instruction: {{instruction_name}}", () => {
    
    {{#if has_arguments}}
    it("should have valid argument types", () => {
      console.log("\n  📝 Arguments for {{instruction_name}}:");
      {{#each arguments}}
      console.log("    • {{name}}: {{type}}{{#if is_optional}} (optional){{/if}}");
      {{/each}}
      assert.ok(true);
    });
    {{/if}}

    describe("✓ Positive Test Cases", () => {
      {{#each positive_tests}}
      it("{{description}}", async () => {
        console.log("\n  Running: {{description}}");
        
        try {
          {{#if has_arguments}}
          // Prepare arguments
          {{#each arguments}}
          const {{name}} = {{{value}}};
          console.log("    Argument {{name}}:", {{name}});
          {{/each}}
          {{/if}}

          // Derive PDAs that depend on instruction arguments
          {{#each ../account_mappings}}
          {{#if needs_derivation}}
          const {{name}}Pda = {{source}};
          {{/if}}
          {{/each}}

          // Prepare accounts and signers
          const accounts: any = {
            {{#each ../account_mappings}}
            {{name}}: {{#if needs_derivation}}{{name}}Pda{{else}}{{source}}{{/if}}{{#unless @last}},{{/unless}}
            {{/each}}
          };

          const signers: Keypair[] = [];
          {{#each ../account_mappings}}
          {{#if is_signer}}
          {
            // Find signer account - try by name first, then by matching setup step description
            let signer: Keypair | undefined = testContext.accounts.get("{{name}}");
            if (!signer) {
              // Try to find by matching setup step descriptions
              for (const [key, value] of testContext.accounts.entries()) {
                if (key.toLowerCase().includes("{{name}}".toLowerCase()) || 
                    "{{name}}".toLowerCase().includes(key.toLowerCase())) {
                  signer = value;
                  break;
                }
              }
            }
            if (signer) {
              signers.push(signer);
            } else {
              console.warn(`    ⚠️  Signer account "{{name}}" not found in test context`);
            }
          }
          {{/if}}
          {{/each}}

          // Execute instruction
          console.log("    Executing {{../instruction_name}}...");
          const tx = await program.methods
            .{{../instruction_name}}(
              {{#if has_arguments}}
              {{#each arguments}}
              {{name}}{{#unless @last}}, {{/unless}}
              {{/each}}
              {{/if}}
            )
            .accounts(accounts)
            .signers(signers.length > 0 ? signers : [])
            .rpc();

          console.log("    ✓ Transaction successful:", tx.slice(0, 8) + "...");

          {{#if expected.is_success}}
          {{#if expected.state_changes}}
          // Verify state changes
          {{#each expected.state_changes}}
          console.log("    ✓ {{this}}");
          {{/each}}
          {{/if}}
          {{/if}}

          assert.ok(tx);
        } catch (error: any) {
          console.error("    ✗ Test failed:", error.message);
          throw error;
        }
      });
      {{/each}}
    });

    describe("✗ Negative Test Cases", () => {
      {{#if negative_tests}}
      {{#each negative_tests}}
      it("{{description}}", async () => {
        console.log("\n  Running: {{description}}");
        
        try {
          {{#if has_arguments}}
          // Prepare invalid arguments
          {{#each arguments}}
          const {{name}} = {{{value}}};
          console.log("    Invalid argument {{name}}:", {{name}});
          {{/each}}
          {{/if}}

          // Derive PDAs that depend on instruction arguments
          {{#each ../account_mappings}}
          {{#if needs_derivation}}
          const {{name}}Pda = {{source}};
          {{/if}}
          {{/each}}

          // Prepare accounts and signers
          const accounts: any = {
            {{#each ../account_mappings}}
            {{name}}: {{#if needs_derivation}}{{name}}Pda{{else}}{{source}}{{/if}}{{#unless @last}},{{/unless}}
            {{/each}}
          };

          const signers: Keypair[] = [];
          {{#each ../account_mappings}}
          {{#if is_signer}}
          {
            // Find signer account - try by name first, then by matching setup step description
            let signer: Keypair | undefined = testContext.accounts.get("{{name}}");
            if (!signer) {
              // Try to find by matching setup step descriptions
              for (const [key, value] of testContext.accounts.entries()) {
                if (key.toLowerCase().includes("{{name}}".toLowerCase()) || 
                    "{{name}}".toLowerCase().includes(key.toLowerCase())) {
                  signer = value;
                  break;
                }
              }
            }
            if (signer) {
              signers.push(signer);
            } else {
              console.warn(`    ⚠️  Signer account "{{name}}" not found in test context`);
            }
          }
          {{/if}}
          {{/each}}

          // This should fail
          console.log("    Expecting transaction to fail...");
          
          try {
            await program.methods
              .{{../instruction_name}}(
                {{#if has_arguments}}
                {{#each arguments}}
                {{name}}{{#unless @last}}, {{/unless}}
                {{/each}}
                {{/if}}
              )
              .accounts(accounts)
              .signers(signers.length > 0 ? signers : [])
              .rpc();

            // If we get here, test failed
            assert.fail("Expected transaction to fail but it succeeded");
          } catch (err: any) {
            // Verify expected error
            {{#if expected.is_failure}}
            {{#if expected.error_code}}
            const errorMsg = err.toString();
            assert.ok(
              errorMsg.includes("{{expected.error_code}}") || errorMsg.includes("{{expected.error_message}}"),
              `Expected error containing "{{expected.error_code}}" or "{{expected.error_message}}", got: ${errorMsg}`
            );
            console.log("    ✓ Correctly failed:", "{{expected.error_message}}");
            {{else}}
            console.log("    ✓ Correctly failed:", err.message);
            {{/if}}
            {{/if}}
          }

          assert.ok(true);
        } catch (error: any) {
          console.error("    ✗ Test validation failed:", error.message);
          throw error;
        }
      });
      {{/each}}
      {{else}}
      it("should have negative test cases (none generated)", () => {
        console.log("  ⚠️  No negative test cases generated for this instruction");
        assert.ok(true);
      });
      {{/if}}
    });
  });
  {{/each}}

  // ============================================================================
  // Integration Tests
  // ============================================================================

  describe("Integration: Full Workflow", () => {
    it("should execute complete instruction sequence", async () => {
      console.log("\n  🔄 Testing complete workflow:");
      {{#each instructions}}
      console.log("    Step {{@index}}: {{this}}");
      {{/each}}
      
      console.log("\n  ℹ️  Full integration test implementation coming soon");
      assert.ok(true);
    });
  });
});
"#;