use crate::tenant::Tenant;
use crate::user::User;
use actix_web::guard;
use actix_web::web::Data;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptyMutation, EmptySubscription, Error, Interface, Object, ID};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use async_graphql_relay::{GlobalId, NodeInterface};
use uuid::Uuid;

mod tenant;
mod user;

pub struct QueryRoot;

#[derive(Interface, NodeInterface)]
#[graphql(field(name = "id", type = "NodeGlobalID"))]
pub enum Node {
    User(User),
    Tenant(Tenant),
}

#[Object]
impl QueryRoot {
    async fn user(&self) -> User {
        User {
            id: Uuid::parse_str("92ba0c2d-4b4e-4e29-91dd-8f96a078c3ff")
                .unwrap()
                .into(),
            name: "Oscar".to_string(),
            role: "Testing123".to_string(),
        }
    }

    async fn tenant(&self) -> Tenant {
        Tenant {
            id: Uuid::parse_str("4e02ec03-f82f-46da-8572-39975bf97d9d")
                .unwrap()
                .into(),
            name: "My Company".to_string(),
            description: "Testing123".to_string(),
        }
    }

    async fn node(&self, ctx: &Context<'_>, id: ID) -> Result<Node, Error> {
        Node::fetch_node(ctx, id.into()).await
    }
}

pub type Schema = async_graphql::Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub async fn handler(schema: web::Data<Schema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn playground() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    println!("Listening http://localhost:8080/ ...");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(web::resource("/").guard(guard::Post()).to(handler))
            .service(web::resource("/").guard(guard::Get()).to(playground))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
