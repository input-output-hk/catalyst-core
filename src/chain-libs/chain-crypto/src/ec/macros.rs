//! Internal macros

macro_rules! std_ops_gen {
    ($lty: ident, $class: ident, $rty: ident, $out: ident, $f: ident) => {
        impl<'a> $class<$rty> for &'a $lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                self.$f(&other)
            }
        }

        impl<'b> $class<&'b $rty> for $lty {
            type Output = $out;

            fn $f(self, other: &'b $rty) -> Self::Output {
                (&self).$f(other)
            }
        }

        impl $class<$rty> for $lty {
            type Output = $out;

            fn $f(self, other: $rty) -> Self::Output {
                (&self).$f(&other)
            }
        }
    };
}
