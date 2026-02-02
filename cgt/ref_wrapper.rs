macro_rules! impl_ref_wrapper {
    ( $(#[$attr:meta])*
      $struct_vis:vis struct $wrapper:ident {
          $(#[$field_attr:meta])*
              $field_vis:vis $field:ident: $inner:ty $(,)?
      }
    ) => {
        $(#[$attr])*
            #[repr(transparent)]
        $struct_vis struct $wrapper {
            $(#[$field_attr])*
                $field: $inner,
        }

        impl $wrapper {
            /// Wrap a reference to the underlying type
            #[allow(dead_code)]
            #[inline(always)]
            $struct_vis fn from_inner(inner: &$inner) -> &$wrapper {
                // SAFETY: repr(transparent)
                unsafe { &*(::std::ptr::from_ref(inner) as *const $wrapper) }
            }

            /// Wrap a mutable reference to the underlying type
            #[allow(dead_code)]
            #[inline(always)]
            $struct_vis fn from_inner_mut(inner: &mut $inner) -> &mut $wrapper {
                // SAFETY: repr(transparent)
                unsafe { &mut *(::std::ptr::from_mut(inner) as *mut $wrapper) }
            }

            /// Unwrap a reference into the underlying type
            #[allow(dead_code)]
            #[inline(always)]
            $struct_vis fn to_inner(&self) -> &$inner {
                // SAFETY: repr(transparent)
                unsafe { &*(::std::ptr::from_ref(self) as *const $inner) }
            }

            /// Unwrap a mutable reference into the underlying type
            #[allow(dead_code)]
            #[inline(always)]
            $struct_vis fn to_inner_mut(&mut self) -> &mut $inner {
                // SAFETY: repr(transparent)
                unsafe { &mut *(::std::ptr::from_mut(self) as *mut $inner) }
            }
        }
    }
}

pub(crate) use impl_ref_wrapper;
