use crate::handler::{not_found, Handler};
use crate::http::{Method, Request, Response, StatusCode};
use crate::router::route::Route;

pub struct Router {
    routes: Vec<Route>,
    not_found: Handler,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            not_found: Box::new(not_found),
        }
    }

    pub fn add<F>(&mut self, method: Method, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.routes.push(Route::new(method, pattern, Box::new(handler)));
        self
    }

    pub fn get<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add(Method::Get, pattern, handler)
    }

    pub fn post<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add(Method::Post, pattern, handler)
    }

    pub fn put<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add(Method::Put, pattern, handler)
    }

    pub fn delete<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.add(Method::Delete, pattern, handler)
    }

    pub fn set_not_found<F>(&mut self, handler: F) -> &mut Self
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.not_found = Box::new(handler);
        self
    }

    pub fn dispatch(&self, mut request: Request) -> Response {
        let mut candidates: Vec<_> = self
            .routes
            .iter()
            .filter_map(|route| route.matches(&request.path).map(|params| (route, params)))
            .collect();

        if candidates.is_empty() {
            return (self.not_found)(&request);
        }

        let wanted = request.method;
        let chosen = candidates
            .iter()
            .position(|(route, _)| route.method == wanted)
            .or_else(|| {
                if wanted == Method::Head {
                    candidates
                        .iter()
                        .position(|(route, _)| route.method == Method::Get)
                } else {
                    None
                }
            });

        match chosen {
            Some(index) => {
                let (route, params) = candidates.swap_remove(index);
                request.params = params;
                route.call(&request)
            }
            None => {
                let mut allowed: Vec<&str> = Vec::new();
                for (route, _) in &candidates {
                    let name = route.method.as_str();
                    if !allowed.contains(&name) {
                        allowed.push(name);
                    }
                }
                Response::text(StatusCode::MethodNotAllowed, "405 Method Not Allowed\n")
                    .with_header("Allow", allowed.join(", "))
            }
        }
    }
}
