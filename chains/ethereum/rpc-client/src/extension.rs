/// Implements the [`::jsonrpsee::core::client::ClientT`] trait for the provided type
///
/// ```rust
/// use jsonrpsee::core::client::ClientT;
///
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
#[macro_export]
macro_rules! impl_client_trait {
    (
        $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?
        $(where $( $glt:tt $( : $gclt:tt $(+ $gdlt:tt )* )? ),+ )?
    ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            ::jsonrpsee::core::client::ClientT
        for $name
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
                    batch: ::jsonrpsee::core::params::BatchRequestBuilder<'a>,
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
pub use impl_client_trait;

/// Implements the [`::jsonrpsee::core::client::SubscriptionClientT`] trait for the provided type
///
/// ```rust
/// use jsonrpsee::core::client::SubscriptionClientT;
///
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
/// impl_subscription_trait!(CustomClient<C> where C: 'static + SubscriptionT + Send + Sync)
/// ```
#[macro_export]
macro_rules! impl_subscription_trait {
    (
        $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)?
        $(where $( $glt:tt $( : $gclt:tt $(+ $gdlt:tt )* )? ),+ )?
    ) => {
        impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
            ::jsonrpsee::core::client::SubscriptionClientT
        for $name
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
pub use impl_subscription_trait;
