#[macro_export]
macro_rules! map {
    ($( $key:literal = $value:expr ),*$(,)?) => {{
        let mut map = ::std::collections::HashMap::new();
		$(
			map.insert($key.to_string(), $value.into());
		)*
		map
    }};
}
