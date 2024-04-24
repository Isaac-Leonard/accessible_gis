use num_traits::Num;

pub trait Mean<A = Self>: Sized {
    fn mean<I: Iterator<Item = A>>(iter: I) -> Self;
}

impl<'a, T: 'a + Num + Copy + From<usize>> Mean<&'a T> for T {
    fn mean<I>(iter: I) -> Self
    where
        I: Iterator<Item = &'a T>,
    {
        let (sum, len) = iter.fold((T::zero(), 0_usize), |(sum, len), c| (sum + *c, len + 1));
        sum / T::from(len)
    }
}

pub trait IteratorMyExt: Iterator {
    fn mean<S>(self) -> S
    where
        Self: Sized,
        S: Mean<Self::Item>,
    {
        Mean::mean(self)
    }
}

impl<T: Iterator> IteratorMyExt for T {}
