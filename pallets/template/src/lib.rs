#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_balance_of)]
	pub(super) type BalanceOf<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

	#[pallet::storage]
	pub(super) type Allowance<T: Config> =
		StorageMap<_, Blake2_128Concat, (T::AccountId, T::AccountId), u64>;

	#[pallet::storage]
	#[pallet::getter(fn get_total_supply)]
	pub(super) type TotalSupply<T: Config> = StorageValue<_, u64>;

	// Pallets use events to inform users when important changes are made
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		TotalSupply {
			value: u64,
		},
		BalanceOf {
			who: T::AccountId,
			balance: u64,
		},
		BalanceSet {
			who: T::AccountId,
			balance: u64,
		},
		Transfer {
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		},
		Approval {
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		},
		TransferFrom {
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AccountNotExist,
		StorageOverflow,
		InsufficientFunds,
		ApprovalNotGranted,
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn total_supply(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let total_supply = Self::_total_supply();
			Self::deposit_event(Event::TotalSupply { value: total_supply });
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn balance_of(origin: OriginFor<T>, user: T::AccountId) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			let balance = Self::_balance_of(&user);
			Self::deposit_event(Event::BalanceOf { who: user, balance });
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn set_balance(origin: OriginFor<T>, balance: u64) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::_balance_set(&who, balance);
			Self::deposit_event(Event::BalanceSet { who, balance });
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn transfer(origin: OriginFor<T>, to: T::AccountId, value: u64) -> DispatchResult {
			let from = ensure_signed(origin)?;
			Self::_transfer(from, to, value)?;
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn approve(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::_approve(from, to, value)?;
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn transfer_from(
			origin: OriginFor<T>,
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		) -> DispatchResult {
			let _who = ensure_signed(origin)?;
			Self::_transfer_from(from, to, value)?;
			Ok(())
		}
	}
	impl<T: Config> Pallet<T> {
		fn _total_supply() -> u64 {
			<TotalSupply<T>>::get().unwrap_or(0)
		}
		fn _balance_of(who: &T::AccountId) -> u64 {
			<BalanceOf<T>>::get(who).unwrap_or(0)
		}
		fn _balance_set(who: &T::AccountId, balance: u64) {
			<BalanceOf<T>>::insert(who, balance);
		}
		fn _check_if_user_has_balance_or_set_zero(who: &T::AccountId) -> u64 {
			if !<BalanceOf<T>>::contains_key(who) {
				<BalanceOf<T>>::insert(who, 0);
			}
			<BalanceOf<T>>::get(who).unwrap_or(0)
		}
		fn _transfer(from: T::AccountId, to: T::AccountId, value: u64) -> Result<(), Error<T>> {
			ensure!(<BalanceOf<T>>::contains_key(&from), Error::<T>::InsufficientFunds);
			let from_balance = Self::_balance_of(&from);
			let to_balance = Self::_check_if_user_has_balance_or_set_zero(&to);
			ensure!(from_balance >= value, Error::<T>::InsufficientFunds);
			Self::_balance_set(&from, from_balance - value);
			Self::_balance_set(&to, to_balance + value);
			Self::deposit_event(Event::Transfer { from, to, value });
			Ok(())
		}
		fn _approve(from: T::AccountId, to: T::AccountId, value: u64) -> Result<(), Error<T>> {
			ensure!(<BalanceOf<T>>::contains_key(&from), Error::<T>::InsufficientFunds);
			let from_balance = <BalanceOf<T>>::get(&from).unwrap();
			let to_balance = Self::_check_if_user_has_balance_or_set_zero(&to);
			ensure!(from_balance >= value, Error::<T>::InsufficientFunds);
			Self::_balance_set(&from, from_balance - value);
			Self::_balance_set(&to, to_balance + value);
			<Allowance<T>>::insert((&from, &to), value);
			Self::deposit_event(Event::Approval { from, to, value });
			Ok(())
		}

		fn _transfer_from(
			from: T::AccountId,
			to: T::AccountId,
			value: u64,
		) -> Result<(), Error<T>> {
			ensure!(<BalanceOf<T>>::contains_key(&from), Error::<T>::InsufficientFunds);
			let from_balance = Self::_check_if_user_has_balance_or_set_zero(&from);
			let to_balance = Self::_check_if_user_has_balance_or_set_zero(&to);
			ensure!(from_balance >= value, Error::<T>::InsufficientFunds);
			let from_approve = match <Allowance<T>>::get((&from, &to)) {
				Some(approve) => approve,
				None => Err(Error::<T>::ApprovalNotGranted)?,
			};
			ensure!(from_approve >= value, Error::<T>::ApprovalNotGranted);
			Self::_balance_set(&from, from_balance - value);
			Self::_balance_set(&to, to_balance + value);
			<Allowance<T>>::insert((&from, &to), from_approve - value);
			Self::deposit_event(Event::TransferFrom { from, to, value });
			Ok(())
		}
	}
}
