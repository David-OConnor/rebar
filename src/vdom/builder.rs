use web_sys::Element;

use super::{alias::*, App};

use crate::{dom_types::View, orders::OrdersContainer, routing, util};

//type InitFn<Ms, Mdl, ElC, GMs> =
//    Box<dyn FnOnce(routing::Url, &mut OrdersContainer<Ms, Mdl, ElC, GMs>) -> Mdl>;
pub type InitFn<Ms, Mdl, ElC, GMs> =
    Box<dyn FnOnce(routing::Url, &mut OrdersContainer<Ms, Mdl, ElC, GMs>) -> Init<Mdl>>;

pub trait MountPoint {
    fn element(self) -> Element;
}

impl MountPoint for &str {
    fn element(self) -> Element {
        util::document().get_element_by_id(self).unwrap_or_else(|| {
            panic!(
                "Can't find element with id={:?} - app cannot be mounted!\n\
                 (Id defaults to \"app\", or can be set with the .mount() method)",
                self
            )
        })
    }
}

impl MountPoint for Element {
    fn element(self) -> Element {
        self
    }
}

impl MountPoint for web_sys::HtmlElement {
    fn element(self) -> Element {
        self.into()
    }
}

/// Used for handling initial routing.
pub enum UrlHandling {
    PassToRoutes,
    None,
    // todo: Expand later, as-required
}

/// Used as a flexible wrapper for the init function.
pub struct Init<Mdl> {
    //    init: InitFn<Ms, Mdl, ElC, GMs>,
    model: Mdl,
    url_handling: UrlHandling,
}

impl<Mdl> Init<Mdl> {
    pub const fn new(model: Mdl) -> Self {
        Self {
            model,
            url_handling: UrlHandling::PassToRoutes,
        }
    }

    pub const fn new_with_url_handling(model: Mdl, url_handling: UrlHandling) -> Self {
        Self {
            model,
            url_handling,
        }
    }
}

/// Used to create and store initial app configuration, ie items passed by the app creator
pub struct Builder<Ms: 'static, Mdl: 'static, ElC: View<Ms>, GMs> {
    init: InitFn<Ms, Mdl, ElC, GMs>,
    update: UpdateFn<Ms, Mdl, ElC, GMs>,
    sink: Option<SinkFn<Ms, Mdl, ElC, GMs>>,
    view: ViewFn<Mdl, ElC>,
    mount_point: Option<Element>,
    routes: Option<RoutesFn<Ms>>,
    window_events: Option<WindowEvents<Ms, Mdl>>,
}

impl<Ms, Mdl, ElC: View<Ms> + 'static, GMs: 'static> Builder<Ms, Mdl, ElC, GMs> {
    /// Constructs the Builder.
    pub(super) fn new(
        init: InitFn<Ms, Mdl, ElC, GMs>,
        update: UpdateFn<Ms, Mdl, ElC, GMs>,
        view: ViewFn<Mdl, ElC>,
    ) -> Self {
        Self {
            init,
            update,
            sink: None,
            view,
            mount_point: None,
            routes: None,
            window_events: None,
        }
    }

    /// Choose the element where the application will be mounted.
    /// The default one is the element with `id` = "app".
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // argument is `&str`
    /// mount("another_id")
    ///
    /// // argument is `HTMLElement`
    /// // NOTE: Be careful with mounting into body,
    /// // it can cause hard-to-debug bugs when there are other scripts in the body.
    /// mount(seed::body())
    ///
    /// // argument is `Element`
    /// mount(seed::body().querySelector("section").unwrap().unwrap())
    /// ```
    pub fn mount(mut self, mount_point: impl MountPoint) -> Self {
        // @TODO: Remove as soon as Webkit is fixed and older browsers are no longer in use.
        // https://github.com/David-OConnor/seed/issues/241
        // https://bugs.webkit.org/show_bug.cgi?id=202881
        let _ = util::document().query_selector("html");

        self.mount_point = Some(mount_point.element());
        self
    }

    /// Registers a function which maps URLs to messages.
    pub fn routes(mut self, routes: RoutesFn<Ms>) -> Self {
        self.routes = Some(routes);
        self
    }

    /// Registers a function which decides how window events will be handled.
    pub fn window_events(mut self, evts: WindowEvents<Ms, Mdl>) -> Self {
        self.window_events = Some(evts);
        self
    }

    /// Registers a sink function.
    ///
    /// The sink function is a function which can update the model based
    /// on global messages. Consider to use a sink function when a
    /// submodule needs to trigger changes in other modules.
    pub fn sink(mut self, sink: SinkFn<Ms, Mdl, ElC, GMs>) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Turn this [`Builder`] into an [`App`] which is ready to run.
    ///
    /// [`Builder`]: struct.Builder.html
    /// [`App`]: struct.App.html
    pub fn finish(mut self) -> App<Ms, Mdl, ElC, GMs> {
        if self.mount_point.is_none() {
            self = self.mount("app")
        }

        let app = App::new(
            self.update,
            self.sink,
            self.view,
            self.mount_point.unwrap(),
            self.routes,
            self.window_events,
        );

        let mut initial_orders = OrdersContainer::new(app.clone());
        let mut init = (self.init)(routing::initial_url(), &mut initial_orders);

        match init.url_handling {
            UrlHandling::PassToRoutes => {
                let url = routing::initial_url();
                if let Some(r) = self.routes {
                    if let Some(u) = r(url) {
                        (self.update)(u, &mut init.model, &mut initial_orders);
                    }
                }
            }
            UrlHandling::None => (),
        };

        app.cfg.initial_orders.replace(Some(initial_orders));
        app.data.model.replace(Some(init.model));

        app
    }
}
