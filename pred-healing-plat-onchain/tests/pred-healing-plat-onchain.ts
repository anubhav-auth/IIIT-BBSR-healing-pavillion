import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { assert } from 'chai';
import { PredHealingPlatOnchain } from '../target/types/pred_healing_plat_onchain';


describe('pred_healing_plat_onchain', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PredHealingPlatOnchain as Program<PredHealingPlatOnchain>;
  
  // Generate keypairs
  const owner = Keypair.generate();
  const unauthorizedUser = Keypair.generate();
  const playerCard = Keypair.generate();

  // Test data
  const playerId = "P001";
  const playerName = "Harry Potter";
  const playerAge = new anchor.BN(17);
  const playerGender = 1; // Assuming 1 for male
  const playerHouse = "Gryffindor";
  const playerBloodGrp = "O+";
  const playerEmergencyCont = "+44 123 456 7890";
  const healthDataHash = "abcdef1234567890abcdef1234567890";
  const healthSummary = "Player is in good health with minor bruising from Quidditch";
  const unauthorizedHash = "unauthorized_hash_attempt";
  const unauthorizedSummary = "Unauthorized summary";

  before(async () => {
    // Airdrop SOL to the owner
    await provider.connection.requestAirdrop(owner.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Airdrop SOL to the unauthorized user for testing
    await provider.connection.requestAirdrop(unauthorizedUser.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    
    // Wait for confirmations
    await new Promise(resolve => setTimeout(resolve, 1000));
  });

  it('Creates a player trading card NFT', async () => {
    // Initialize player trading card
    await program.methods.initialize(
      playerId,
      playerName,
      playerAge,
      playerGender,
      playerHouse,
      playerBloodGrp,
      playerEmergencyCont
    )
    .accounts({
      playerTradingCard: playerCard.publicKey,
      user: owner.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .signers([owner, playerCard])
    .rpc();

    // Fetch the created account
    const account = await program.account.playerTradingCard.fetch(playerCard.publicKey);
    
    // Verify the account data
    assert.equal(account.playerId, playerId);
    assert.equal(account.playerName, playerName);
    assert.ok(account.playerAge.eq(playerAge));
    assert.equal(account.playerGender, playerGender);
    assert.equal(account.playerHouse, playerHouse);
    assert.equal(account.playerBloodGrp, playerBloodGrp);
    assert.equal(account.playerEmergencyCont, playerEmergencyCont);
    assert.equal(account.owner.toString(), owner.publicKey.toString());
    assert.equal(account.authorizedViewers.length, 1);
    assert.equal(account.authorizedViewers[0].toString(), owner.publicKey.toString());
    assert.equal(account.updateCounter.toNumber(), 0);
    
    console.log("✅ Successfully created and verified player trading card");
  });

  it('Updates health data as authorized owner', async () => {
    await program.methods.updateHealthData(
      healthDataHash,
      healthSummary
    )
    .accounts({
      playerTradingCard: playerCard.publicKey,
      owner: owner.publicKey,
    })
    .signers([owner])
    .rpc();

    // Fetch the updated account
    const account = await program.account.playerTradingCard.fetch(playerCard.publicKey);
    
    // Verify the updated data
    assert.equal(account.healthDataHash, healthDataHash);
    assert.equal(account.healthDataSummary, healthSummary);
    assert.equal(account.updateCounter.toNumber(), 1);
    
    console.log("✅ Successfully updated health data as authorized owner");
  });

  it('Fails to update health data as unauthorized user', async () => {
    try {
      await program.methods.updateHealthData(
        unauthorizedHash,
        unauthorizedSummary
      )
      .accounts({
        playerTradingCard: playerCard.publicKey,
        owner: unauthorizedUser.publicKey,
      })
      .signers([unauthorizedUser])
      .rpc();
      
      // If we get here, the transaction succeeded, which is not what we want
      assert.fail("Transaction should have failed with unauthorized user");
    } catch (err) {
      // This is expected - we want the transaction to fail
      console.log("✅ Correctly rejected update from unauthorized user");
    }

    // Verify the data was not changed
    const account = await program.account.playerTradingCard.fetch(playerCard.publicKey);
    assert.equal(account.healthDataHash, healthDataHash);
    assert.notEqual(account.healthDataHash, unauthorizedHash);
  });

  it('Adds an authorized viewer', async () => {
    await program.methods.addAuthorizedViewer(
      unauthorizedUser.publicKey
    )
    .accounts({
      playerTradingCard: playerCard.publicKey,
      owner: owner.publicKey,
    })
    .signers([owner])
    .rpc();

    // Fetch the updated account
    const account = await program.account.playerTradingCard.fetch(playerCard.publicKey);
    
    // Verify the viewer was added
    assert.equal(account.authorizedViewers.length, 2);
    assert.equal(account.authorizedViewers[1].toString(), unauthorizedUser.publicKey.toString());
    
    console.log("✅ Successfully added authorized viewer");
  });

  it('Removes an authorized viewer', async () => {
    await program.methods.removeAuthorizedViewer(
      unauthorizedUser.publicKey
    )
    .accounts({
      playerTradingCard: playerCard.publicKey,
      owner: owner.publicKey,
    })
    .signers([owner])
    .rpc();

    // Fetch the updated account
    const account = await program.account.playerTradingCard.fetch(playerCard.publicKey);
    
    // Verify the viewer was removed
    assert.equal(account.authorizedViewers.length, 1);
    
    console.log("✅ Successfully removed authorized viewer");
  });

  it('Deletes the player trading card', async () => {
    await program.methods.deletePlayer()
    .accounts({
      playerTradingCard: playerCard.publicKey,
      user: owner.publicKey,
      owner: owner.publicKey,
    })
    .signers([owner])
    .rpc();

    // Try to fetch the account (should fail as it was deleted)
    try {
      await program.account.playerTradingCard.fetch(playerCard.publicKey);
      assert.fail("Account should be deleted");
    } catch (err) {
      // This is expected as the account should be deleted
      console.log("✅ Successfully verified player trading card deletion");
    }
  });
});