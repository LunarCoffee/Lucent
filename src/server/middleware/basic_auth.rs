use pwhash::bcrypt;

use crate::{consts, log};
use crate::http::message::MessageBuilder;
use crate::http::request::Request;
use crate::http::response::Response;
use crate::http::response::Status;
use crate::server::config::auth_info::AuthInfo;
use crate::server::config::Config;
use crate::server::config::route_spec::RouteSpec;
use crate::server::middleware::{MiddlewareOutput, MiddlewareResult};

pub struct BasicAuthChecker<'a> {
    request: &'a Request,
    config: &'a Config,
}

impl<'a> BasicAuthChecker<'a> {
    pub fn new(request: &'a Request, config: &'a Config) -> Self {
        BasicAuthChecker { request, config }
    }

    pub fn check(&self) -> MiddlewareResult<bool> {
        let target = self.request.uri.to_string();
        for (RouteSpec(rule_regex), auth_info) in &self.config.basic_auth {
            if rule_regex.captures(&target).is_some() {
                return match self.request.headers.get(consts::H_AUTHORIZATION) {
                    Some(auth) => self.check_auth_header(&auth, &auth_info),
                    _ => self.www_authenticate_output(&auth_info),
                };
            }
        }
        Ok(false)
    }

    fn check_auth_header(&self, auth: &Vec<String>, auth_info: &AuthInfo) -> MiddlewareResult<bool> {
        let challenge = self.www_authenticate_output(auth_info);

        let auth = auth[0].splitn(2, ' ').collect::<Vec<_>>();
        if auth.len() > 1 && auth[0].eq_ignore_ascii_case(consts::H_AUTH_BASIC) {
            let encoded_credentials = &auth[1];
            let maybe_credentials = base64::decode(encoded_credentials).map(|c| String::from_utf8(c));
            let credentials = match maybe_credentials {
                Ok(Ok(c)) => c,
                _ => return challenge,
            };

            let credentials = credentials.splitn(2, ':').collect::<Vec<_>>();
            if credentials.len() > 1 {
                let user = credentials[0];
                let password = credentials[1];
                for c in &auth_info.credentials {
                    if c.user == user && bcrypt::verify(password, &c.password_hash) {
                        return Ok(true);
                    }
                }
            }
        }
        challenge
    }

    fn www_authenticate_output(&self, auth_info: &AuthInfo) -> MiddlewareResult<bool> {
        log::info(format!("({}) {} {}", Status::Unauthorized, self.request.method, self.request.uri));

        let auth = format!("{} {}=\"{}\"", consts::H_AUTH_BASIC, consts::H_AUTH_REALM, auth_info.realm);
        let response = MessageBuilder::<Response>::new()
            .with_status(Status::Unauthorized)
            .with_header(consts::H_WWW_AUTHENTICATE, &auth)
            .build();
        Err(MiddlewareOutput::Response(response, false))
    }
}
