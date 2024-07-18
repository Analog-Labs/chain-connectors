pub trait FnMut1<A> {
    type Output;
    fn call_mut(&mut self, arg: A) -> Self::Output;
}

impl<T, A, R> FnMut1<A> for T
where
    T: FnMut(A) -> R,
{
    type Output = R;
    fn call_mut(&mut self, arg: A) -> R {
        self(arg)
    }
}
