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