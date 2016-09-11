/// Defines a type built out of other types.
macro_rules! define_composite_type {
    ($ty:ident { $( $field_name:ident : $field_ty:ty ),+ }) => {
        #[derive(Clone,Debug)]
        pub struct $ty
        {
            $( $field_name : $field_ty ),+
        }

        impl ::protocol::Type for $ty
        {
            fn read(read: &mut ::std::io::Read) -> Result<Self, ::protocol::Error> {
                Ok($ty {
                    $( $field_name: <$field_ty as ::protocol::Type>::read(read)? ),+
                })
            }

            fn write(&self, write: &mut ::std::io::Write) -> Result<(), ::protocol::Error> {
                $( self.$field_name.write(write)?; )+

                Ok(())
            }
        }
    }
}

