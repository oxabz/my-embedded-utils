#![no_std]

/// Trait to simply packing and unpacking simple types into buffers
pub trait Pack:Sized {
    /// The length of the item into once serialize
    const SIZE: usize;

    /// Pack self into a byte buffer
    /// buf need the capacity to accept self 
    fn pack(&self, buf: &mut [u8]);

    /// Read the self from the buffer
    fn unpack(buf: &[u8]) -> Self;
    
    /// Read the self from the buffer and advance the read pointer
    fn unpack_it(buf: &mut &[u8]) -> Option<Self>
    {
        if buf.len() < Self::SIZE {
            return None;
        }

        let val = Self::unpack(*buf);

        *buf = &buf[Self::SIZE..];

        Some(val)
    }
    
    /// Read the self from the buffer and advance the read pointer. But doesnt check the remaining length of the buffer
    fn unpack_it_unchecked(buf: &mut &[u8]) -> Self {
        let val = Self::unpack(buf);

        *buf = &buf[Self::SIZE..];

        val
    }
}

impl Pack for u8 {
    const SIZE: usize = 1;

    fn pack(&self, buf: &mut [u8]) {
        buf[0] = *self;
    }

    fn unpack(buf: &[u8]) -> Self {
        buf[0]
    }

}

impl Pack for i8 {
    const SIZE: usize = 1;

    fn pack(&self, buf: &mut [u8]) {
        buf[0] = *self as u8;
    }

    fn unpack(buf: &[u8]) -> Self {
        buf[0] as i8
    }

}

impl Pack for bool {
    const SIZE: usize = 1;

    fn pack(&self, buf: &mut [u8]) {
        buf[0] = if *self {1} else {0}; 
    }

    fn unpack(buf: &[u8]) -> Self {
        buf[0] != 0
    }
}


macro_rules! pack_uint_impl {
    ($typ: path, $size: expr) => {
        impl Pack for $typ {
            const SIZE: usize = $size;

            fn pack(&self, buf: &mut [u8]) {
                let mut this = *self;
                
                for i in 1..=Self::SIZE {
                    buf[Self::SIZE - i] = (this & 0xFF) as u8;
                    this = this >> 8;
                }
            }
        
            fn unpack(buf: &[u8]) -> Self {
                let mut this = 0;

                for i in 0..Self::SIZE {
                    this = this << 8;
                    this |= (buf[i] as $typ);
                }

                this
            }
        }
    };
}

macro_rules! pack_iint_impl {
    ($utyp: path, $typ: path, $size: expr) => {
        impl Pack for $typ {
            const SIZE: usize = $size;

            fn pack(&self, buf: &mut [u8]) {
                let mut this = (*self) as $utyp;
                
                for i in 1..=Self::SIZE {
                    buf[Self::SIZE - i] = (this & 0xFF) as u8;
                    this = this >> 8;
                }
            }
        
            fn unpack(buf: &[u8]) -> Self {
                let mut this: $utyp = 0;

                for i in 0..Self::SIZE {
                    this = this << 8;
                    this |= (buf[i] as $utyp);
                }

                this as $typ
            }
        }
    };
}

pack_uint_impl!(u16, 2);
pack_uint_impl!(u32, 4);
pack_uint_impl!(u64, 8);
pack_uint_impl!(u128, 16);
pack_iint_impl!(u16, i16, 2);
pack_iint_impl!(u32, i32, 4);
pack_iint_impl!(u64, i64, 8);
pack_iint_impl!(u128, i128, 16);

macro_rules! pack_tuple_impl {
    ($($idents: ident),+) => {
        #[allow(non_snake_case)]
        impl<
        $($idents: Pack),+
        > Pack for ($($idents,)+) {
            const SIZE: usize = $($idents::SIZE +)+ 0;

            #[allow(unused_assignments)]
            fn pack(&self, buf: &mut [u8]) {
                let (
                    $($idents,)+
                ) = self;

                let mut i = 0;
                $(
                    $idents.pack(&mut buf[i..]);
                    i += $idents::SIZE;
                )+
            }
        
            fn unpack(buf: &[u8]) -> Self {
                let mut buf = buf;
                $(
                    let $idents = $idents::unpack_it_unchecked(&mut buf);
                )+

                ($($idents,)+)
            }
        }
    };
}

pack_tuple_impl!{A}
pack_tuple_impl!{A, B}
pack_tuple_impl!{A, B, C}
pack_tuple_impl!{A, B, C, D}
pack_tuple_impl!{A, B, C, D, E}
pack_tuple_impl!{A, B, C, D, E, F}
pack_tuple_impl!{A, B, C, D, E, F, G}

pub mod special{
    use core::ops::Deref;

    pub struct I48(i64);

    impl I48 {
        pub fn into_inner(self)->i64{
            self.0
        }
        
    }

    impl From<i64> for I48 {
        fn from(value: i64) -> Self {
            Self(value)
        }
    }

    impl Deref for I48 {
        type Target = i64;
    
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl AsRef<i64> for I48 {
        fn as_ref(&self) -> &i64 {
            &self.0
        }
    }

    impl super::Pack for I48 {
        const SIZE: usize = 6;
    
        fn pack(&self, buf: &mut [u8]) {
            
            let mut this: i64 = self.0;
                
            for i in 1..=Self::SIZE {
                buf[Self::SIZE - i] = (this & 0xFF) as u8;
                this = this >> 8;
            }
        }
    
        fn unpack(buf: &[u8]) -> Self {
            let mut this = 0i64;

            for i in 0..Self::SIZE {
                this |= buf[i] as i64;
                this = this << 8;
            }

            // Extending two's complement            
            if this & (1 << 47) != 0 {
                this |= 0xFFFFi64 << 48;
            }

            Self(this)
        }
    }

}