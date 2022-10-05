macro_rules! non_null {
    ( $obj:expr ) => {
        if let Some(obj) = $obj.as_ref() {
            obj
        } else {
            return Error::invalid_input(stringify!($expr))
                .with(crate::c::NulPtr)
                .into();
        }
    };
}

macro_rules! non_null_mut {
    ( $obj:expr ) => {
        if let Some(obj) = $obj.as_mut() {
            obj
        } else {
            return Error::invalid_input(stringify!($expr))
                .with(crate::c::NulPtr)
                .into();
        }
    };
}

macro_rules! non_null_array {
    ( $obj:expr, $len:expr) => {
        if $obj.is_null() {
            return Error::invalid_input(stringify!($expr))
                .with(crate::c::NulPtr)
                .into();
        } else {
            std::slice::from_raw_parts($obj, $len)
        }
    };
}
