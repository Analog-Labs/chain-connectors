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

pub trait Fn1<A>: FnMut1<A> {
    fn call(&self, arg: A) -> Self::Output;
}

impl<T, A, R> Fn1<A> for T
where
    T: Fn(A) -> R,
{
    fn call(&self, arg: A) -> R {
        self(arg)
    }
}
