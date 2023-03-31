#[macro_export]
macro_rules! map {
    ($( $key:literal = $value:expr ),*$(,)?) => {{
        let mut map = HashMap::new();
		$(
			map.insert($key.to_string(), $value.into());
		)*
		map
    }};
}
