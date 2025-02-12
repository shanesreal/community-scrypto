use scrypto::prelude::*;
// A proposal that unlocks funds if accepted.
blueprint! {
struct Proposal{
            // Fund which are looked for the vote
            cost_vault: Vault,
            // If proposal is accepted: Funds are send to this adress
            destination_adress_funds: Address,
            // What are users voting on?
            reason: String,
            // The token that is needed to vote.
            company_voting_token: Vault,
            // Material for voting. If the vault could count, yes and no counter wouldnt be needed
            replacement_tokens_type_yes: Vault,
            replacement_tokens_type_no: Vault,
            yes_counter: Decimal,
            no_counter: Decimal,
            // The votes needed for the vote to succeed
            needed_votes: Decimal,
            // When this epoch is reached and try_solve() is called,
            //no more voting will be possible and cost_vault will be send to owners address
            end_epoch: u64,
            fund_owner_adress: Address,
    }

            impl Proposal{

                /// creates a new instance of a proposal
                pub fn new(cost: Bucket, destination_adress_funds: Address, reason: String, admin_adress: Address, end_epoch: u64,
                    needed_votes: Decimal, company_voting_token_resource_def: ResourceDef)-> Component {

                    // The token that the user gets in exchange for their "yes" voting.
                    let replacement_token_yes_resource_def = ResourceBuilder::new()
                       .metadata("name", "Replacement token yes").metadata("symbol", "RTY")
                    .new_token_fixed(1_000_000);

                     // The token that the user gets in exchange for their "no" voting.
                     let replacement_token_no_resource_def = ResourceBuilder::new()
                     .metadata("name", "Replacement token no").metadata("symbol", "RTN")
                  .new_token_fixed(1_000_000);

                    // the vault that holds the costs that are associated with the proposal
                    let cost_vault = Vault::new(cost.resource_def());
                    // fills the cost vault
                    cost_vault.put(cost);

                  Self {
                    cost_vault: cost_vault,
                    destination_adress_funds: destination_adress_funds, reason: reason,
                    company_voting_token : Vault::new(company_voting_token_resource_def),
                    replacement_tokens_type_yes: Vault::with_bucket(replacement_token_yes_resource_def),
                    replacement_tokens_type_no: Vault::with_bucket(replacement_token_no_resource_def),
                    yes_counter: Decimal(0.0 as i128),
                    no_counter: Decimal(0.0 as i128),
                    needed_votes: needed_votes,
                    fund_owner_adress: admin_adress,
                    end_epoch: end_epoch,
                }.instantiate()
                }

                // As voting tokens are limited, users might want to retrive their voting tokens at any point to vote for a different propsal
                // for now they also have to retrive them manually after the proposale is accepted / declined
                pub fn retrive_voting_tokens(&mut self, replacement_tokens: Bucket) -> Bucket {
                    let amount = replacement_tokens.amount();
                    if self.replacement_tokens_type_yes.resource_def() == replacement_tokens.resource_def() {
                        self.yes_counter = self.yes_counter - replacement_tokens.amount();
                    } else { self.no_counter = self.no_counter - replacement_tokens.amount();}
                    // send shares from vault
                    self.company_voting_token.take(amount)

                }

                /// Allows the user to vote on an issue using his voting tokens which will be locked
                pub fn vote(&mut self, vote: bool, voting_tokens: Bucket) -> Bucket {
                    // get amount of voting tokens
                    let amount = voting_tokens.amount();
                    // Lock voting tokens in Vault
                    self.company_voting_token.put(voting_tokens);
                    // Increase correct counter & prepare replacement_tokens
                    let replacement_tokens;
                    if vote {self.yes_counter += amount; replacement_tokens =  self.replacement_tokens_type_yes.take(amount);}
                    else {self.no_counter += amount; replacement_tokens =  self.replacement_tokens_type_no.take(amount);};
                    // Send replacement_tokens to adress
                    replacement_tokens

                }
                /// Checks if a finish condition is reached. Sends funds if such a condition is reached
               pub fn try_solve(&mut self){
                   if self.yes_counter > self.needed_votes {
                    Account::from(self.destination_adress_funds).deposit(self.cost_vault.take_all());
                    info!("Proposal resolved: Positive");
                   }
                   if self.no_counter > self.needed_votes {
                    Account::from(self.fund_owner_adress).deposit(self.cost_vault.take_all());
                    info!("Proposal resolved: Negative");
                   }
                   if Context::current_epoch() > self.end_epoch {
                    Account::from(self.fund_owner_adress).deposit(self.cost_vault.take_all());
                    info!("Proposal resolved: Negative");
                   }
                   info!("No end condition reached");
                /*Needs additional Methods (not in the scope of this demo): auto-send-back of voting tokens,
                update_needed_votes (to fix vulnerability "buy a lot of shares to win vote"), and more */
                }

              // prints info about the proposal
              pub fn info(&self){
                   info!("Needed votes: {}", self.needed_votes);
                   info!("Yes votes: {}", self.yes_counter);
                   info!("No votes: {}", self.no_counter);
                   info!("Proposal end epoch: {}", self.end_epoch);
                   info!("Proposal destination addres {}", self.destination_adress_funds);
              }
            }


        }
