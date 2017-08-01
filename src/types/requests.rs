use types::content;
use std::collections::HashMap;
use serde::{Deserialize,Serialize};
use hyper::Method;
use strfmt::strfmt;

// fn req<T>(mx: &mut ::MatrixClient,req: Request)-> ::MatrixFuture<T>
// where T: Deserialize + 'static
// {
//     let endp: String = req.fmt_endpoint(mx);
//     let body = ::serde_json::to_string(&req.body).unwrap();
//     mx.req::<T>(req.method, &endp, req.params, Some(body.into()))
// }

#[derive(Debug)]
pub struct Request<B>
where B: Serialize,
{
    pub method: Method,
    pub endpoint: String,
    pub params: Vec<String>,
    pub body: B,
}

impl<B> Request<B>
where B: Serialize
// + 'static
{
    pub fn send<R>(&self,mx: &mut ::MatrixClient)-> ::MatrixFuture<()>
    where R: Deserialize + 'static
    {
        let endp: String = self.fmt_endpoint(mx);
        let body = match ::serde_json::to_string(&self.body) {
            Ok(x) => x,
            Err(e) => return Box::new(::futures::future::err(e.into()))
        };
        mx.req(self.method.clone(), &endp, self.params.clone(), Some(body.into()))
    }
    pub fn discarding_send(&self,mx: &mut ::MatrixClient)-> ::MatrixFuture<()>
    // where R: Deserialize + 'static
    {
        let endp: String = self.fmt_endpoint(mx);
        let body = match ::serde_json::to_string(&self.body) {
            Ok(x) => x,
            Err(e) => return Box::new(::futures::future::err(e.into()))
        };
        mx.discarding_req(self.method.clone(), &endp, self.params.clone(), Some(body.into()))
    }
    fn fmt_endpoint(&self,mx: &::MatrixClient) -> String {
        let mut vars = HashMap::new();
        vars.insert("user_id".to_string(), &mx.user_id);
        strfmt(&self.endpoint,&vars).unwrap()
    }
}

#[derive(Serialize, Debug)]
pub struct Presence {
    pub presence: content::Presence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_msg: Option<String>,
}

#[test]
fn instantiate_req() {
    let req = Request{
        method: Method::Put,
        endpoint: String::from("end"),
        params: vec!(),
        body: Presence{
            presence: content::Presence::Online,
            status_msg: Some(String::from("I am text YAY")),
        },
    };
    println!("{:?}",req);
}
