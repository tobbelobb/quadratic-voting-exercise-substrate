# Quadratic Voting Exercise Substrate
A learning project to get me into Substrate development. A simple quadratic voting system pallet for Substrate.

## User Stories & Development Status

 - [x] Use FRAME Identity pallet
 - [x] Users (IDs) can vote by reserving tokens
 - [x] Votes = sqrt(reserved tokens)
 - [x] Proposals: on-chain hashes
 - [ ] Simple case voting alternatives: Aye or Nay
 - [ ] Bonus: Proposing and voting on multiple things at once

## Done
 - [x] Make reservable currency available to the qv pallet
 - [x] Make identity pallet available to the qv pallet
 - [x] Reserve tokens function exists
   - [x] Test it
 - [x] Weigh reserved tokens on quadratic scale
 - [x] Represent referendums


## Want to Highlight

### The idea below
It describes a voting system that is hard to game.
It gets around the problem of who creates the referendums by splitting them into two phases.

The code uses tight coupling  with Frame's identity pallet.

The two-phase quadratic voting is implemented by piggy-backing on Frame's referendum pallet, which
uses three phases. I have implemented the "Votion launch phase" by adding some quadratic voting logic
to the referendum pallet's preparation phase.

Because of time constraints, only the launch phase was implemented.
The voting phase was not implemented, but all the referendum pallet functions for it,
and the full state machine in there, are already available.

For the code itself, I want to highlight the functions `initiate_referendum()` and `cast_launch_votes()`.

I spent a lot of time making `cast_launch_votes()` robust and well tested, as well as making the quadratic voting.

Look into `pallets/qv/tests.rs`, and run the tests:

```
cargo test -p pallet-qv
```

## Parachain Idea: Votion
On Votion we have a system for identifying users.
All users must be identified to participate.

The Votion parachain builds upon the pallet-qv that is developed in this repo.
The pallet-qv uses FRAME's official pallet-identity.
A real Votion parachain could plug in a more built out identity solution like Kilt, Litentry, or Encointer.

Every verified identity gets an amount of coins, say 1000.
Let's call them PWR, since they represent voting power.
Each user gets 1000 PWR upon joining the pool of verified users.
PWR can not be lost from the account or transmitted between users.

The Votion parachain is an engine for referendums that use quadratic vote pricing.

Referendums have two phases: launch phase and voting phase.
Each of them last 1 month.
The voting phase is only initialized if the launch phase succeeds.
If the proposal receives 1000 quadratically priced votes in the launch phase,
then a voting phase is launched automatically through on-chain logic.

During the launch phase, users vote about whether to launch a voting phase.
The only voting option is "YES" during the launch phase, and any user can only vote once.
They reserve an amount of PWR to "buy" quadratically priced YES-votes.

Users who voted in a launch phase are allowed to also vote in the voting phase, but their PWR from the launch phase remain reserved and are not available.
During the voting phase there are two voting options: "AYE" and "NEY".
Users can vote once and must cast all votes on "AYE" or all votes on "NEY", they can not split their votes.

If the launch phase of a proposal fails, all backers' PWR get unreserved.

Any user can post a referendum proposal, but must back it by reserving 1000 PWR.

When the referendum is over, all PWR reserved by proposer, backers and voters get unreserved and available to users again.
The result of any finished referendum gets recorded on-chain.
This list of referendum results is the main output and value produced by the Votion system.

Any number of referenda, up to a very high max limit, can be ongoing simultaneously at any time,
both launch phase ones and voting phase ones.

Votion can be thought of as an ocean of referenda and voting.
The word votion is also similar to the word devotion.

Votion does not try to affect what's getting voted over.
The accumulated results of voting outcomes will most likely contain self-contradictions.

For example, on day 50, we might see "Alice should color her hair green. YES: 42, NO: 10."
However, on day 51, we might get a result "Alice should not have green hair ever. YES: 20, NO: 1."
Votion does nothing to solve such contradictions.
Votion does not handle the results outside of simply posting verifiable referenda results.


## Reading List

 - [Wikipedia: Quadratic Voting](https://en.wikipedia.org/wiki/Quadratic_voting)
 - [Quadratic Voting, Lalley, Weyl (2014)](https://www.aeaweb.org/conference/2015/retrieve.php?pdfid=3009&tk=BHDG8H2E)
 - [Substrate DID Pallet (Demo Project)](https://github.com/substrate-developer-hub/pallet-did)
 - [Polkadot Wiki: Identity](https://wiki.polkadot.network/docs/learn-identity)
 - [FRAME Pallet Identity](https://paritytech.github.io/substrate/master/pallet_identity/index.html)
 - [FRAME Pallet Democracy](https://paritytech.github.io/substrate/master/pallet_democracy/index.html), [on
 - [FRAME Pallet Referenda](https://paritytech.github.io/substrate/master/pallet_referenda/index.html)
   Github](https://github.com/paritytech/substrate/tree/master/frame/democracy)
 - [Substrate Quadratic Democracy (2020)](https://github.com/MVPWorkshop/substrate-quadratic-democracy),
   [discussion](https://github.com/substrate-developer-hub/hacktoberfest/issues/22)
 - [Pallet-Quadratic-Funding](https://github.com/jakehemmerle/uc-zk-voting)
 - [How to Build Custom Pallets with Substrate](https://learn.figment.io/tutorials/how-to-build-custom-pallets-with-substrate)
 - [Alternative Identity Pallet](https://github.com/sunshine-protocol/sunshine-keybase)
