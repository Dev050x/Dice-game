import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DiceGame } from "../target/types/dice_game";
import { Keypair, LAMPORTS_PER_SOL, NONCE_ACCOUNT_LENGTH } from "@solana/web3.js";
import nacl from "tweetnacl";

describe("dice-game", () => {
  //here provider is house
  //player is player
  // Configure the client to use the local cluster.
  let proivder = anchor.AnchorProvider.env();
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.diceGame as Program<DiceGame>;
  console.log("program id is: ", program.programId);

  let user = Keypair.generate();
  let seed = new anchor.BN(BigInt(1));
  let vault = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("vault") , proivder.wallet.publicKey.toBytes()] , program.programId)[0];
  let bet = anchor.web3.PublicKey.findProgramAddressSync([Buffer.from("bet") , vault.toBytes() , Buffer.from(seed.toArray("le" , 16))] , program.programId)[0];
  const STARTINT_HOUSE_VAULT_BALANCE = new anchor.BN(100 * LAMPORTS_PER_SOL);

  before(async () => {
    //sending some sol to user
    let tx = new anchor.web3.Transaction().add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey:proivder.wallet.publicKey,
        toPubkey: user.publicKey,
        lamports:3 * LAMPORTS_PER_SOL
      })
    )
    let sig = await proivder.sendAndConfirm(tx);
    console.log("sol sent to user: ", sig);
  })

  it("Creating the house", async () => {
    // Add your test here.
    const tx = await program
              .methods
              .initialize(STARTINT_HOUSE_VAULT_BALANCE)
              .accountsPartial({
                house:proivder.wallet.publicKey,
                vault,
                systemProgram:anchor.web3.SystemProgram.programId,
              })
              .rpc();
    console.log("House Created Succefully", tx);
    let house_vault_balance:number = Number((await proivder.connection.getBalance(vault)).toString()) / LAMPORTS_PER_SOL;
    let player_balance:number = Number((await proivder.connection.getBalance(user.publicKey)).toString()) / LAMPORTS_PER_SOL;
    console.log("house vault balance: ", house_vault_balance);
    console.log("Player balance: ", player_balance);
  });

  it("Placing bet" , async () => {
    let amount = new anchor.BN(2 * LAMPORTS_PER_SOL);
    const tx = await program
              .methods
              .placeBet(seed,15,amount)
              .accountsPartial({
                player:user.publicKey,
                house:proivder.wallet.publicKey,
                vault,
                bet,
                systemProgram:anchor.web3.SystemProgram.programId,
              })
              .signers([user])
              .rpc()
    console.log("Bet Succefully Placed", tx);
    let house_vault_balance:number = Number((await proivder.connection.getBalance(vault)).toString()) / LAMPORTS_PER_SOL;
    let player_balance:number = Number((await proivder.connection.getBalance(user.publicKey)).toString()) / LAMPORTS_PER_SOL;
    console.log("house vault balance: ", house_vault_balance);
    console.log("Player balance: ", player_balance);
              
  })

  it.skip("refund to bet" , async () => {
    await sleep(1000);
    let amount = new anchor.BN(2 * LAMPORTS_PER_SOL);
    const tx = await program
              .methods
              .refund()
              .accountsPartial({
                player:user.publicKey,
                house:proivder.wallet.publicKey,
                vault,
                bet,
                systemProgram:anchor.web3.SystemProgram.programId,
              })
              .signers([user])
              .rpc()
    console.log("refunded bet", tx);
    let house_vault_balance:number = Number((await proivder.connection.getBalance(vault)).toString()) / LAMPORTS_PER_SOL;
    let player_balance:number = Number((await proivder.connection.getBalance(user.publicKey)).toString()) / LAMPORTS_PER_SOL;
    console.log("house vault balance: ", house_vault_balance);
    console.log("Player balance: ", player_balance);
              
  })

  it("Resolving Bet" , async () => {
    let amount = new anchor.BN(2 * LAMPORTS_PER_SOL);
    const betAccount = await program.account.bet.fetch(bet);
    const seedBytes = Buffer.from(new anchor.BN(betAccount.seed).toArray("le", 16));
    const amountBytes = Buffer.from(new anchor.BN(betAccount.amount).toArray("le", 8));
    const slotBytes = Buffer.from(new anchor.BN(betAccount.slot).toArray("le", 8));
    const rollByte = Buffer.from([betAccount.roll]);
    const bumpByte = Buffer.from([betAccount.bump]);

    const message = Buffer.concat([
      betAccount.player.toBytes(),
      seedBytes,
      amountBytes,
      slotBytes,
      rollByte,
      bumpByte
    ])
    const signature = Buffer.from(nacl.sign.detached(message, proivder.wallet.payer.secretKey));
    const ed25519Ix = anchor.web3.Ed25519Program.createInstructionWithPublicKey({
      publicKey: proivder.wallet.publicKey.toBytes(),
      message,
      signature,
    });

    const tx = await program
              .methods
              .resolveBet(signature)
              .accountsPartial({
                house:proivder.wallet.publicKey,
                player:user.publicKey,
                vault,
                bet,
                instructionSysvar:anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
                systemProgram:anchor.web3.SystemProgram.programId,
              })
              .preInstructions([ed25519Ix])
              .rpc()
    console.log("Bet Resolved", tx);
    let house_vault_balance:number = Number((await proivder.connection.getBalance(vault)).toString()) / LAMPORTS_PER_SOL;
    let player_balance:number = Number((await proivder.connection.getBalance(user.publicKey)).toString()) / LAMPORTS_PER_SOL;
    console.log("house vault balance: ", house_vault_balance);
    console.log("Player balance: ", player_balance);
              
  })

});

async function sleep(ms:number){
    return new Promise(resolve => setTimeout(resolve , ms));
}
