This mainly describes my limited understanding of what I'm about to build going into this project.

The system handles administration related to quadratic voting.
The system will run on a blockchain where nodes share relevant state, such as identities of
voters, what can they vote on, and the actual votes.
Both proposals, votes, and results are registered through blockchain transactions and
the system is reachable via the blockchain's nodes.


The system as viewed from a node, can be described through its
 - [Events](#Events)
 - [Errors](#Errors)
 - [Storage](#Storage)

As a Substrate Module, the system also has some
 - [Public Functions](#Public Functions)

### Events
 - Identity Generated
 - Identity Claimed
 - Identity Deleted
 - Tokens Reserved
 - Proposal Registered
 - Something Stored

### Errors
 - Storage Overflow
 - Identity Not Found
 - Identity Already Claimed
 - Invalid Vote: Out of Funds
 - Invalid Vote: Voting Ended Already
 - Invalid Vote: Proposal Not Found
 - Invalid Vote: Voting Option Not

### Storage
 - Identity Map: Identity => AccountId
 - Proposal 

### Public Functions
 - `create_identity(Identity)`
 - `delete_identity(Identity)`
 - `propose(Proposal)`
 - `vote(ProposalId, Amount, Signature)`


### Data Types
 - `Proposal`
   * `Signature`
   * `Deposit`
   * Block height interval: {start, stop}
 - `Identity`
