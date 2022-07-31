#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
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
		traits::{Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;

	use pallet_identity::IdentityField;
	const IDENTITY_FIELD_DISPLAY: u64 = IdentityField::Display as u64;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	/// Uses tight coupling of pallet_identity
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_identity::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: ReservableCurrency<Self::AccountId>;
	}

	// pallet_identity gives function has_identity()
	// and access to the pallet_identity storage map
	// we need to figure out how we want to use the storage map ourselves

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// An amount from the specified accound was reserved
		AmountReserved(T::AccountId, <<T as Config>::Currency as Currency<T::AccountId>>::Balance),
		AmountUnreserved(
			T::AccountId,
			<<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		),
		VotesCast {
			id: T::AccountId,
			number_of_votes: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		},
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
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		/// Reserves an amount of token for a user.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn reserve_an_amount_of_token(
			origin: OriginFor<T>,
			amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				// If funds are too low and Err will be returned
				let res = <T as Config>::Currency::reserve(&who, amount);
				if res == Ok(()) {
					Self::deposit_event(Event::AmountReserved(who, amount));
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
			amount: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			if pallet_identity::Pallet::<T>::has_identity(&who, IDENTITY_FIELD_DISPLAY) {
				// Will return a number:
				// return max(0, amount - balance)
				// but will never fail
				<T as Config>::Currency::unreserve(&who, amount);
				Self::deposit_event(Event::AmountUnreserved(who, amount));
				Ok(())
			} else {
				Err(Error::<T>::NoIdentity.into())
			}
		}

		/// Casts vote on behalf of identified user
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn cast_votes(
			origin: OriginFor<T>,
			number_of_votes: <<T as Config>::Currency as Currency<T::AccountId>>::Balance,
		) -> DispatchResult {
			let tokens_bound =
				Self::reserve_an_amount_of_token(origin.clone(), number_of_votes * number_of_votes);
			if tokens_bound == Ok(()) {
				// Tokens are bound now so we can update the referendum
				// ... however a referendum will end up being represented

				Self::deposit_event(Event::VotesCast {
					id: ensure_signed(origin)?, // There must be a better way to get AccountId out
					number_of_votes,
				});
				Ok(())
			} else {
				tokens_bound
			}
		}
	}
}
