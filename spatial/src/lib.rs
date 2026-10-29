#![no_std]

use embassy_executor::Spawner;

pub mod encoder;
pub mod magnetometer;
pub mod orientation;
pub mod usb;
pub mod status;

pub trait ErrorSink {}

pub trait StaticStorage {
    const DEFAULT: Self;
}

impl StaticStorage for () {
    const DEFAULT: Self = ();
}

pub trait ModuleBuilder {
    type Target: Module;
    fn build(
        self,
        storage: &'static mut <Self::Target as Module>::Storage,
    ) -> Self::Target;
}

/// Represents a vertical slice which is responsible for
/// the whole range of operation related to the same concept:
/// fetching the data from sensors, processing it, reporting errors,
/// sending the data over USB, etc.
///
/// The modules are singletons stored in static memory. This allows
/// to use tasks instead of futures.
pub trait Module: Sized {
    /// The type which is used to store module's static data
    type Storage: StaticStorage;

    /// Runs the module on given [`Spawner`], which is used to spawn module's
    /// tasks.
    fn run_on(self, spawner: &Spawner) -> impl core::future::Future<Output = ()>;
}

#[macro_export]
macro_rules! init_module {
    ($mod:ty : $config:expr) => {{
        static mut STORAGE: <$mod as Module>::Storage =
            <<$mod as Module>::Storage as $crate::StaticStorage>::DEFAULT;
        $config.build(unsafe {(&raw mut STORAGE).as_mut_unchecked()})
    }};
    ($($mod:ty : $config:expr,)*) => {{
        ($(init_module!($mod : $config), )*)
    }}
}

pub trait ModuleCollection {
    fn run(self, spawner: &Spawner) -> impl Future<Output = ()>;
}

macro_rules! impl_module_collection_for_tuple {
    {$($name:ident)+} => {
        #[allow(non_snake_case)]
        impl<$($name,)+> ModuleCollection for ($($name,)+)
        where $($name: Module + 'static,)+ {
            async fn run(self, spawner: &Spawner) {
                // TODO: join
                let ($($name,)+) = self;
                $($name.run_on(spawner);)*
            }
        }
    };
}

impl_module_collection_for_tuple! {A}
impl_module_collection_for_tuple! {A B}
impl_module_collection_for_tuple! {A B C}
impl_module_collection_for_tuple! {A B C D}
impl_module_collection_for_tuple! {A B C D E}
impl_module_collection_for_tuple! {A B C D E F}
