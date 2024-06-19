// If this code breaks, the foreign exchange market will take a twenty-eight percent hit - people will die

macro_rules! newtype_aliases {
    ($($a: ident = [$($adv: ident),+]);+;) => {
        #[macro_export]
        macro_rules! newtype_alias {
            $(
                ([$a] $$($$x:tt)*) => {newtype!(final [$($adv),+] $$($$x)*);};
                ([$a, $$($$y:ident),+] $$($$x:tt)*) => {newtype_alias!([$$($$y),+, $($adv),+] $$($$x)*);};
            )+
            ([$$($$dv: ident),*] $$($$x:tt)*) => {newtype!(final [$$($$dv),+] $$($$x)*);};
        }
    };
}

newtype_aliases! {
    c = [Clone];
    d = [Debug];
    dd = [Debug, Default];
    cc = [Clone, Copy];
    e = [PartialEq];
    ee = [PartialEq, Eq];
    o = [PartialOrd];
    oo = [PartialOrd, Ord];
    h = [Hash];
    all = [Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash];
}

#[macro_export]
macro_rules! newtype {
    ($i: ident = $t: ty) => {newtype!([] $i = $t);};
    ($i: ident = $t: ty: $c: literal) => {newtype!([] $i = $t: $c);};
    ($v: vis $i: ident = $t: ty) => {newtype!([] $v $i = $t);};
    ($v: vis $i: ident = $t: ty: $c: literal) => {newtype!([] $v $i = $t: $c);};
    ([$($dv: ident),*] $i: ident = $t: ty) => {newtype!([$($dv),*] $i = $t: "");};
    ([$($dv: ident),*] $i: ident = $t: ty: $c: literal) => {newtype!([$($dv),*] pub(self) $i = $t: $c);};
    ([$($dv: ident),*] $v: vis $i: ident = $t: ty) => {newtype!([$($dv),*] $v $i = $t: "");};
    ([$($dv: ident),*] $v: vis $i: ident = $t: ty: $c: literal) => {newtype_alias!([$($dv),*] $v $i = $t: $c);};
    (final [$($dv: ident),*] $v: vis $i: ident = $t: ty: $c: literal) => {
        #[allow(clippy::empty_docs)]
        #[allow(clippy::needless_pub_self)]
        #[doc = $c]
        #[derive($($dv),*)]
        $v struct $i(pub $t);

        impl From<$t> for $i {
            fn from(value: $t) -> Self {
                Self(value)
            }
        }

        impl std::ops::Deref for $i {
            type Target = $t;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $i {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<T> AsRef<T> for $i
        where
            T: ?Sized,
            <$i as std::ops::Deref>::Target: AsRef<T>,
        {
            fn as_ref(&self) -> &T {
                std::ops::Deref::deref(self).as_ref()
            }
        }

        impl<T> AsMut<T> for $i
        where
            T: ?Sized,
            <$i as std::ops::Deref>::Target: AsMut<T>,
        {
            fn as_mut(&mut self) -> &mut T {
                std::ops::DerefMut::deref_mut(self).as_mut()
            }
        }
    };
}
