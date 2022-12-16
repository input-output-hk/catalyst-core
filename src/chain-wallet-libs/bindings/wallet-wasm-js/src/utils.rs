extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// taken from:
// https://rustwasm.github.io/book/game-of-life/debugging.html
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
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
