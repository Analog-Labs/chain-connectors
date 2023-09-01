#![allow(dead_code)]
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use jsonrpsee::core::params::BatchRequestBuilder;
use jsonrpsee::core::{
    client::{BatchResponse, ClientT, Subscription, SubscriptionClientT},
    traits::ToRpcParams,
    Error,
};
use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;

/// Helper trait for extending `ClientT`
/// This prevents the wrapper from re-implement all trait methods every time
pub trait ClientExtension<C>
where
    C: 'static + ClientT + Send + Sync,
{
    fn rpc_client(&self) -> &C;

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn notification<'life0, 'life1, 'async_trait, Params>(
        &'life0 self,
        method: &'life1 str,
        params: Params,
    ) -> BoxFuture<'async_trait, Result<(), Error>>
    where
        Params: ToRpcParams + Send,
        Params: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        ClientT::notification(self.rpc_client(), method, params)
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn request<'life0, 'life1, 'async_trait, R, Params>(
        &'life0 self,
        method: &'life1 str,
        params: Params,
    ) -> BoxFuture<'async_trait, Result<R, Error>>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
        R: 'async_trait,
        Params: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        ClientT::request(self.rpc_client(), method, params)
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn batch_request<'a, 'life0, 'async_trait, R>(
        &'life0 self,
        batch: BatchRequestBuilder<'a>,
    ) -> BoxFuture<'async_trait, Result<BatchResponse<'a, R>, Error>>
    where
        R: DeserializeOwned + Debug + 'a,
        'a: 'async_trait,
        R: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        ClientT::batch_request(self.rpc_client(), batch)
    }
}

impl<C> ClientExtension<C> for C
where
    C: ClientT + 'static + Send + Sync,
{
    fn rpc_client(&self) -> &C {
        self
    }
}

/// Helper trait for extending `SubscriptionT`
pub trait SubscriptionExtension<C>: ClientExtension<C>
where
    C: 'static + SubscriptionClientT + Send + Sync,
{
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn subscribe<'a, 'life0, 'async_trait, Notif, Params>(
        &'life0 self,
        subscribe_method: &'a str,
        params: Params,
        unsubscribe_method: &'a str,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Result<Subscription<Notif>, Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        Params: ToRpcParams + Send,
        Notif: DeserializeOwned,
        'a: 'async_trait,
        Notif: 'async_trait,
        Params: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        SubscriptionClientT::subscribe(
            self.rpc_client(),
            subscribe_method,
            params,
            unsubscribe_method,
        )
    }

    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn subscribe_to_method<'a, 'life0, 'async_trait, Notif>(
        &'life0 self,
        method: &'a str,
    ) -> BoxFuture<'async_trait, Result<Subscription<Notif>, Error>>
    where
        Notif: DeserializeOwned,
        'a: 'async_trait,
        Notif: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        SubscriptionClientT::subscribe_to_method(self.rpc_client(), method)
    }
}

impl<C> SubscriptionExtension<C> for C where C: SubscriptionClientT + 'static + Send + Sync {}

pub trait ExtendedClient<C>: 'static + Sized + Send + Sync {
    fn rpc_client(&self) -> &C;

    fn into_extended(self) -> Extended<C, Self> {
        Extended {
            _marker: PhantomData,
            inner: self,
        }
    }
}

impl<C> ExtendedClient<C> for C
where
    C: ClientT + Send + Sync + 'static,
{
    fn rpc_client(&self) -> &C {
        self
    }
}

/// Implements the ClientT trait for the provided type
///
/// ```rust
/// struct CustomClient<C> {
///     client: C,
/// }
///
/// impl <C> AsRef<C> for CustomClient<C> {
///   fn as_ref(&self) -> &C {
///     &self.client
///   }
/// }
///
/// impl_client_trait!(CustomClient<C> where C: 'static + ClientT + Send + Sync)
/// ```
#[allow(unused_macros)]
macro_rules! impl_client_trait {
    // `()` indicates that the macro takes no argument.
    (
        $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?
        $(where $( $glt:tt $( : $gclt:tt $(+ $gdlt:tt )* )? ),+ )?
    ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            ::jsonrpsee::core::client::ClientT
        for $name
            // the bounds are not required here
            $(< $( $lt ),+ >)?
            $(where $( $glt $( : $gclt $(+ $gdlt )* )? ),+ )?
        {
                #[must_use]
                #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
                fn notification<'life0, 'life1, 'async_trait, Params>(
                    &'life0 self,
                    method: &'life1 str,
                    params: Params,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                                Output = ::core::result::Result<(), ::jsonrpsee::core::Error>,
                            > + ::core::marker::Send
                            + 'async_trait,
                    >,
                >
                where
                    Params: ::jsonrpsee::core::traits::ToRpcParams + ::core::marker::Send,
                    Params: 'async_trait,
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    ::jsonrpsee::core::client::ClientT::notification(self.as_ref(), method, params)
                }

                #[must_use]
                #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
                fn request<'life0, 'life1, 'async_trait, R, Params>(
                    &'life0 self,
                    method: &'life1 str,
                    params: Params,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<Output = ::core::result::Result<R, ::jsonrpsee::core::Error>>
                            + ::core::marker::Send
                            + 'async_trait,
                    >,
                >
                where
                    R: ::serde::de::DeserializeOwned,
                    Params: ::jsonrpsee::core::traits::ToRpcParams + ::core::marker::Send,
                    R: 'async_trait,
                    Params: 'async_trait,
                    'life0: 'async_trait,
                    'life1: 'async_trait,
                    Self: 'async_trait,
                {
                    ::jsonrpsee::core::client::ClientT::request(self.as_ref(), method, params)
                }

                #[must_use]
                #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
                fn batch_request<'a, 'life0, 'async_trait, R>(
                    &'life0 self,
                    batch: BatchRequestBuilder<'a>,
                ) -> ::core::pin::Pin<
                    Box<
                        dyn ::core::future::Future<
                                Output = ::core::result::Result<
                                    ::jsonrpsee::core::client::BatchResponse<'a, R>,
                                    ::jsonrpsee::core::Error,
                                >,
                            > + ::core::marker::Send
                            + 'async_trait,
                    >,
                >
                where
                    R: ::serde::de::DeserializeOwned + ::core::fmt::Debug + 'a,
                    'a: 'async_trait,
                    R: 'async_trait,
                    'life0: 'async_trait,
                    Self: 'async_trait,
                {
                    ::jsonrpsee::core::client::ClientT::batch_request(self.as_ref(), batch)
                }
        }
    };
}

/// Implements the [`SubscriptionClientT`] trait for the provided type
///
/// ```rust
/// struct CustomClient<C> {
///     client: C,
/// }
///
/// impl <C> AsRef<C> for CustomClient<C> {
///   fn as_ref(&self) -> &C {
///     &self.client
///   }
/// }
///
/// impl_subscription_trait!(CustomClient<C> where C: 'static + ClientT + Send + Sync)
/// ```
#[allow(unused_macros)]
macro_rules! impl_subscription_trait {
    // `()` indicates that the macro takes no argument.
    (
        $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?
        $(where $( $glt:tt $( : $gclt:tt $(+ $gdlt:tt )* )? ),+ )?
    ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            ::jsonrpsee::core::client::SubscriptionClientT
        for $name
            // the bounds are not required here
            $(< $( $lt ),+ >)?
            $(where $( $glt $( : $gclt $(+ $gdlt )* )? ),+ )?
        {
            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn subscribe<'a, 'life0, 'async_trait, Notif, Params>(
                &'life0 self,
                subscribe_method: &'a str,
                params: Params,
                unsubscribe_method: &'a str,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<Output = ::core::result::Result<
                            ::jsonrpsee::core::client::Subscription<Notif>,
                            ::jsonrpsee::core::Error
                        >>
                        + ::core::marker::Send
                        + 'async_trait,
                >,
            >
            where
                Params: ::jsonrpsee::core::traits::ToRpcParams + ::core::marker::Send,
                Notif: ::serde::de::DeserializeOwned,
                'a: 'async_trait,
                Notif: 'async_trait,
                Params: 'async_trait,
                'life0: 'async_trait,
                Self: 'async_trait,
            {
                ::jsonrpsee::core::client::SubscriptionClientT::subscribe(
                    self.as_ref(),
                    subscribe_method,
                    params,
                    unsubscribe_method,
                )
            }

            #[must_use]
            #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
            fn subscribe_to_method<'a, 'life0, 'async_trait, Notif>(
                &'life0 self,
                method: &'a str,
            ) -> ::core::pin::Pin<
                Box<
                    dyn ::core::future::Future<Output = ::core::result::Result<
                        ::jsonrpsee::core::client::Subscription<Notif>,
                        ::jsonrpsee::core::Error
                    >>
                    + ::core::marker::Send
                    + 'async_trait,
                >
            >
            where
                Notif: ::serde::de::DeserializeOwned,
                'a: 'async_trait,
                Notif: 'async_trait,
                'life0: 'async_trait,
                Self: 'async_trait,
            {
                ::jsonrpsee::core::client::SubscriptionClientT::subscribe_to_method(self.as_ref(), method)
            }
        }
    };
}

struct CustomClient<C> {
    id: u32,
    client: C,
}

impl<C> AsRef<C> for CustomClient<C> {
    fn as_ref(&self) -> &C {
        &self.client
    }
}

impl_client_trait!(CustomClient<C> where C: 'static + ClientT + Send + Sync);
impl_subscription_trait!(CustomClient<C> where C: 'static + SubscriptionClientT + Send + Sync);

/// Should not be implemented directly, use [`ExtendedClient`] instead.
/// # Description
/// Glue between [`ExtendedClient`] to [`ClientT`] and [`SubscriptionClientT`] traits, necessary
/// once only traits defined in the current crate can be implemented for arbitrary types.
pub struct Extended<C, T> {
    _marker: PhantomData<C>,
    pub(crate) inner: T,
}

impl<C, T> Extended<C, T> {
    pub fn new(inner: T) -> Self {
        Self {
            _marker: PhantomData,
            inner,
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<C, T> Extended<C, T>
where
    T: Into<C>,
{
    pub fn into_rpc_client(self) -> C {
        self.inner.into()
    }
}

impl<C, T> Clone for Extended<C, T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker,
            inner: self.inner.clone(),
        }
    }
}

impl<C, T> Debug for Extended<C, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Extended")
            .field("_marker", &self._marker)
            .field("state", &self.inner)
            .finish()
    }
}

impl<C, T> Display for Extended<C, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl<C, T> AsRef<T> for Extended<C, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<C, T> AsMut<T> for Extended<C, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

impl<C, T> ExtendedClient<C> for Extended<C, T>
where
    T: ExtendedClient<C>,
    C: Send + Sync + 'static,
{
    fn rpc_client(&self) -> &C {
        self.inner.rpc_client()
    }

    fn into_extended(self) -> Extended<C, Self> {
        log::warn!("into_rpc_client should not be called on Extended type");
        Extended {
            _marker: PhantomData,
            inner: self,
        }
    }
}

#[async_trait]
impl<C, T> ClientT for Extended<C, T>
where
    T: ExtendedClient<C>,
    C: ClientT + Send + Sync + 'static,
{
    async fn notification<Params>(&self, method: &str, params: Params) -> Result<(), Error>
    where
        Params: ToRpcParams + Send,
    {
        ClientT::notification::<Params>(ExtendedClient::<C>::rpc_client(self), method, params).await
    }

    async fn request<R, Params>(&self, method: &str, params: Params) -> Result<R, Error>
    where
        R: DeserializeOwned,
        Params: ToRpcParams + Send,
    {
        ClientT::request::<R, Params>(ExtendedClient::<C>::rpc_client(self), method, params).await
    }

    async fn batch_request<'a, R>(
        &self,
        batch: BatchRequestBuilder<'a>,
    ) -> Result<BatchResponse<'a, R>, Error>
    where
        R: DeserializeOwned + Debug + 'a,
    {
        ClientT::batch_request::<R>(ExtendedClient::<C>::rpc_client(self), batch).await
    }
}

#[async_trait]
impl<C, T> SubscriptionClientT for Extended<C, T>
where
    T: ExtendedClient<C>,
    C: SubscriptionClientT + Send + Sync + 'static,
{
    async fn subscribe<'a, Notif, Params>(
        &self,
        subscribe_method: &'a str,
        params: Params,
        unsubscribe_method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Params: ToRpcParams + Send,
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe::<Notif, Params>(
            ExtendedClient::<C>::rpc_client(self),
            subscribe_method,
            params,
            unsubscribe_method,
        )
        .await
    }

    async fn subscribe_to_method<'a, Notif>(
        &self,
        method: &'a str,
    ) -> Result<Subscription<Notif>, Error>
    where
        Notif: DeserializeOwned,
    {
        SubscriptionClientT::subscribe_to_method(ExtendedClient::<C>::rpc_client(self), method)
            .await
    }
}
