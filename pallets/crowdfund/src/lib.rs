#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod tipos;

use frame_support::traits::{Currency, Get};
use tipos::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, Blake2_128Concat};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type LargoMinimoNombreProyecto: Get<u32>;

		#[pallet::constant]
		type LargoMaximoNombreProyecto: Get<u32>;

		type Currency: Currency<Self::AccountId>; // Pueden no utilizarlo.
	}

	#[pallet::storage]
	pub type Proyectos<T> =
		StorageMap<_, Blake2_128Concat, BoundedString<T>, BalanceDe<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ProyectoCreado { quien: T::AccountId, nombre: NombreProyecto<T> },
		ProyectoApoyado { nombre: NombreProyecto<T>, cantidad: BalanceDe<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		NombreMuyLargo,
		NombreMuyCorto,
		/// El usuario quiso apoyar un proyecto con más fondos de los que dispone.
		FondosInsuficientes,
		/// El usuario quiso apoyar un proyecto inexistente.
		ProyectoNoExiste,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Crea un proyecto.
		pub fn crear_proyecto(origen: OriginFor<T>, nombre: String) -> DispatchResult {
			let who = ensure_signed(origen)?;

			// Convertir el nombre de bytes a un tipo `NombreProyecto<T>` acotado.
			let acotado_result: Result<NombreProyecto<T>, _> =
				NombreProyecto::<T>::try_from(nombre.clone());
			let acotado_result = match acotado_result {
				Ok(v) => v,
				Err(_) => return Err(Error::<T>::NombreMuyLargo.into()),
			};

			// Obtener la longitud del nombre acotado.
			let longitud = acotado_result.len() as u32;
			ensure!(longitud >= T::LargoMinimoNombreProyecto::get(), Error::<T>::NombreMuyCorto);
			ensure!(longitud <= T::LargoMaximoNombreProyecto::get(), Error::<T>::NombreMuyLargo);

			// Crear un proyecto con un balance inicial de cero.
			let balance_cero = BalanceDe::<T>::from(0u32);
			Proyectos::<T>::insert(acotado_result, balance_cero);

			Ok(())
		}

		pub fn apoyar_proyecto(
			origen: OriginFor<T>,
			nombre: String,
			cantidad: BalanceDe<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origen)?;

			let bounded_name: NombreProyecto<T> = nombre.clone().try_into().unwrap();

			ensure!(Proyectos::<T>::contains_key(&bounded_name), Error::<T>::ProyectoNoExiste);

			let current_balance = T::Currency::free_balance(&sender);
			ensure!(current_balance >= cantidad, Error::<T>::FondosInsuficientes);

			let project_balance = Proyectos::<T>::get(&bounded_name);
			let new_project_balance = project_balance + cantidad;
			Proyectos::<T>::insert(&bounded_name, new_project_balance);

			T::Currency::withdraw(
				&sender,
				cantidad,
				frame_support::traits::WithdrawReasons::TRANSFER,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)?;

			Ok(())
		}
}

}
