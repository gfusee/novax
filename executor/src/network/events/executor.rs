use std::marker::PhantomData;
use crate::network::events::proxy::ElasticSearchProxy;

#[derive(Clone, Debug)]
pub struct QueryElasticSearchExecutor<Proxy: ElasticSearchProxy> {
    /// The URL of the gateway to the elastic search server.
    pub gateway_url: String,
    /// A phantom data field to keep the generic `Proxy` type.
    _data: PhantomData<Proxy>
}