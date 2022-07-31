use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

use pallet_balances::Error as BalancesError;
use pallet_identity::{Data, IdentityInfo};
use sp_runtime::DispatchError;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(Qv::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(Qv::something(), Some(42));
	});
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
		let who = Origin::signed(10);

		assert_ok!(Identity::set_identity(who.clone(), Box::new(info())));

		assert_ok!(Qv::reserve_an_amount_of_token(who.clone(), SMALL_AMOUNT));
	});
}
