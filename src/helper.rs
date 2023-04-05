#[macro_export]
macro_rules! map {
    ($( $key:literal = $value:expr ),*$(,)?) => {{
		#[allow(unused_imports)]
		use ::tf_bindgen::value::IntoValue;
        let mut map = ::std::collections::HashMap::<String, Value<_>>::new();
		$(
			map.insert($key.to_string(), $value.into_value());
		)*
		map
    }};
}
