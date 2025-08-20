#[macro_export]
macro_rules! generate_enum_with_type_id {
    (
        $type:ty,
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($body)*
        }

        $crate::generate_enum_with_type_id!(@collect_arms $name, $type, [], $($body)*);
    };

    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($body)*
        }

        $crate::generate_enum_with_type_id!(@find_repr_in_meta
            { name: $name, body: { $($body)* } },
            $(#[$meta])*
        );
    };

    (@find_repr_in_meta { name: $name:ident, body: { $($body:tt)* } }, #[repr($type:ty)] $($rest:meta)*) => {
        $crate::generate_enum_with_type_id!(@collect_arms $name, $type, [], $($body)*);
    };
    (@find_repr_in_meta { name: $name:ident, body: { $($body:tt)* } }, #[$_other:meta] $($rest:meta)*) => {
        $crate::generate_enum_with_type_id!(@find_repr_in_meta { name: $name, body: { $($body)* } }, $($rest)*);
    };
    (@find_repr_in_meta { name: $name:ident, body: { $($body:tt)* } }, ) => {
        $crate::generate_enum_with_type_id!(@collect_arms $name, u8, [], $($body)*);
    };

    (@build_impl $name:ident, $type:ty, [ $( { $($arm:tt)* } )* ]) => {
        impl $name {
            pub fn type_id(&self) -> $type {
                match self {
                    $( $($arm)* )*
                }
            }
        }
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], ) => {
        $crate::generate_enum_with_type_id!(@build_impl $name, $type, [ $($acc)* ]);
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident ($($fields:tt)+) = $value:literal, $($rest:tt)*) => {
        $crate::generate_enum_with_type_id!(@collect_arms $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant(..) => $value as $type, } ], $($rest)*);
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident ($($fields:tt)+) = $value:literal) => {
        $crate::generate_enum_with_type_id!(@build_impl $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant(..) => $value as $type, } ]);
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident { $($fields:tt)+ } = $value:literal, $($rest:tt)*) => {
        $crate::generate_enum_with_type_id!(@collect_arms $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant{..} => $value as $type, } ], $($rest)*);
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident { $($fields:tt)+ } = $value:literal) => {
        $crate::generate_enum_with_type_id!(@build_impl $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant{..} => $value as $type, } ]);
    };

    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident = $value:literal, $($rest:tt)*) => {
        $crate::generate_enum_with_type_id!(@collect_arms $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant => $value as $type, } ], $($rest)*);
    };
    
    (@collect_arms $name:ident, $type:ty, [ $($acc:tt)* ], $(#[$meta:meta])* $variant:ident = $value:literal) => {
        $crate::generate_enum_with_type_id!(@build_impl $name, $type, [ $($acc)* { $(#[$meta])* Self::$variant => $value as $type, } ]);
    };
}