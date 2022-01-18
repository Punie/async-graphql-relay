use async_graphql::{ComplexObject, Context, Error, SimpleObject};
use async_graphql_relay::{GlobalId, Node, NodeInstance};
use async_trait::async_trait;
use uuid::Uuid;

use crate::Node;

#[derive(Debug, SimpleObject, Node)]
#[graphql(complex)]
#[relay(type_name = "User")]
pub struct User {
    pub id: GlobalId<Self>,
    pub name: String,
    pub role: String,
}

#[async_trait]
impl NodeInstance for User {
    type Node = Node;

    async fn fetch_by_id(_ctx: &Context<'_>, id: Uuid) -> Result<Option<Self::Node>, Error> {
        Ok(Some(
            User {
                id: id.into(),
                name: "Oscar".to_string(),
                role: "Testing123".to_string(),
            }
            .into(),
        ))
    }
}

#[ComplexObject]
impl User {
    pub async fn test(&self) -> String {
        "testing".to_string()
    }
}
