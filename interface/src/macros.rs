macro_rules! try_from_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident : $type:ty { $($variant:ident,)* }) => {
        $(#[$meta])*
        #[repr($type)]
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        $vis enum $name {
            $($variant,)*
        }

        impl TryFrom<$type> for $name {
            type Error = ();

            fn try_from(value: $type) -> Result<Self, Self::Error> {
                match value {
                    $(x if x == $name::$variant as $type => Ok($name::$variant),)*
                    _ => Err(()),
                }
            }
        }
    };
}
