macro_rules! jcall {
    ($ctx:expr, $func_name:ident) => {
        unsafe {
            (*(*($ctx))).$func_name.expect(concat!(stringify!($func_name), " unavailable"))($ctx)
        }
    };
    ($ctx:expr, $func_name:ident, $($args:expr),*) => {
        unsafe {
            (*(*($ctx))).$func_name.expect(concat!(stringify!($func_name), " unavailable"))($ctx, $($args),*)
        }
    };
}

pub(crate) use jcall;
