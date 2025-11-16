import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Solify } from "../target/types/solify";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import { assert } from "chai";

type IdlSeed = {
  kind: string;
  path: string;
  value: string;
};

type IdlPda = {
  seeds: IdlSeed[];
  program: string;
};

type IdlAccountItem = {
  name: string;
  isMut: boolean;
  isSigner: boolean;
  isOptional: boolean;
  docs: string[];
  pda: IdlPda | null;
};

type IdlField = {
  name: string;
  fieldType: string;
};

type IdlInstruction = {
  name: string;
  accounts: IdlAccountItem[];
  args: IdlField[];
  docs: string[];
};

type IdlAccount = {
  name: string;
  fields: IdlField[];
};

type IdlTypeDef = {
  name: string;
  kind: string;
  fields: string[];
};

type IdlError = {
  code: number;
  name: string;
  msg: string;
};

type IdlConstant = {
  name: string;
  constantType: string;
  value: string;
};

type IdlEvent = {
  name: string;
  discriminator: Buffer;
  fields: IdlField[];
};

type IdlData = {
  name: string;
  version: string;
  instructions: IdlInstruction[];
  accounts: IdlAccount[];
  types: IdlTypeDef[];
  errors: IdlError[];
  constants: IdlConstant[];
  events: IdlEvent[];
};

describe("solify", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.solify as Program<Solify>;
  const connection = provider.connection;

  const user1 = Keypair.generate();
  const userPubkey = user1.publicKey;

  let journal_testMetadataPda: PublicKey;
  let journal_idlStoragePda: PublicKey;
  let counter_testMetadataPda: PublicKey;
  let counter_idlStoragePda: PublicKey;
  let message_testMetadataPda: PublicKey;
  let message_idlStoragePda: PublicKey;
  let vault_testMetadataPda: PublicKey;
  let vault_idlStoragePda: PublicKey;


  console.log("userPubkey", userPubkey.toBase58());
  before(async () => {

    await new Promise((resolve) => setTimeout(resolve, 20000)); // wait for 20 seconds to airdrop SOL to user1
    console.log("user1 balance", await connection.getBalance(userPubkey)/ LAMPORTS_PER_SOL);
  });


  describe("Journal IDL testing", async () => {
    const journal_programId = new PublicKey(
        "4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb"
      );
      const journal_program_name = "journal";
      const journal_executionOrder = [
        "create_journal_entry",
        "update_journal_entry",
        "delete_journal_entry",
      ];
      const journal_idlData = {
        name: "journal",
        version: "0.1.0",
        instructions: [
          {
            name: "create_journal_entry",
            accounts: [
              {
                name: "journal_entry",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: {
                  seeds: [
                    {
                      kind: "arg",
                      path: "title",
                      value: "",
                    },
                    {
                      kind: "account",
                      path: "owner",
                      value: "",
                    },
                  ],
                  program: "",
                },
              },
              {
                name: "owner",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null,
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null,
              },
            ],
            args: [
              {
                name: "title",
                fieldType: "string",
              },
              {
                name: "message",
                fieldType: "string",
              },
            ],
            docs: [],
          },
          {
            name: "delete_journal_entry",
            accounts: [
              {
                name: "journal_entry",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: {
                  seeds: [
                    {
                      kind: "arg",
                      path: "title",
                      value: "",
                    },
                    {
                      kind: "account",
                      path: "owner",
                      value: "",
                    },
                  ],
                  program: "",
                },
              },
              {
                name: "owner",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null,
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null,
              },
            ],
            args: [
              {
                name: "title",
                fieldType: "string",
              },
            ],
            docs: [],
          },
          {
            name: "update_journal_entry",
            accounts: [
              {
                name: "journal_entry",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: {
                  seeds: [
                    {
                      kind: "arg",
                      path: "title",
                      value: "",
                    },
                    {
                      kind: "account",
                      path: "owner",
                      value: "",
                    },
                  ],
                  program: "",
                },
              },
              {
                name: "owner",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null,
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null,
              },
            ],
            args: [
              {
                name: "title",
                fieldType: "string",
              },
              {
                name: "message",
                fieldType: "string",
              },
            ],
            docs: [],
          },
        ],
        accounts: [
          {
            name: "JournalEntryState",
            fields: [],
          },
        ],
        types: [
          {
            name: "JournalEntryState",
            kind: "struct",
            fields: ["owner", "title", "message"],
          },
        ],
        errors: [],
        constants: [],
        events: [],
      };
      const journal_pharaphase = "journal1";

      before(async () => {
        [journal_idlStoragePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("idl_storage"), journal_programId.toBuffer(), userPubkey.toBuffer()],
            program.programId
          );
          [journal_testMetadataPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("tests_metadata"),
              journal_programId.toBuffer(),
              userPubkey.toBuffer(),
              Buffer.from(journal_pharaphase),
            ],
            program.programId
          );
      })

      it("should store journal IdlData on-chain", async () => {
        const tx = await program.methods.storeIdlData(journal_idlData,journal_programId)
            .accountsStrict({
                idlStorage: journal_idlStoragePda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("journal IDL storage PDA: ", journal_idlStoragePda.toBase58());
      })
      it("should generate journal metadata", async () => {
        const tx = await program.methods.generateMetadata(journal_executionOrder, journal_programId, journal_program_name, journal_pharaphase)
            .accountsStrict({
                idlStorage: journal_idlStoragePda,
                testMetadataConfig: journal_testMetadataPda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();


      
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("journal testMetadata PDA: ", journal_testMetadataPda.toBase58());
      })

  })

  describe("Counter IDL testing", async () => {
    const counter_programId = new PublicKey(
        "69ctt84hJMj9b8bChtsgUVufU11UEf2kqofsJUb39iaa"
      );
      const counter_program_name = "counter";
      const counter_executionOrder = [
        "initialize",
        "increment",
        "decrement",
      ];
      const counter_idlData = {
        name: "counter",
        version: "0.1.0",
        instructions: [
          {
            name: "decrement",
            accounts: [
              {
                name: "counter",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null,
              }
            ],
            args: [],
            docs: [],
          },
          {
            name: "increment",
            accounts: [
              {
                name: "counter",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null,
              }
            ],
            args: [],
            docs: [],
          },
          {
            name: "initialize",
            accounts: [
              {
                name: "counter",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "user",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [],
            docs: []
          }
        ],
        accounts: [
          {
            name: "Counter",
            fields: []
          }
        ],
        types: [
          {
            name: "Counter",
            kind: "struct",
            fields: ["count"]
          }
        ],
        errors: [],
        constants: [],
        events: []
      };
      
    
      const counter_pharaphase = "counter1";

      before(async () => {
        [counter_idlStoragePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("idl_storage"), counter_programId.toBuffer(), userPubkey.toBuffer()],
            program.programId
          );
          [counter_testMetadataPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("tests_metadata"),
              counter_programId.toBuffer(),
              userPubkey.toBuffer(),
              Buffer.from(counter_pharaphase),
            ],
            program.programId
          );
      })

      it("should store counter IdlData on-chain", async () => {
        const tx = await program.methods.storeIdlData(counter_idlData,counter_programId)
            .accountsStrict({
                idlStorage: counter_idlStoragePda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("counter IDL storage PDA: ", counter_idlStoragePda.toBase58());
      })
      it("should generate counter metadata", async () => {
        const tx = await program.methods.generateMetadata(counter_executionOrder, counter_programId, counter_program_name, counter_pharaphase)
            .accountsStrict({
                idlStorage: counter_idlStoragePda,
                testMetadataConfig: counter_testMetadataPda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();

      
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("counter testMetadata PDA: ", counter_testMetadataPda.toBase58());
      })

  })

  describe("Message IDL testing", async () => {
    const message_programId = new PublicKey(
        "Dpd54vhwUn6eq5RM6J1SgtrowTJTCJdxApu4nRtfh7av"
      );
      const message_program_name = "message";
      const message_executionOrder = [
        "initialize",
        "update_message",
      ];
      const message_idlData = {
        name: "message",
        version: "0.1.0",
        instructions: [
          {
            name: "initialize",
            accounts: [
              {
                name: "board",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "user",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [
              {
                name: "message",
                fieldType: "string"
              }
            ],
            docs: []
          },
          {
            name: "update_message",
            accounts: [
              {
                name: "board",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [
              {
                name: "new_message",
                fieldType: "string"
              }
            ],
            docs: []
          }
        ],
        accounts: [
          {
            name: "Board",
            fields: []
          }
        ],
        types: [
          {
            name: "Board",
            kind: "struct",
            fields: ["message"]
          }
        ],
        errors: [],
        constants: [],
        events: []
      };
      
      
    
      const message_pharaphase = "message1";

      before(async () => {
        [message_idlStoragePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("idl_storage"), message_programId.toBuffer(), userPubkey.toBuffer()],
            program.programId
          );
          [message_testMetadataPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("tests_metadata"),
              message_programId.toBuffer(),
              userPubkey.toBuffer(),
              Buffer.from(message_pharaphase),
            ],
            program.programId
          );
      })

      it("should store message IdlData on-chain", async () => {
        const tx = await program.methods.storeIdlData(message_idlData,message_programId)
            .accountsStrict({
                idlStorage: message_idlStoragePda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("message IDL storage PDA: ", message_idlStoragePda.toBase58());
      })
      it("should generate message metadata", async () => {
        const tx = await program.methods.generateMetadata(message_executionOrder, message_programId, message_program_name, message_pharaphase)
            .accountsStrict({
                idlStorage: message_idlStoragePda,
                testMetadataConfig: message_testMetadataPda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();

      
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("message testMetadata PDA: ", message_testMetadataPda.toBase58());
      })

  })

  describe("Vault IDL testing", async () => {
    const vault_programId = new PublicKey(
        "B3mqUcr9rES8fYhtVd7ezYSotpTdM4Uzkv6Fp8nwCDV1"
      );
      const vault_program_name = "vault";
      const vault_executionOrder = [
        "initialize",
        "deposit",
        "withdraw",
      ];
      const vault_idlData = {
        name: "vault",
        version: "0.1.0",
        instructions: [
          {
            name: "deposit",
            accounts: [
              {
                name: "vault",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "user",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [
              {
                name: "amount",
                fieldType: "u64"
              }
            ],
            docs: []
          },
          {
            name: "initialize",
            accounts: [
              {
                name: "vault",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "user",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "system_program",
                isMut: false,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [],
            docs: []
          },
          {
            name: "withdraw",
            accounts: [
              {
                name: "vault",
                isMut: true,
                isSigner: false,
                isOptional: false,
                docs: [],
                pda: null
              },
              {
                name: "user",
                isMut: true,
                isSigner: true,
                isOptional: false,
                docs: [],
                pda: null
              }
            ],
            args: [
              {
                name: "amount",
                fieldType: "u64"
              }
            ],
            docs: []
          }
        ],
        accounts: [
          {
            name: "Vault",
            fields: []
          }
        ],
        types: [
          {
            name: "Vault",
            kind: "struct",
            fields: []
          }
        ],
        errors: [],
        constants: [],
        events: []
      };
      
      
      
    
      const vault_pharaphase = "vault1";

      before(async () => {
        [vault_idlStoragePda] = PublicKey.findProgramAddressSync(
            [Buffer.from("idl_storage"), vault_programId.toBuffer(), userPubkey.toBuffer()],
            program.programId
          );
          [vault_testMetadataPda] = PublicKey.findProgramAddressSync(
            [
              Buffer.from("tests_metadata"),
              vault_programId.toBuffer(),
              userPubkey.toBuffer(),
              Buffer.from(vault_pharaphase),
            ],
            program.programId
          );
      })

      it("should store vault IdlData on-chain", async () => {
        const tx = await program.methods.storeIdlData(vault_idlData,vault_programId)
            .accountsStrict({
                idlStorage: vault_idlStoragePda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("vault IDL storage PDA: ", vault_idlStoragePda.toBase58());
      })
      it("should generate vault metadata", async () => {
        const tx = await program.methods.generateMetadata(vault_executionOrder, vault_programId, vault_program_name, vault_pharaphase)
            .accountsStrict({
                idlStorage: vault_idlStoragePda,
                testMetadataConfig: vault_testMetadataPda,
                authority: userPubkey,
                systemProgram: SystemProgram.programId,
            })
            .signers([user1])
            .rpc();

      
            console.log(`tx is: https://explorer.solana.com/tx/${tx.toString()}?cluster=devnet`);
            console.log("vault testMetadata PDA: ", vault_testMetadataPda.toBase58());
      })

  })
});



  