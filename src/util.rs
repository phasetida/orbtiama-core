#[macro_export]
macro_rules! function_test {
    ($name:ident, $function: ident, $($args:tt)*) => {
        #[test]
        fn $name() {
            expect_file![format!("./test/{}.txt", stringify!($name))]
                .assert_debug_eq(&$function($($args)*));
        }
    };
}

#[macro_export]
macro_rules! function_test_display {
    ($name:ident, $function: ident, $($args:tt)*) => {
        #[test]
        fn $name() {
            expect_file![format!("./test/{}.txt", stringify!($name))]
                .assert_eq(&$function($($args)*));
        }
    };
}

#[macro_export]
macro_rules! function_test_svg {
    ($name:ident, $function: ident, $($args:tt)*) => {
        #[test]
        fn $name() {
            expect_file![format!("./test/{}.svg", stringify!($name))]
                .assert_eq(&$function($($args)*));
        }
    };
}

#[macro_export]
macro_rules! quick_deref {
    ($host:ident, $target:ident, $name:ident) => {
        impl Deref for $host {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                &self.$name
            }
        }
    };
}
pub trait Dense {
    fn to_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<Self>(self).cast::<u8>(),
                std::mem::size_of::<Self>(),
            )
        }
    }
}