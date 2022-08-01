//! # Quadratic Voting Pallet ( pallet-qv )
//!
//! > NOTE: This pallet is tightly coupled with pallet-identity and pallet-referenda.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{schedule::DispatchTime, Currency, ReservableCurrency},
	};
	use frame_system::{pallet_prelude::*, RawOrigin};

	// From pallet_identity we use functions has_identity() and set_identity()
	use pallet_identity::IdentityField;
	const IDENTITY_FIELD_DISPLAY: u64 = IdentityField::Display as u64;

	use pallet_referenda::ReferendumIndex;

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// Uses tight coupling of pallet_identity and pallet_referenda
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_identity::Config + pallet_referenda::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An amount from the specified accound was reserved
		/// Exposing account id here would go against voting anonymity.
		AmountReserved(BalanceOf<T>),

		/// An amount from the specified accound was unreserved
		AmountUnreserved(BalanceOf<T>),

		/// Somebody voted to launch a referendum
		LaunchVotesCast { number_of_votes: BalanceOf<T>, index: ReferendumIndex },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// User has not set an identity
		NoIdentity,
		/// The proposal already exists
		DuplicateProposal,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Reserves an amount of token for a user.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reserve_an_amount_of_token(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				// If funds are too low and Err will be returned
				let res = <T as Config>::Currency::reserve(&who, amount);
				if res == Ok(()) {
					Self::deposit_event(Event::AmountReserved(amount));
				}
				res
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		/// Unreserves an amount of token for a user.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn unreserve_an_amount_of_token(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				<T as Config>::Currency::unreserve(&who, amount);
				Self::deposit_event(Event::AmountUnreserved(amount));
				Ok(())
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn initiate_referendum(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			const REFERENDUM_BLOCKS_TOTAL: u32 = 892800; // 2*31*24*60*60/6 = "Two months" / "block time"
			let now = <frame_system::Pallet<T>>::block_number();
			<pallet_referenda::Pallet<T>>::submit(
				origin,
				Box::new(RawOrigin::Root.into()),
				proposal,
				DispatchTime::At(now + REFERENDUM_BLOCKS_TOTAL.into()),
			)
		}

		/// Casts vote on behalf of identified user
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cast_launch_votes(
			origin: OriginFor<T>,
			number_of_votes: BalanceOf<T>,
			index: ReferendumIndex,
		) -> DispatchResultWithPostInfo {
			// TODO: if the referendum exists.. {
			Self::reserve_an_amount_of_token(origin.clone(), number_of_votes * number_of_votes)?;
			Self::deposit_event(Event::LaunchVotesCast { number_of_votes, index });

			// TODO: If enough launch votes have been cast:
			if false {
				return <pallet_referenda::Pallet<T>>::place_decision_deposit(
					origin, /* TODO: We need a special origin that represents everyone, to
					         * "split the bill" for us */
					index,
				)
			}
			Ok(().into())
			//}
		}

		//pub fn check_proposal_exists() -> Option<T::Hash> {}
	}
}
