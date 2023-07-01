#[macro_export]
macro_rules! key_value_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($variant:ident { text: $text:expr, key: $key:expr }),* $(,)? }) => {
        $(#[$meta])*
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub fn key(&self) -> &str {
                match self {
                    $(Self::$variant => $key),*
                }
            }

            pub fn text(&self) -> &str {
                match self {
                    $(Self::$variant => $text),*
                }
            }

            pub fn from_key(key: &str) -> Result<Self, &'static str> {
                match key {
                    $($key => Ok(Self::$variant),)*
                    _ => Err("Invalid key"),
                }
            }

            pub fn from_option_key(key: Option<String>) -> Result<Self, &'static str> {
                match key {
                    Some(key) => Self::from_key(&key),
                    None => Err("Invalid key"),
                }
            }

            #[allow(dead_code)]
            pub fn from_text(text: &str) -> Result<Self, &'static str> {
                match text {
                    $($text => Ok(Self::$variant),)*
                    _ => Err("Invalid text"),
                }
            }
        }


    };
}
