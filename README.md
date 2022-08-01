# Quadratic Voting Exercise Substrate
A learning project to get me into Substrate development. A simple quadratic voting system pallet for Substrate.

## User Stories & Development Status

 - [ ] Use FRAME Identity pallet
 - [ ] Users (IDs) can vote by reserving tokens
 - [ ] Votes = sqrt(reserved tokens)
 - [ ] Proposals: on-chain hashes
 - [ ] Simple case voting alternatives: Aye or Nay
 - [ ] Bonus: Proposing and voting on multiple things at once

## Done
 - [x] Make reservable currency available to the qv pallet
 - [x] Make identity pallet available to the qv pallet
 - [x] Reserve tokens function exists
   - [x] Test it
 - [x] Weigh reserved tokens on quadratic scale
 - [ ] Represent referendums

## Backlog
 - [ ] In release builds, the events should not expose voters too much. Bribery and buying others' votes should be made harder by making it impossible to prove to someone
   else that one voted in a particular way.

## Parachain Idea: Votion
On Votion we only have identified users.
It builds upon the pallet-qv from this repo.
The pallet-qv uses pallet-identity for now, but the real Votion parachain should plug in a better identity solution like Kilt, Litentry, or even Encointer.

Every verified identity gets an amount of coins, say 1000.
Let's call them POW, as in power, so each user gets 1000 POW.

The parachain is an engine for referendums that use quadratic vote pricing.
Any user can post a referendum proposal, backing it with a minimum of 1 POW -> 1 BAK.
Any user can back any referendum proposal only once.
The proposal gets backed by an amount of BAK equal to the square root of how much POW the user pays.

The POW that the user sends gets reserved.
They are sent back to the users when the proposal fails, or when an eventual referendum is over.

The referendum is in the proposal phase for 1 month.
If the proposal receives 1000 BAK within 1 month, then it will go into a voting phase after that month.

A voting phase lasts 1 month.
During a voting phase, any user can vote one time by sending POW.
The referendum receives a number of YESs or NOs for each user that sends POW.
Each user must vote either YES or NO, and back that by reserving some amount of POW.
The number of YES/NO, again, is equal to the square root of the number of POW sent by the specific user.

POW is an on-chain coin. BAK, YES, and NO need not be coins. They are used here to make it easier to describe vote pricing.

When the referendum is over, all POW sent to mint BAK, YES, and NO for the referendu gets unreserved, to become available to users again.
The result of any finished referendum gets recorded on-chain.
This list of referendum results is the main output and value produced by the Votion system.

Any number of referenda can be ongoing at any time, both proposal phase ones and voting phase ones.
Votion can be thought of as an ocean of referenda and voting.
The word votion is also similar to the word devotion.

Votion does not try to affect what's getting voted over.
The accumulated results of voting outcomes will most likely contains self-contradictions.

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
