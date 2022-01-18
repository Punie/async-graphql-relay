use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[macro_use]
extern crate quote;
extern crate proc_macro;

#[derive(FromDeriveInput, Default)]
#[darling(default, attributes(relay))]
struct NodeAttributes {
    type_name: Option<String>,
}

/// ```
/// #[derive(SimpleObject, Node)]
/// #[graphql(complex)]
/// #[relay(type_name = "User")]
/// pub struct User {
///     pub id: GlobalId<User>,
///     pub name: String,
///     pub role: String,
/// }
/// ```
#[proc_macro_derive(Node, attributes(relay))]
pub fn derive_relay_node_object(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    let attrs =
        NodeAttributes::from_derive_input(&input).expect("Error parsing 'Node' macro options!");
    let DeriveInput { ident, data, .. } = input;

    if !matches!(data, Data::Struct(_)) {
        panic!("The 'Node' macro can only be used on structs!");
    }

    let value = if let Some(type_name) = attrs.type_name {
        type_name
    } else {
        ident.to_string()
    };

    quote! {
        impl async_graphql_relay::NamedNode for #ident {
            const TYPE_NAME: &'static str = #value;
        }
    }
    .into()
}

/// ```
/// #[derive(Interface, NodeInterface)]
/// #[graphql(field(name = "id", type = "NodeGlobalID"))]
/// pub enum Node {
///     User(User),
///     Tenant(Tenant),
/// }
/// ```
#[proc_macro_derive(NodeInterface)]
pub fn derive_relay_interface(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);

    let id_ident = format_ident!("{}GlobalID", ident);
    let impls;
    let node_matchers;
    if let Data::Enum(data) = &data {
        impls = data.variants.iter().map(|variant| {
            let variant_ident = &variant.ident;
            quote! {
                impl std::convert::From<&async_graphql_relay::GlobalId<#variant_ident>> for #id_ident {
                    fn from(t: &async_graphql_relay::GlobalId<#variant_ident>) -> Self {
                        #id_ident(String::from(t))
                    }
                }
            }
        });

        node_matchers = data.variants.iter().map(|variant| {
            let variant_ident = &variant.ident;
            quote! {
                <#variant_ident as async_graphql_relay::NamedNode>::TYPE_NAME => {
                    <#variant_ident as async_graphql_relay::NodeInstance>::fetch_by_id(
                        ctx,
                        relay_id.parse::<GlobalId<#variant_ident>>()?.into(),
                    )
                    .await?
                    .ok_or_else(|| async_graphql::Error::new("A node with the specified id could not be found!"))
                }
            }
        });
    } else {
        panic!("The 'RelayNodeObject' macro can only be used on enums!");
    }

    quote! {
        #[derive(Clone, Debug)]
        pub struct #id_ident(String);

        #(#impls)*

        #[async_graphql::Scalar(name = "ID")]
        impl async_graphql::ScalarType for #id_ident {
            fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
                unimplemented!();
            }

            fn to_value(&self) -> async_graphql::Value {
                async_graphql::Value::String(self.0.clone())
            }
        }

        #[async_graphql_relay::async_trait]
        impl async_graphql_relay::NodeInterface for Node {
            async fn fetch_node(ctx: &async_graphql::Context<'_>, relay_id: String) -> Result<Self, async_graphql::Error> {
                let (prefix, _) = async_graphql_relay::from_global_id(&relay_id)?;

                match prefix.as_str() {
                    #(#node_matchers)*
                    _ => Err(async_graphql::Error::new("A node with the specified id could not be found!")),
                }
            }
        }
    }
    .into()
}
