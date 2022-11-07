use convert_case::{Case, Casing};
use reqwest::{
    header::{HeaderName, HeaderValue},
    Client, Url,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    body: Option<String>,
    headers: HashMap<String, String>, // Headers key is converted to kebab-case, value is untouched
    method: RequestMethod,
    url: String,
    params: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestMethod {
    GET,
    POST,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    status: u16,
    headers: HashMap<String, String>,
    body: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    status: Option<u16>,
    url: Option<String>,
}

impl Request {
    pub fn new(
        body: Option<String>,
        headers: HashMap<String, String>,
        method: RequestMethod,
        url: String,
        params: HashMap<String, String>,
    ) -> Request {
        Request {
            body,
            headers: headers
                .iter()
                .map(|(k, v)| (k.to_case(Case::Kebab), v.to_string()))
                .collect(),
            method,
            url,
            params,
        }
    }

    async fn send_request(&self) -> Result<Response, Error> {
        let client = Client::new();
        let headers = &self.headers;
        let response = match &self.method {
            RequestMethod::GET => {
                client.get(Url::parse_with_params(&self.url, &self.params).unwrap())
            }
            RequestMethod::POST => client.post(&self.url),
        }
        .headers(
            headers
                .into_iter()
                .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
                .collect(),
        )
        .send()
        .await;

        match response {
            Ok(response) => {
                return Ok(Response {
                    status: response.status().as_u16(),
                    headers: response
                        .headers()
                        .iter()
                        .map(|(k, v): (&HeaderName, &HeaderValue)| {
                            (k.to_string(), v.to_str().unwrap().to_string())
                        })
                        .collect(),
                    // May crash if there is no body in the response
                    body: serde_json::from_str(response.text().await.ok().unwrap().as_str())
                        .unwrap(),
                });
            }
            Err(error) => {
                return Err(Error {
                    status: error.status().map(|s| s.as_u16()),
                    url: error.url().map(|u| u.to_string()),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Error, Request, RequestMethod, Response};

    #[tokio::test]
    async fn make_get_request() {
        let req = Request {
            body: None,
            url: String::from("https://postman-echo.com/get?name=john"),
            method: RequestMethod::GET,
            headers: HashMap::new(),
            params: HashMap::new(),
        };

        let res = req.send_request().await;
        assert_eq!(true, res.is_ok());
    }

    #[tokio::test]
    async fn make_get_request_with_params() {
        let req = Request {
            body: None,
            url: String::from("https://postman-echo.com/get"),
            method: RequestMethod::GET,
            headers: HashMap::new(),
            params: HashMap::from([("name".to_string(), "john".to_string())]),
        };

        let res: Result<Response, Error> = req.send_request().await;
        assert_eq!(true, res.is_ok());
        assert_eq!("john", res.ok().unwrap().body["args"]["name"]);
    }

    #[tokio::test]
    async fn make_get_request_with_headers() {
        let req = Request {
            body: None,
            url: String::from("https://postman-echo.com/get"),
            method: RequestMethod::GET,
            headers: HashMap::from([("randomHeader".to_string(), "1337".to_string())]),
            params: HashMap::new(),
        };

        let res = req.send_request().await;
        assert_eq!(true, res.is_ok());
        assert_eq!(200, res.as_ref().ok().unwrap().status);
        dbg!(res.as_ref().ok().unwrap());
        assert_eq!(
            "1337",
            res.as_ref().ok().unwrap().body["headers"]["random-header"]
        );
    }

    // #[tokio::test]
    // async fn make_get_request_with_body() {
    //     let req = Request::new(
    //         Some("RAWR!! x3 nuzzles! pounces on u uwu u so warm.".to_string()),
    //         HashMap::new(),
    //         RequestMethod::GET,
    //         String::from("https://postman-echo.com/get"),
    //         HashMap::new(),
    //     );

    //     let res = req.send_request().await;
    //     assert_eq!(true, res.is_ok());
    //     assert_eq!(200, res.as_ref().ok().unwrap().status);
    //     dbg!(res.as_ref().ok().unwrap());
    //     assert_eq!("1337", res.as_ref().ok().unwrap().body["args"]["body"]);
    // }
}
