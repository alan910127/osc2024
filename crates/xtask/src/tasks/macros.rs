#[macro_export]
macro_rules! envs {
    ($($key:expr => $value:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut map = ::std::collections::HashMap::new();
        $(map.insert($key.into(), $value.into());)*
        map
    }};
}
