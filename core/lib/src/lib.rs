#![forbid(unsafe_code)]

use std::{fmt, marker::PhantomData, str::FromStr};

use async_graphql::{
    futures_util::future::join_all, Context, Error, InputValueError, InputValueResult, Scalar,
    ScalarType, Value,
};
use uuid::Uuid;

pub use async_graphql_relay_derive::*;

#[doc(hidden)]
pub use async_trait::async_trait;
#[doc(hidden)]
pub use base64;

#[async_trait]
pub trait NodeInterface
where
    Self: Sized,
{
    async fn fetch_node(ctx: &Context<'_>, id: String) -> Result<Self, Error>;

    async fn fetch_nodes(ctx: &Context<'_>, ids: Vec<String>) -> Vec<Option<Self>> {
        let futures = ids.into_iter().map(|id| Self::fetch_node(ctx, id));

        join_all(futures)
            .await
            .into_iter()
            .map(Result::ok)
            .collect()
    }
}

pub trait NamedNode {
    const TYPE_NAME: &'static str;
}

#[async_trait]
pub trait NodeInstance: NamedNode {
    type Node: NodeInterface;

    async fn fetch_by_id(ctx: &Context<'_>, id: Uuid) -> Result<Option<Self::Node>, Error>;
}

pub fn to_global_id(type_name: &str, id: Uuid) -> String {
    let clear = format!("{}:{}", type_name, id.to_string());

    base64::encode(clear)
}

pub fn from_global_id(global_id: &str) -> Result<(String, Uuid), Error> {
    let decoded = base64::decode(global_id)?;
    let clear = String::from_utf8(decoded)?;

    let (type_name, id) = clear
        .split_once(':')
        .ok_or(Error::new("Invalid global ID provided to node query"))?;
    let uuid = Uuid::parse_str(id)
        .map_err(|_err| Error::new("Invalid global ID provided to node query"))?;

    Ok((type_name.into(), uuid))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GlobalId<T: ?Sized>(Uuid, PhantomData<T>);

impl<T> From<Uuid> for GlobalId<T> {
    fn from(uuid: Uuid) -> Self {
        Self(uuid, PhantomData)
    }
}

impl<T> From<GlobalId<T>> for Uuid {
    fn from(global_id: GlobalId<T>) -> Self {
        global_id.0
    }
}

impl<T> From<&GlobalId<T>> for Uuid {
    fn from(global_id: &GlobalId<T>) -> Self {
        global_id.0
    }
}

impl<T: NodeInstance> From<GlobalId<T>> for String {
    fn from(global_id: GlobalId<T>) -> Self {
        to_global_id(T::TYPE_NAME, global_id.into())
    }
}

impl<T: NodeInstance> From<&GlobalId<T>> for String {
    fn from(global_id: &GlobalId<T>) -> Self {
        to_global_id(T::TYPE_NAME, global_id.into())
    }
}

impl<T: NodeInstance> ToString for GlobalId<T> {
    fn to_string(&self) -> String {
        String::from(self)
    }
}

impl<T: NodeInstance> fmt::Debug for GlobalId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("GlobalID")
            .field(&self.0)
            .field(&T::TYPE_NAME)
            .finish()
    }
}

impl<T: NodeInstance> FromStr for GlobalId<T> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (_, uuid) = from_global_id(s)?;
        Ok(uuid.into())
    }
}

#[Scalar(name = "ID")]
impl<T: NodeInstance + Send + Sync> ScalarType for GlobalId<T> {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => s
                .parse::<GlobalId<T>>()
                .map_err(|err| InputValueError::custom(format!("{:?}", err))),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(String::from(self))
    }
}
