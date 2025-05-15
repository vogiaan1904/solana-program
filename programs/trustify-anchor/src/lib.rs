use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata},
    token::{Mint, Token, TokenAccount},
};
use mpl_token_metadata::types::DataV2;

declare_id!("CxQNPSEJtPtqT55NYzZ6VgwFR2jcvJfviTSdFPbVkz11");

#[program]
pub mod trustify_anchor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, symbol: String) -> Result<()> {
        let program_data = &mut ctx.accounts.program_data;

        program_data.authority = ctx.accounts.authority.key();
        program_data.trusted_forwarder = ctx.accounts.trusted_forwarder.key();
        program_data.name = name;
        program_data.symbol = symbol;
        program_data.token_counter = 0;

        Ok(())
    }

    pub fn mint_nft(ctx: Context<MintNFT>, token_uri: String) -> Result<()> {
        // Get the token counter from program data
        let program_data = &mut ctx.accounts.program_data;
        let token_counter = program_data.token_counter;

        // Increment the token counter
        program_data.token_counter = program_data.token_counter.checked_add(1).unwrap();

        // Create metadata for the NFT
        let seeds = &["program_data".as_bytes()];
        let bump = &[ctx.bumps.program_data];
        let signer_seeds = &[seeds[0], bump];

        // Get the real transaction signer (could be forwarder or original sender)
        let sender = if ctx.accounts.trusted_forwarder.key() == program_data.trusted_forwarder {
            // In a real implementation, you would extract the original sender from tx data
            // For now, we'll use the original signer
            ctx.accounts.signer.key()
        } else {
            ctx.accounts.signer.key()
        };

        // Mint 1 token to the recipient
        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.mint_authority.to_account_info(), // FIXED: Use mint_authority PDA
                },
            ).with_signer(&[&[
                "mint_authority".as_bytes(),
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]]),
            1,
        )?;

        // Create metadata for the NFT using token-metadata program
        let creators = vec![mpl_token_metadata::types::Creator {
            address: sender,
            verified: false,
            share: 100,
        }];

        let data_v2 = DataV2 {
            name: program_data.name.clone(),
            symbol: program_data.symbol.clone(),
            uri: token_uri,
            seller_fee_basis_points: 0,
            creators: Some(creators),
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    mint_authority: ctx.accounts.mint_authority.to_account_info(),
                    update_authority: ctx.accounts.signer.to_account_info(),
                    payer: ctx.accounts.signer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
            ).with_signer(&[&[
                "mint_authority".as_bytes(),
                ctx.accounts.mint.key().as_ref(),
                &[ctx.bumps.mint_authority],
            ]]),
            data_v2,
            true, // is_mutable
            true, // update_authority_is_signer
            None, // collection_details
        )?;

        Ok(())
    }
}

#[account]
pub struct ProgramData {
    pub authority: Pubkey,         // Owner of the contract
    pub trusted_forwarder: Pubkey, // Address of the trusted forwarder (for gasless tx)
    pub name: String,              // NFT collection name
    pub symbol: String,            // NFT collection symbol
    pub token_counter: u64,        // Counter for NFT IDs
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: This is the trusted forwarder for meta-transactions
    pub trusted_forwarder: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 50 + 10 + 8, // Discriminator + authority + forwarder + name + symbol + counter
        seeds = ["program_data".as_bytes()],
        bump
    )]
    pub program_data: Account<'info, ProgramData>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: This is the trusted forwarder for meta-transactions
    pub trusted_forwarder: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = ["program_data".as_bytes()],
        bump
    )]
    pub program_data: Account<'info, ProgramData>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = mint_authority,
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: This account will be created by the token program
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,

    /// CHECK: This is the recipient of the NFT
    pub recipient: UncheckedAccount<'info>,

    /// CHECK: Metadata account for the token
    #[account(
        mut,
        seeds = [
            b"metadata",
            token_metadata_program.key().as_ref(),
            mint.key().as_ref(),
        ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: PDA to serve as mint authority
    #[account(
        seeds = [
            "mint_authority".as_bytes(),
            mint.key().as_ref(),
        ],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,

    /// CHECK: Token Metadata Program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
