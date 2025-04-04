use anchor_lang::prelude::*;


declare_id!("ALe6xgAc8QQGpqsqo4zWTUyf6mPrsdUj8T5FrwfTBFGo");

#[program]
pub mod pred_healing_plat_onchain {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        player_id: String,
        player_name: String,
        player_age: u64,
        player_gender: u8,
        player_house: String,
        player_blood_grp: String,
        player_emergency_cont: String
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        nft.player_id = player_id;
        nft.player_name = player_name.clone();
        nft.player_age = player_age;
        nft.player_gender = player_gender;
        nft.player_house = player_house;
        nft.player_blood_grp = player_blood_grp;
        nft.player_emergency_cont = player_emergency_cont;
        nft.owner = ctx.accounts.user.key();
        nft.authorized_viewers = vec![ctx.accounts.user.key()];
        nft.last_updated_at = Clock::get()?.unix_timestamp;
        nft.update_counter = 0;
        
        msg!("Greetings from: {:?}", ctx.program_id);
        msg!("Player Trading Card initialized for: {}", player_name);
        Ok(())
    }

    pub fn update_health_data(
        ctx: Context<UpdateHealthData>, 
        health_data_hash: String,
        health_summary: Option<String>
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        
        
        if nft.health_data_history.len() >= 5 {
            nft.health_data_history.remove(0); 
        }
        
        if !nft.health_data_hash.is_empty() {
            
            let current_hash = nft.health_data_hash.clone();
            let current_timestamp = nft.last_updated_at;
            
            nft.health_data_history.push(HealthDataRecord {
                hash: current_hash,
                timestamp: current_timestamp,
            });
        }
        
        
        nft.health_data_hash = health_data_hash;
        if let Some(summary) = health_summary {
            nft.health_data_summary = summary;
        }
        
        nft.last_updated_at = Clock::get()?.unix_timestamp;
        nft.update_counter = nft.update_counter.checked_add(1).ok_or(ErrorCode::Overflow)?;
        
        msg!("Health data updated for player: {}", nft.player_name);
        Ok(())
    }

    pub fn verify_health_data(
        ctx: Context<VerifyHealthData>,
        off_chain_data_hash: String
    ) -> Result<bool> {
        let nft = &ctx.accounts.player_trading_card;
        let is_valid = nft.health_data_hash == off_chain_data_hash;
        
        msg!("Health data verification result: {}", is_valid);
        Ok(is_valid)
    }

    pub fn add_authorized_viewer(
        ctx: Context<ManageAuthorization>,
        new_viewer: Pubkey
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        
        if !nft.authorized_viewers.contains(&new_viewer) {
            nft.authorized_viewers.push(new_viewer);
            msg!("Added new authorized viewer");
        }
        
        Ok(())
    }

    pub fn remove_authorized_viewer(
        ctx: Context<ManageAuthorization>,
        viewer_to_remove: Pubkey
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        
        if nft.authorized_viewers.contains(&viewer_to_remove) && viewer_to_remove != nft.owner {
            nft.authorized_viewers.retain(|&v| v != viewer_to_remove);
            msg!("Removed authorized viewer");
        } else if viewer_to_remove == nft.owner {
            return Err(ErrorCode::CannotRemoveOwner.into());
        }
        
        Ok(())
    }

    pub fn enable_emergency_access(
        ctx: Context<EmergencyAccess>,
        emergency_pubkey: Pubkey,
        duration_seconds: i64
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        
        // Set emergency access
        nft.emergency_access = Some(EmergencyAccessInfo {
            accessor: emergency_pubkey,
            expires_at: Clock::get()?.unix_timestamp + duration_seconds,
        });
        
        msg!("Emergency access enabled for: {:?}", emergency_pubkey);
        Ok(())
    }

    pub fn disable_emergency_access(ctx: Context<EmergencyAccess>) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        nft.emergency_access = None;
        
        msg!("Emergency access disabled");
        Ok(())
    }

    pub fn update_player_info(
        ctx: Context<UpdatePlayerInfo>,
        player_name: Option<String>,
        player_age: Option<u64>,
        player_house: Option<String>,
        player_blood_grp: Option<String>,
        player_emergency_cont: Option<String>
    ) -> Result<()> {
        let nft = &mut ctx.accounts.player_trading_card;
        
        if let Some(name) = player_name {
            nft.player_name = name;
        }
        
        if let Some(age) = player_age {
            nft.player_age = age;
        }
        
        if let Some(house) = player_house {
            nft.player_house = house;
        }
        
        if let Some(blood_grp) = player_blood_grp {
            nft.player_blood_grp = blood_grp;
        }
        
        if let Some(emergency_cont) = player_emergency_cont {
            nft.player_emergency_cont = emergency_cont;
        }
        
        msg!("Player information updated");
        Ok(())
    }

    pub fn delete_player(_ctx: Context<DeletePlayer>) -> Result<()> {
        msg!("Player trading card deleted");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + PlayerTradingCard::INIT_SPACE)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateHealthData<'info> {
    #[account(mut, has_one = owner)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyHealthData<'info> {
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    pub viewer: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdatePlayerInfo<'info> {
    #[account(mut, has_one = owner)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct ManageAuthorization<'info> {
    #[account(mut, has_one = owner)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct EmergencyAccess<'info> {
    #[account(mut, has_one = owner)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeletePlayer<'info> {
    #[account(mut, close = user, constraint = player_trading_card.owner == owner.key() @ ErrorCode::NotAuthorized)]
    pub player_trading_card: Account<'info, PlayerTradingCard>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub owner: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct HealthDataRecord {
    pub hash: String,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyAccessInfo {
    pub accessor: Pubkey,
    pub expires_at: i64,
}

#[account]
pub struct PlayerTradingCard {
    
    pub player_id: String,
    pub player_name: String,
    pub player_age: u64,
    pub player_gender: u8,
    pub player_house: String,  
    pub player_blood_grp: String,
    pub player_emergency_cont: String,
    
    
    pub health_data_hash: String,
    pub health_data_summary: String,  
    pub health_data_history: Vec<HealthDataRecord>,  

    pub owner: Pubkey,
    pub authorized_viewers: Vec<Pubkey>,
    pub emergency_access: Option<EmergencyAccessInfo>,
    pub last_updated_at: i64,
    pub update_counter: u64,
}

impl PlayerTradingCard {
    const INIT_SPACE: usize = 8 +   
                               (4 + 32) +   
                               (4 + 32) +   
                               8 +          
                               1 +          
                               (4 + 32) +   
                               (4 + 32) +   
                               (4 + 32) +   
                               (4 + 64) +   
                               (4 + 128) +  
                               (4 + 5 * (64 + 8)) + 
                               32 +         
                               (4 + 10 * 32) + 
                               (1 + 32 + 8) + 
                               8 +          
                               8;           
}

#[error_code]
pub enum ErrorCode {
    #[msg("Overflow error occurred.")]
    Overflow,
    
    #[msg("Not authorized to perform this action.")]
    NotAuthorized,
    
    #[msg("Cannot remove owner from authorized viewers.")]
    CannotRemoveOwner,
    
    #[msg("Emergency access has expired.")]
    EmergencyAccessExpired,
}