import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { TrustifyAnchor } from "../target/types/trustify_anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

describe("trustify-anchor", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TrustifyAnchor as Program<TrustifyAnchor>;
  const wallet = provider.wallet;

  // For testing, we'll create a new keypair to act as the trusted forwarder
  const trustedForwarder = Keypair.generate();

  // Test constants
  const NFT_NAME = "TestNFT";
  const NFT_SYMBOL = "TNFT";
  const TOKEN_URI = "https://example.com/metadata.json";

  // Store the program data PDA
  let programDataPDA: PublicKey;

  before(async () => {
    // Fund the trusted forwarder for testing
    const airdropSig = await provider.connection.requestAirdrop(
      trustedForwarder.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    // Find the program data PDA
    [programDataPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("program_data")],
      program.programId
    );
  });

  it("Initialize the program", async () => {
    try {
      // Initialize the program
      await program.methods
        .initialize(NFT_NAME, NFT_SYMBOL)
        .accounts({
          authority: wallet.publicKey,
          trustedForwarder: trustedForwarder.publicKey,
          programData: programDataPDA,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      // Fetch the program data to verify it was initialized correctly
      const programData = await program.account.programData.fetch(
        programDataPDA
      );

      assert.equal(
        programData.authority.toString(),
        wallet.publicKey.toString()
      );
      assert.equal(
        programData.trustedForwarder.toString(),
        trustedForwarder.publicKey.toString()
      );
      assert.equal(programData.name, NFT_NAME);
      assert.equal(programData.symbol, NFT_SYMBOL);
      assert.equal(programData.tokenCounter.toNumber(), 0);
    } catch (error) {
      console.error("Error initializing program:", error);
      throw error;
    }
  });

  it("Can mint an NFT", async () => {
    try {
      // Create a mint account for the NFT
      const mintKeypair = Keypair.generate();

      // Find mint authority PDA
      const [mintAuthority] = await PublicKey.findProgramAddress(
        [Buffer.from("mint_authority"), mintKeypair.publicKey.toBuffer()],
        program.programId
      );

      // Get token metadata program ID
      const tokenMetadataProgramId = new PublicKey(
        "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s"
      );

      // Find metadata account PDA
      const [metadataAccount] = await PublicKey.findProgramAddress(
        [
          Buffer.from("metadata"),
          tokenMetadataProgramId.toBuffer(),
          mintKeypair.publicKey.toBuffer(),
        ],
        tokenMetadataProgramId
      );

      // Get the associated token account for the recipient
      const tokenAccount = await getAssociatedTokenAddress(
        mintKeypair.publicKey,
        wallet.publicKey
      );

      // Mint the NFT
      await program.methods
        .mint_nft(TOKEN_URI)
        .accounts({
          signer: wallet.publicKey,
          trustedForwarder: trustedForwarder.publicKey,
          programData: programDataPDA,
          mint: mintKeypair.publicKey,
          tokenAccount: tokenAccount,
          recipient: wallet.publicKey,
          metadata: metadataAccount,
          mintAuthority: mintAuthority,
          tokenMetadataProgram: tokenMetadataProgramId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair])
        .rpc();

      // Verify the token was minted
      const tokenAccountInfo = await getAccount(
        provider.connection,
        tokenAccount
      );

      assert.equal(tokenAccountInfo.amount.toString(), "1");
      assert.equal(
        tokenAccountInfo.mint.toString(),
        mintKeypair.publicKey.toString()
      );
      assert.equal(
        tokenAccountInfo.owner.toString(),
        wallet.publicKey.toString()
      );

      // Verify the token counter was incremented
      const programData = await program.account.programData.fetch(
        programDataPDA
      );
      assert.equal(programData.tokenCounter.toNumber(), 1);
    } catch (error) {
      console.error("Error minting NFT:", error);
      throw error;
    }
  });
});
