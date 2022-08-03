use crate::{mock::*, Error};
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::RawOrigin, BoundedVec};

use pallet_balances::Error as BalancesError;
use pallet_identity::{Data, IdentityInfo};
use pallet_referenda::{Error as ReferendaError, ReferendumCount};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	DispatchError,
};

use crate::Event as QvEvent;

fn last_event() -> QvEvent<Test> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let Event::Qv(inner) = e { Some(inner) } else { None })
		.last()
		.unwrap()
}

const SMALL_AMOUNT: u64 = 1;

#[test]
fn try_reserve_bad_origin() {
	new_test_ext().execute_with(|| {
		// Unsigned origin
		assert_noop!(
			Qv::reserve_an_amount_of_token(Origin::none(), SMALL_AMOUNT),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn try_reserve_no_identity() {
	new_test_ext().execute_with(|| {
		// User has an no registered identity
		assert_noop!(
			Qv::reserve_an_amount_of_token(Origin::signed(1), SMALL_AMOUNT),
			Error::<Test>::NoIdentity
		);
	});
}

fn info() -> IdentityInfo<MaxAdditionalFields> {
	IdentityInfo {
		additional: BoundedVec::default(),
		display: Data::Raw(b"name".to_vec().try_into().unwrap()),
		legal: Data::default(),
		web: Data::Raw(b"website".to_vec().try_into().unwrap()),
		riot: Data::default(),
		email: Data::default(),
		pgp_fingerprint: None,
		image: Data::default(),
		twitter: Data::default(),
	}
}

#[test]
fn try_reserve_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let who = Origin::signed(1);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_noop!(
			Qv::reserve_an_amount_of_token(who, SMALL_AMOUNT),
			BalancesError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn try_successful_zero_reserve() {
	new_test_ext().execute_with(|| {
		let who = Origin::signed(1);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), 0));
	});
}

#[test]
fn try_successful_nonzero_reserve() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), SMALL_AMOUNT));

		// Identity pallet should have emitted an event
		System::assert_has_event(Event::Identity(pallet_identity::Event::IdentitySet { who: 10 }));
		// Checks that the correct event was emitted by pallet-qv
		assert_eq!(last_event(), QvEvent::AmountReserved(SMALL_AMOUNT));
	});
}

#[test]
fn successive_reserves_until_out_of_funds() {
	new_test_ext().execute_with(|| {
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), 80));
		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), 10));
		assert_noop!(
			Qv::reserve_an_amount_of_token(who.clone(), 10),
			BalancesError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn unreserve_then_reserve_again() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), 90));
		assert_noop!(
			Qv::reserve_an_amount_of_token(who.clone(), 10),
			BalancesError::<Test>::InsufficientBalance
		);
		const UNRESERVE_AMOUNT: u64 = 11;
		assert_ok!(Qv::unreserve_an_amount_of_token(RawOrigin::Root.into(), 10, UNRESERVE_AMOUNT));

		// Checks that the correct event was emitted
		assert_eq!(last_event(), QvEvent::AmountUnreserved(UNRESERVE_AMOUNT));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), 10));
		assert_noop!(
			Qv::reserve_an_amount_of_token(who.clone(), 10),
			BalancesError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn initiate_referendum_insufficient_balance() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let who = Origin::signed(1);
		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		let proposal = BlakeTwo256::hash_of(&1);

		assert_eq!(Balances::free_balance(1), 0);
		assert_noop!(
			Qv::initiate_referendum(who, proposal),
			BalancesError::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn initiate_referendum_happy_case() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let who = Origin::signed(30);
		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		let proposal_hash = BlakeTwo256::hash_of(&1);

		assert_eq!(Balances::free_balance(30), 1000);
		assert_ok!(Qv::initiate_referendum(who.clone(), proposal_hash));
		assert_eq!(Balances::free_balance(30), 0);

		System::assert_has_event(Event::Referenda(pallet_referenda::Event::Submitted {
			index: 0,
			track: 0,
			proposal_hash,
		}));

		assert_eq!(ReferendumCount::<Test>::get(), 1);
	});
}

#[test]
fn cast_single_launch_vote() {
	new_test_ext().execute_with(|| {
		// Events are not populated in the genesis block
		System::set_block_number(1);
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter = Origin::signed(20);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));
		assert_ok!(Qv::cast_launch_votes(launch_voter, 1, 0));

		assert_eq!(last_event(), QvEvent::LaunchVotesCast { number_of_votes: 1, index: 0 });
	});
}

#[test]
fn cast_zero_votes() {
	new_test_ext().execute_with(|| {
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter = Origin::signed(20);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));
		assert_eq!(Balances::free_balance(20), 100);
		assert_noop!(Qv::cast_launch_votes(launch_voter, 0, 0), Error::<Test>::ZeroVote);
		assert_eq!(Balances::free_balance(20), 100);
	});
}

#[test]
fn cast_more_launch_votes_than_allowed() {
	new_test_ext().execute_with(|| {
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter = Origin::signed(20);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));
		assert_eq!(Balances::free_balance(20), 100);
		assert_noop!(
			Qv::cast_launch_votes(launch_voter, 11, 0),
			BalancesError::<Test>::InsufficientBalance
		); // 11*11 == 121 and 121 > 100
		assert_eq!(Balances::free_balance(20), 100);
	});
}

#[test]
fn try_cast_vote_no_identity() {
	new_test_ext().execute_with(|| {
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter = Origin::signed(20);
		assert_noop!(Qv::cast_launch_votes(launch_voter, 1, 0), Error::<Test>::NoIdentity);
	});
}

#[test]
fn try_cast_vote_not_ongoing() {
	new_test_ext().execute_with(|| {
		let launch_voter = Origin::signed(20);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));
		assert_noop!(Qv::cast_launch_votes(launch_voter, 1, 0), ReferendaError::<Test>::NotOngoing);
	});
}

#[test]
fn cast_launch_votes_twice() {
	new_test_ext().execute_with(|| {
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter = Origin::signed(31);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));

		assert_ok!(Qv::cast_launch_votes(launch_voter.clone(), 5, 0));
		assert_noop!(Qv::cast_launch_votes(launch_voter, 1, 0), Error::<Test>::AlreadyVoted);
	});
}

#[test]
fn initiator_tries_to_cast_launch_votes() {
	new_test_ext().execute_with(|| {
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator.clone(), proposal_hash));

		assert_noop!(
			Qv::cast_launch_votes(referendum_initiator, 1, 0),
			Error::<Test>::AlreadyVoted
		);
	});
}

#[test]
fn cast_launch_votes_until_full_deposit_triggered_and_beyond() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		let referendum_initiator = Origin::signed(30);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let launch_voter_0 = Origin::signed(31);
		assert_ok!(Identity::set_identity(launch_voter_0.clone(), Box::new(info())));
		let launch_voter_1 = Origin::signed(32);
		assert_ok!(Identity::set_identity(launch_voter_1.clone(), Box::new(info())));

		assert_eq!(Balances::free_balance(31), 250_000);
		assert_ok!(Qv::cast_launch_votes(launch_voter_0, 500, 0)); // Cast 500 votes
		assert_eq!(Balances::free_balance(31), 0);

		assert_eq!(Balances::free_balance(32), 250_000);
		assert_ok!(Qv::cast_launch_votes(launch_voter_1, 500, 0)); // Cast 500 votes
		assert_eq!(Balances::free_balance(32), 0);
		System::assert_has_event(Event::Referenda(
			pallet_referenda::Event::DecisionDepositPlaced { index: 0, who: 32, amount: 250_000 },
		));

		let launch_voter_2 = Origin::signed(20);
		assert_ok!(Identity::set_identity(launch_voter_2.clone(), Box::new(info())));

		assert_noop!(
			Qv::cast_launch_votes(launch_voter_2, 1, 0),
			ReferendaError::<Test>::HasDeposit,
		);
	});
}

#[test]
fn launch_phase_can_get_cancelled_by_root_leads_to_slash() {
	new_test_ext().execute_with(|| {
		// Events are not populated in the genesis block
		System::set_block_number(1);
		let initiator_num = 30;
		let referendum_initiator = Origin::signed(initiator_num);
		assert_ok!(Identity::set_identity(referendum_initiator.clone(), Box::new(info())));
		let proposal_hash = BlakeTwo256::hash_of(&1);
		assert_ok!(Qv::initiate_referendum(referendum_initiator, proposal_hash));

		let voter_num = 20;
		let launch_voter = Origin::signed(voter_num);
		assert_ok!(Identity::set_identity(launch_voter.clone(), Box::new(info())));
		assert_ok!(Qv::cast_launch_votes(launch_voter, 10, 0));

		// Both actors have zero balances
		assert_eq!(Balances::free_balance(initiator_num), 0);
		assert_eq!(Balances::free_balance(voter_num), 0);

		assert_noop!(
			Qv::refund_launch_votes(RawOrigin::Root.into(), 0),
			Error::<Test>::StillOngoing
		);

		assert_ok!(Referenda::cancel(RawOrigin::Root.into(), 0));

		assert_err!(
			Qv::refund_launch_votes(RawOrigin::Root.into(), 0),
			ReferendaError::<Test>::NoDeposit
		);

		assert_eq!(Balances::free_balance(initiator_num), 0);
		assert_eq!(Balances::free_balance(voter_num), 0);
	});
}
