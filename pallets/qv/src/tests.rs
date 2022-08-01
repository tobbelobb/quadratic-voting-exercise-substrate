use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

use pallet_balances::Error as BalancesError;
use pallet_identity::{Data, IdentityInfo};
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	DispatchError,
};

use crate::Event as QvEvent;

/// TODO: there's something called something like
/// frame_system::pallet::<T>::assert_has_event!()
/// and assert_last_event!() that we should maybe use instead
/// of rolling our own here.
fn last_event() -> QvEvent<Test> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| if let Event::Qv(inner) = e { Some(inner) } else { None })
		.last()
		.unwrap()
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(Qv::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
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

		// Checks that the correct event was emitted
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
		assert_ok!(Qv::unreserve_an_amount_of_token(who.clone(), UNRESERVE_AMOUNT));

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
fn cast_single_vote() {
	new_test_ext().execute_with(|| {
		// Events are not populated in the genesis block
		System::set_block_number(1);
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::cast_votes(who.clone(), 1));

		// Should use assert_last_event!()
		assert_eq!(last_event(), QvEvent::VotesCast { id: 10, number_of_votes: 1 });
	});
}

#[test]
fn cast_more_votes_than_allowed() {
	new_test_ext().execute_with(|| {
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		// Free balance is not reduced by trying to cast too many votes
		assert_eq!(Balances::free_balance(10), 90);
		assert_noop!(Qv::cast_votes(who, 11), BalancesError::<Test>::InsufficientBalance);
		assert_eq!(Balances::free_balance(10), 90);
	});
}

#[test]
fn try_cast_vote_no_identity() {
	new_test_ext().execute_with(|| {
		assert_noop!(Qv::cast_votes(Origin::signed(1), 1), Error::<Test>::NoIdentity);
	});
}

#[test]
fn post_proposal_happy_case() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		let who = Origin::signed(10);
		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		let proposal = BlakeTwo256::hash_of(&1);

		assert_ok!(Qv::post_proposal(who.clone(), 1, proposal));
	});
}
