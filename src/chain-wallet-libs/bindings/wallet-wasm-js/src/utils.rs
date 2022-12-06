use wasm_bindgen::prelude::*;

/// `set_panic_hook` function can be called at least once during initialization,
/// to get better error messages if the code ever panics.
/// The function has no parameters.
#[wasm_bindgen]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// taken from:
// https://github.com/input-output-hk/js-chain-libs/blob/cc463b59fdc64a4fff63f67901118f60b783520c/src/utils.rs#L12
#[macro_export]
macro_rules! impl_collection {
    ($collection:ident, $type:ty) => {
        #[wasm_bindgen]
        pub struct $collection(Vec<$type>);

        #[allow(clippy::new_without_default)]
        #[wasm_bindgen]
        impl $collection {
            pub fn new() -> $collection {
                Self(vec![])
            }

            pub fn size(&self) -> usize {
                self.0.len()
            }

            pub fn get(&self, index: usize) -> $type {
                self.0[index].clone()
            }

            pub fn add(&mut self, item: $type) {
                self.0.push(item);
            }
        }

        impl From<Vec<$type>> for $collection {
            fn from(vec: Vec<$type>) -> $collection {
                $collection(vec)
            }
        }
    };
}
