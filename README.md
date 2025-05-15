# Solana Anchor NFT Program

A Solana Anchor implementation of NFT minting with gasless transaction support, similar to the ERC721 GaslessMintNFT Ethereum contract.

## Features

- NFT minting with Metaplex metadata standards
- Token URI storage for metadata
- Gasless transaction support via trusted forwarder pattern
- Automatic token ID increments (similar to ERC721 tokenCounter)
- Initialization with custom name and symbol

## Project Structure

```
Solana-anchor/
├── programs/                # Anchor program source code
│   └── Solana-anchor/
│       ├── src/lib.rs       # Main program logic
│       └── Cargo.toml       # Program dependencies
├── client/                  # TypeScript client utilities
│   ├── adapter.ts           # Client adapter for the Anchor program
│   └── example.ts           # Example usage
├── tests/                   # Program tests
│   └── Solana-anchor.ts   # Test suite
├── contractIDL.json         # Program IDL for client integration
└── Anchor.toml              # Anchor configuration
```

## Prerequisites

- Rust and Cargo
- Solana CLI tools
- Anchor CLI (v0.28.0 or later)
- Node.js and npm/yarn

## Setup and Installation

1. Install dependencies:

```bash
# Install Solana CLI tools
sh -c "$(curl -sSfL https://release.solana.com/v1.16.0/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install 0.28.0
avm use 0.28.0

# Install project dependencies
yarn install
```

2. Build the Anchor program:

```bash
anchor build
```

3. Deploy locally:

```bash
anchor test
```

4. Deploy to devnet or mainnet:

```bash
# Update your Anchor.toml with the correct cluster
solana config set --url devnet
anchor deploy
```

## Using the Anchor Program

### Program Initialization

Initialize the program with a name and symbol:

```typescript
// Initialize the adapter
const adapter = new SolanaAnchorAdapter(
  connection,
  keypair,
  programId,
  trustedForwarder
);

// Initialize the program (only needs to be done once)
await adapter.initialize("MyNFT", "MNFT");
```

### Minting NFTs

Mint an NFT to a recipient with metadata:

```typescript
// Create a mint keypair for the NFT
const mintKeypair = Keypair.generate();

// Mint the NFT
const tx = await adapter.mintNFT(mintKeypair, recipientPublicKey, metadataUri);
```

### Gasless Transactions

The program supports a trusted forwarder pattern similar to ERC2771 for gasless transactions.

1. The trusted forwarder account is set during initialization
2. The forwarder can submit transactions on behalf of users
3. For full gasless implementation, additional server-side relay is required

## Program Architecture

### Accounts

- `ProgramData`: Stores program state (authority, trusted forwarder, name, symbol, token counter)
- `MintNFT`: Instruction account context for NFT minting

### Instructions

- `initialize`: Sets up the program with a name, symbol, and trusted forwarder
- `mint_nft`: Mints a new NFT with metadata to a recipient

### PDAs (Program Derived Addresses)

- `program_data`: Main state account (seeds = ["program_data"])
- `mint_authority`: Authority for minting tokens (seeds = ["mint_authority", mint_pubkey])

## Client Integration

The client adapter (`client/adapter.ts`) provides a simple interface for interacting with the program.

Example integration with IPFS/Pinata for storing NFT metadata:

```typescript
// Upload file to IPFS
const fileUrl = await uploadToPinata(fileBuffer, "document.pdf");

// Create and upload metadata
const metadataUrl = await uploadMetadataToPinata(fileUrl, "Document Name");

// Mint NFT with the metadata
const result = await mintDocumentNFT(metadataUrl, recipientAddress);
```

## Testing

Run the test suite:

```bash
anchor test
```

The tests verify:

- Program initialization
- NFT minting
- Token ownership
- Token counter increments

## License

MIT

## Credits

This project adapts concepts from:

- [GaslessMintNFT-Contract](https://github.com/sloweyyy/GaslessMintNFT-Contract/tree/main/)
- Metaplex Token Metadata program
- Solana SPL Token program
