#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod tests;
pub mod types;

use frame_support::ensure;
use types::*;

#[allow(unexpected_cfgs)] // skip warning "unexpected `cfg` condition value: `try-runtime`"
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config + scale_info::TypeInfo {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxLength: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn unique_asset)]
	pub(super) type UniqueAsset<T: Config> =
		StorageMap<_, Blake2_128Concat, UniqueAssetId, UniqueAssetDetails<T, T::MaxLength>>;

	#[pallet::storage]
	#[pallet::getter(fn account)]
	/// The holdings of a specific account for a specific asset.
	pub(super) type Account<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		UniqueAssetId,
		Blake2_128Concat,
		T::AccountId,
		u128,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn nonce)]
	/// Nonce for id of the next created asset
	pub(super) type Nonce<T: Config> = StorageValue<_, UniqueAssetId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New unique asset created
		Created {
			creator: T::AccountId,
			asset_id: UniqueAssetId,
		},
		/// Some assets have been burned
		Burned {
			asset_id: UniqueAssetId,
			owner: T::AccountId,
			total_supply: u128,
		},
		/// Some assets have been transferred
		Transferred {
			asset_id: UniqueAssetId,
			from: T::AccountId,
			to: T::AccountId,
			amount: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The asset ID is unknown
		UnknownAssetId,
		/// The signing account does not own any amount of this asset
		NotOwned,
		/// Supply must be positive
		NoSupply,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight({0})]
		pub fn mint(
			origin: OriginFor<T>,
			metadata: BoundedVec<u8, T::MaxLength>,
			supply: u128,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			if supply == 0 {
				return Err(Error::<T>::NoSupply.into());
			}

			let id = Self::nonce();
			let details = UniqueAssetDetails::new(origin.clone(), metadata, supply);

			UniqueAsset::<T>::insert(id, details);
			Nonce::<T>::set(id.saturating_add(1));

			Account::<T>::insert(id, origin.clone(), supply);

			Self::deposit_event(Event::<T>::Created {
				creator: origin,
				asset_id: id,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({0})]
		pub fn burn(origin: OriginFor<T>, asset_id: UniqueAssetId, amount: u128) -> DispatchResult {
			let origin = ensure_signed(origin)?;
			let details = Self::unique_asset(asset_id).ok_or(Error::<T>::UnknownAssetId)?;
			ensure!(details.creator() == origin, Error::<T>::NotOwned);

			let balance = Account::<T>::get(asset_id, origin.clone());
			let amount = amount.min(balance);

			let total_supply = details.supply.saturating_sub(amount);
			UniqueAsset::<T>::mutate(asset_id, |details| {
				let details = details.as_mut().unwrap();
				details.supply = total_supply;
			});

			Account::<T>::mutate(asset_id, origin.clone(), |balance| {
				*balance = balance.saturating_sub(amount);
			});

			Self::deposit_event(Event::<T>::Burned {
				asset_id,
				owner: origin,
				total_supply,
			});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({0})]
		pub fn transfer(
			origin: OriginFor<T>,
			asset_id: UniqueAssetId,
			amount: u128,
			to: T::AccountId,
		) -> DispatchResult {
			let origin = ensure_signed(origin)?;

			let exists = UniqueAsset::<T>::contains_key(asset_id);
			if !exists {
				return Err(Error::<T>::UnknownAssetId.into());
			}

			let balance = Account::<T>::get(asset_id, origin.clone());
			if balance == 0 {
				return Err(Error::<T>::NotOwned.into());
			}

			let amount = amount.min(balance);

			Account::<T>::mutate(asset_id, origin.clone(), |balance| {
				*balance = balance.saturating_sub(amount);
			});

			Account::<T>::mutate(asset_id, to.clone(), |balance| {
				*balance = balance.saturating_add(amount);
			});

			Self::deposit_event(Event::<T>::Transferred {
				asset_id,
				from: origin,
				to,
				amount,
			});

			Ok(())
		}
	}
}
