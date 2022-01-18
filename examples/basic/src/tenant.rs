use async_graphql::{Context, Error, SimpleObject};
use async_graphql_relay::{GlobalId, Node, NodeInstance};
use async_trait::async_trait;
use uuid::Uuid;

use crate::Node;

#[derive(Debug, SimpleObject, Node)]
pub struct Tenant {
    pub id: GlobalId<Self>,
    pub name: String,
    pub description: String,
}

#[async_trait]
impl NodeInstance for Tenant {
    type Node = Node;

    async fn fetch_by_id(_ctx: &Context<'_>, id: Uuid) -> Result<Option<Self::Node>, Error> {
        Ok(Some(
            Tenant {
                id: id.into(),
                name: "My Company".to_string(),
                description: "Testing123".to_string(),
            }
            .into(),
        ))
    }
}
