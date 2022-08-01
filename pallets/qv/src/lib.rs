//! # Quadratic Voting Pallet ( pallet-qv )
//!
//! > NOTE: This pallet is tightly coupled with pallet-identity and pallet-referenda.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(
	Clone,
	Encode,
	Decode,
	MaxEncodedLen,
	Eq,
	PartialEq,
	sp_runtime::RuntimeDebug,
	Default,
	scale_info::TypeInfo,
)]
pub struct Proposal<Balance, Hash> {
	backing: Balance,
	statement: Hash,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	// From pallet_identity we use functions has_identity() and set_identity()
	use pallet_identity::IdentityField;
	const IDENTITY_FIELD_DISPLAY: u64 = IdentityField::Display as u64;

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

	/// The public proposals. Unsorted. The second item is the proposal's hash.
	/// TODO: do we need a proposal index?
	///       I don't think we need the account id.
	///       do we want to store referendums in their entirety?
	#[pallet::storage]
	#[pallet::getter(fn public_props)]
	pub type PublicProps<T: Config> =
		StorageValue<_, crate::Proposal<BalanceOf<T>, T::Hash>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An amount from the specified accound was reserved
		/// Exposing account id here would go against voting anonymity.
		AmountReserved(BalanceOf<T>),

		/// An amount from the specified accound was unreserved
		AmountUnreserved(T::AccountId, BalanceOf<T>),

		/// TODO: Exposing account id here goes against voting anonymity.
		VotesCast { id: T::AccountId, number_of_votes: BalanceOf<T> },
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
		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			ensure_signed(origin)?;

			// Read a value from storage.
			match <PublicProps<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(_old) => {
					// Increment the value read from storage; will error in the event of overflow.
					// TODO: new thing
					//let new = old.ok_or(Error::<T>::StorageOverflow)?;
					//// Update the value in storage with the incremented result.
					//<PublicProps<T>>::put(new);
					Ok(())
				},
			}
		}

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
				Self::deposit_event(Event::AmountUnreserved(who, amount));
				Ok(())
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn post_proposal(
			origin: OriginFor<T>,
			number_of_votes: BalanceOf<T>,
			statement: T::Hash,
		) -> DispatchResult {
			Self::reserve_an_amount_of_token(origin, number_of_votes * number_of_votes)?;
			PublicProps::<T>::put(crate::Proposal { backing: number_of_votes, statement });
			Ok(())
		}

		/// Casts vote on behalf of identified user
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cast_votes(origin: OriginFor<T>, number_of_votes: BalanceOf<T>) -> DispatchResult {
			let tokens_bound =
				Self::reserve_an_amount_of_token(origin.clone(), number_of_votes * number_of_votes);
			if tokens_bound == Ok(()) {
				// Tokens are bound now so we can update the referendum
				// ... however a referendum will end up being represented

				// Update storage.
				//<PublicProps<T>>::put(number_of_votes);
				//<PublicProps<T>>::put(BlakeTwo256::hash_of(&1));
				Self::deposit_event(Event::VotesCast {
					id: ensure_signed(origin)?, // There must be a better way to get AccountId out
					number_of_votes,
				});
				Ok(())
			} else {
				tokens_bound
			}
		}

		//pub fn check_proposal_exists() -> Option<T::Hash> {}
	}
}
