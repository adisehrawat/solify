import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Solify } from "../target/types/solify";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL, ComputeBudgetProgram } from "@solana/web3.js";
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
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.solify as Program<Solify>;
  const connection = provider.connection;
  

//   const user = Keypair.generate();
const user = provider.wallet;
  const userPubkey = user.publicKey;

  let userPda: PublicKey;
  let testMetadataPda: PublicKey;
  let idlStoragePda: PublicKey;

  const idlData = {
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
        fields: [
          "owner",
          "title",
          "message",
        ],
      },
    ],
    errors: [],
    constants: [],
    events: [],
  };

  const programId = new PublicKey("4ZccwG28ne8hTmKLWDyDZmHw35su99iUxFRj5jy1p1Cb");
  const programName = "journal";
  const executionOrder = ["create_journal_entry", "update_journal_entry", "delete_journal_entry"];

  console.log("userPubkey", userPubkey.toBase58());

  before(async () => {
    // const airdropSig = await connection.requestAirdrop(userPubkey, LAMPORTS_PER_SOL * 100);
    // await connection.confirmTransaction(airdropSig);

    console.log("user balance", await connection.getBalance(userPubkey));

    [userPda] = PublicKey.findProgramAddressSync([Buffer.from("user_config"), userPubkey.toBuffer()], program.programId);
    [testMetadataPda] = PublicKey.findProgramAddressSync([Buffer.from("tests_metadata"), programId.toBuffer(), userPubkey.toBuffer()], program.programId);
    [idlStoragePda] = PublicKey.findProgramAddressSync([Buffer.from("idl_storage"), programId.toBuffer(), userPubkey.toBuffer()], program.programId);
  });

  it("should initialize user", async () => {
    const tx = await program.methods.initializeUser().accountsStrict({
      userConfig: userPda,
      authority: userPubkey,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("tx", tx);

    const userConfig = await program.account.userConfig.fetch(userPda);
    assert.equal(userConfig.authority.toBase58(), userPubkey.toBase58());
    assert.equal(userConfig.totalTestsGenerated.toNumber(), 0);
    assert.equal(userConfig.lastGeneratedAt.toNumber(), 0);
  });

  it("should store idl data", async () => {

    
    const tx = await program.methods
      .storeIdlData(idlData, programId)
      .accountsStrict({
        idlStorage: idlStoragePda,
        authority: userPubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("tx", tx);

    const idlStorage = await program.account.idlStorage.fetch(idlStoragePda);
    assert.equal(idlStorage.authority.toBase58(), userPubkey.toBase58());
    assert.equal(idlStorage.programId.toBase58(), programId.toBase58());
    console.log("idlStorage", JSON.stringify(idlStorage.idlData, null, 2));
  });
  it("should generate metadata", async () => {
    const tx = await program.methods
      .generateMetadata(executionOrder, programId, programName)
      .accountsStrict({
        userConfig: userPda,
        idlStorage: idlStoragePda,
        testMetadataConfig: testMetadataPda,
        authority: userPubkey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("tx", tx);

    const testMetadata = await program.account.testMetadataConfig.fetch(testMetadataPda);
    console.log("testMetadata", JSON.stringify(testMetadata, null, 2));

    const userConfig = await program.account.userConfig.fetch(userPda);
    console.log("userConfig", JSON.stringify(userConfig, null, 2));
  });  
});
