use crate::MappedIdentity;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Serialize, Deserialize, Clone, TypedBuilder)]
pub struct Response {
    pub code: u16,
    pub message: String,
    pub reply: Reply,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Reply {
    NormalReply(NormalReply),
    CustomerId(i32),
    Empty,
}

#[derive(Debug, Serialize, Deserialize, Clone, TypedBuilder)]
pub struct NormalReply {
    pub customer_id: i32,
    pub leak_id: String,
    pub identities: Vec<MappedIdentity>,
}

impl Response {
    pub fn create_response_with_identities(
        code: u16,
        message: String,
        normal_reply: NormalReply,
    ) -> Response {
        Response::builder()
            .code(code)
            .message(message)
            .reply(Reply::NormalReply(normal_reply))
            .build()
    }

    pub fn create_empty_response(code: u16, message: String) -> Response {
        Response::builder()
            .code(code)
            .message(message)
            .reply(Reply::Empty)
            .build()
    }

    pub fn create_response_with_id(code: u16, message: String, customer_id: i32) -> Response {
        Response::builder()
            .code(code)
            .message(message)
            .reply(Reply::CustomerId(customer_id))
            .build()
    }
}
