use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use crate::net::Fresh;
use crate::server::{Handler, Request, Response};
use crate::status::StatusCode;
use crate::uri::RequestUri::AbsolutePath;
use std::io::copy;
use std::ops::Deref;
use std::sync::Arc;
use crate::runtime::{SyncHashMap};
use crate::uri::RequestUri;

pub struct HandleBox {
    pub url: String,
    pub inner: Box<dyn Handler>,
}

impl Debug for HandleBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandleBox")
            .field("url", &self.url)
            .field("inner", &"*")
            .finish()
    }
}

pub trait MiddleWare: Send + Sync + Debug {
    //return is finish. if finish will be return
    fn handle(&self, req: &mut Request, res: &mut Response) -> bool;
}

#[derive(Debug)]
pub struct Route {
    pub container: SyncHashMap<String, Arc<Box<dyn Any>>>,
    pub middleware: Vec<Box<dyn MiddleWare>>,
    pub handlers: SyncHashMap<String, HandleBox>,
}


impl Route {
    pub fn new() -> Self {
        Self {
            container: SyncHashMap::new(),
            middleware: vec![],
            handlers: SyncHashMap::new(),
        }
    }
    /// handle a fn
    /// for example:
    /// ```rust
    /// use mco_http::route::Route;
    /// use mco_http::server::{Request, Response};
    ///
    /// let mut route = Route::new();
    /// //Common way
    /// route.handle_fn("/", |req: Request, res: Response| {
    ///         res.send(b"Hello World!").unwrap();
    ///     });
    ///
    /// //or you can use method. It can even nest calls to Handle
    /// fn hello(req: Request, res: Response) {
    ///     res.send(b"Hello World!").unwrap();
    /// }
    /// route.handle_fn("/",hello);
    ///
    ///
    /// ```
    pub fn handle_fn<H: Handler + 'static>(&self, url: &str, h: H) {
        self.handlers.insert(url.to_string(), HandleBox {
            url: url.to_string(),
            inner: Box::new(h),
        });
    }

    pub fn add_middleware<M: MiddleWare + 'static>(&mut self, m: M) {
        self.middleware.push(Box::new(m));
    }

    pub fn insert<T: Any>(&self, key: &str, data: T) {
        self.container.insert(key.to_string(), Arc::new(Box::new(data)));
    }

    pub fn get<T: Any>(&self, key: &str) -> Option<&T> {
        match self.container.get(key) {
            None => {
                None
            }
            Some(b) => {
                let r: Option<&T> = b.downcast_ref();
                r
            }
        }
    }

    /// index will get from container.if not exist will be panic!
    pub fn index<T: Any>(&self, key: &str) -> &T {
        self.get(key).expect(&format!("key:{} Does not exist in the container",key))
    }
}


impl Handler for Route {
    fn handle(&self, mut req: Request, mut res: Response<'_, Fresh>) {
        for x in &self.middleware {
            //finish?.this is safety
            if x.handle(&mut req, &mut res) {
                return;
            }
        }
        match req.uri().path() {
            p => {
                let path = &p[0..p.find("?").unwrap_or(p.len())];
                match self.handlers.get(path) {
                    None => {
                        //404
                        res.status = http::StatusCode::NOT_FOUND;
                        return;
                    }
                    Some(h) => {
                        let i = &h.inner;
                        i.handle(req, res);
                        return;
                    }
                }
            }
        }
    }
}

impl Handler for Arc<Route> {
    fn handle(&self, mut req: Request, mut res: Response<'_, Fresh>) {
        Route::handle(self, req, res)
    }
}