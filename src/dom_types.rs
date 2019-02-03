//! This module contains structs and enums that represent dom types, and their parts.
//! These are the types used internally by our virtual dom.

use crate::vdom::Mailbox;
use core::convert::AsRef;
use pulldown_cmark;
use std::{collections::HashMap, fmt};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys;

/// Common Namespaces
#[derive(Debug, Clone, PartialEq)]
pub enum Namespace {
    /// SVG Namespace
    Svg,
    Custom(String),
}

impl Namespace {
    pub fn as_str(&self) -> &str {
        use self::Namespace::*;
        match self {
            Svg => "http://www.w3.org/2000/svg",
            Custom(s) => s,
        }
    }
}

// todo cleanup enums vs &strs for restricting events/styles/attrs to
// todo valid ones.

/// Create an event that passes no data, other than it occured. Foregoes using a closure,
/// in favor of pointing to a message directly.
pub fn simple_ev<Ms, T>(trigger: T, message: Ms) -> Listener<Ms>
where
    Ms: Clone + 'static,
    T: ToString,
{
    let handler = || message;
    let closure = move |_| handler.clone()();
    Listener::new(&trigger.to_string(), Some(Box::new(closure)))
}

/// Create an event that passes a String of field text, for fast input handling.
pub fn input_ev<Ms, T: ToString>(
    trigger: T,
    mut handler: impl FnMut(String) -> Ms + 'static,
) -> Listener<Ms> {
    let closure = move |event: web_sys::Event| {
        if let Some(target) = event.target() {
            return handler(util::input_value(&target));
        }
        handler(String::new())
    };

    Listener::new(&trigger.to_string(), Some(Box::new(closure)))
}

/// Create an event that passes a web_sys::Event, allowing full control of
/// event-handling
pub fn raw_ev<Ms, T: ToString>(
    trigger: T,
    mut handler: impl FnMut(web_sys::Event) -> Ms + 'static,
) -> Listener<Ms> {
    let closure = move |event: web_sys::Event| handler(event);
    Listener::new(&trigger.to_string(), Some(Box::new(closure)))
}

/// Create an event that passes a web_sys::KeyboardEvent, allowing easy access
/// to items like key_code() and key().
pub fn keyboard_ev<Ms, T: ToString>(
    trigger: T,
    mut handler: impl FnMut(web_sys::KeyboardEvent) -> Ms + 'static,
) -> Listener<Ms> {
    let closure = move |event: web_sys::Event| {
        handler(event.dyn_ref::<web_sys::KeyboardEvent>().unwrap().clone())
    };
    Listener::new(&trigger.to_string(), Some(Box::new(closure)))
}

/// See keyboard_ev
pub fn mouse_ev<Ms, T: ToString>(
    trigger: T,
    mut handler: impl FnMut(web_sys::MouseEvent) -> Ms + 'static,
) -> Listener<Ms> {
    let closure = move |event: web_sys::Event| {
        handler(event.dyn_ref::<web_sys::MouseEvent>().unwrap().clone())
    };
    Listener::new(&trigger.to_string(), Some(Box::new(closure)))
}

type EventHandler<Ms> = Box<FnMut(web_sys::Event) -> Ms>;

/// Ev-handling for Elements
pub struct Listener<Ms> {
    pub trigger: Ev,
    pub handler: Option<EventHandler<Ms>>,
    // We store closure here so we can detach it later.
    pub closure: Option<Closure<FnMut(web_sys::Event)>>,
    // Control listeners prevent input on controlled input elements, and
    // are not assoicated with a message.
    pub control: bool,
    pub id: Option<u32>,
}

impl<Ms> fmt::Debug for Listener<Ms> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Listener {{ trigger:{:?}, handler:{:?}, closure:{:?}, control:{:?}, id:{:?} }}",
            self.trigger,
            fmt_hook_fn(&self.handler),
            fmt_hook_fn(&self.closure),
            self.control,
            self.id
        )
    }
}

// https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen/closure/struct.Closure.html
impl<Ms> Listener<Ms> {
    pub fn new(trigger: &str, handler: Option<EventHandler<Ms>>) -> Self {
        Self {
            // We use &str instead of Event here to allow flexibility in helper funcs,
            // without macros by using ToString.
            trigger: trigger.into(),
            handler,
            closure: None,
            control: false,
            id: None,
        }
    }

    pub fn new_control() -> Self {
        Self {
            trigger: dom_types::Ev::Input,
            handler: None,
            closure: None,
            control: true,
            id: None,
        }
    }

    /// This method is where the processing logic for events happens.
    pub fn attach<T>(&mut self, el_ws: &T, mailbox: Mailbox<Ms>)
    where
        T: AsRef<web_sys::EventTarget>,
    {
        // This and detach taken from Draco.
        let mut handler = self.handler.take().expect("Can't find old handler");

        let closure =
            Closure::wrap(
                Box::new(move |event: web_sys::Event| mailbox.send(handler(event)))
                    as Box<FnMut(web_sys::Event) + 'static>,
            );

        (el_ws.as_ref() as &web_sys::EventTarget)
            .add_event_listener_with_callback(
                self.trigger.as_str(),
                closure.as_ref().unchecked_ref(),
            )
            .expect("problem adding listener to element");

        // Store the closure so we can detach it later. Not detaching it when an element
        // is removed will trigger a panic.
        self.closure = Some(closure);
        //        self.handler.replace(handler);  // todo ?
    }

    pub fn detach<T>(&self, el_ws: &T)
    where
        T: AsRef<web_sys::EventTarget>,
    {
        // This and attach taken from Draco.
        let closure = self.closure.as_ref().expect("Can't find closure to detach");

        (el_ws.as_ref() as &web_sys::EventTarget)
            .remove_event_listener_with_callback(
                &self.trigger.as_str(),
                closure.as_ref().unchecked_ref(),
            )
            .expect("problem removing listener from element");
    }
}

impl<Ms> PartialEq for Listener<Ms> {
    fn eq(&self, other: &Self) -> bool {
        // todo we're only checking the trigger - will miss changes if
        // todo only the fn passed changes!
        self.trigger == other.trigger && self.id == other.id
    }
}

/// UpdateEl is used to distinguish arguments in element-creation macros, and handle
/// each type appropriately.
pub trait UpdateEl<T> {
    // T is the type of thing we're updating; eg attrs, style, events etc.
    fn update(self, el: &mut T);
}

impl<Ms: Clone> UpdateEl<El<Ms>> for Attrs {
    fn update(self, el: &mut El<Ms>) {
        el.attrs = self;
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for &Attrs {
    fn update(self, el: &mut El<Ms>) {
        el.attrs = self.clone();
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for Style {
    fn update(self, el: &mut El<Ms>) {
        el.style = self;
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for &Style {
    fn update(self, el: &mut El<Ms>) {
        el.style = self.clone();
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for Listener<Ms> {
    fn update(self, el: &mut El<Ms>) {
        el.listeners.push(self)
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for Vec<Listener<Ms>> {
    fn update(self, el: &mut El<Ms>) {
        for listener in self.into_iter() {
            el.listeners.push(listener)
        }
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for DidMount {
    fn update(self, el: &mut El<Ms>) {
        el.hooks.did_mount = Some(self.actions)
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for DidUpdate {
    fn update(self, el: &mut El<Ms>) {
        el.hooks.did_update = Some(self.actions)
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for WillUnmount {
    fn update(self, el: &mut El<Ms>) {
        el.hooks.will_unmount = Some(self.actions)
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for &str {
    // This, or some other mechanism seems to work for String too... note sure why.
    fn update(self, el: &mut El<Ms>) {
        el.children.push(El::new_text(self))
        //        el.text = Some(self.into());
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for Vec<El<Ms>> {
    fn update(self, el: &mut El<Ms>) {
        for child in self.into_iter() {
            el.children.push(child);
        }
    }
}

impl<Ms: Clone> UpdateEl<El<Ms>> for El<Ms> {
    fn update(self, el: &mut El<Ms>) {
        el.children.push(self)
    }
}

/// This is intended only to be used for the custom! element macro.
impl<Ms: Clone> UpdateEl<El<Ms>> for Tag {
    fn update(self, el: &mut El<Ms>) {
        el.tag = self;
    }
}

/// Similar to tag population.
macro_rules! make_attrs {
    // Create shortcut macros for any element; populate these functions in this module.
    { $($attr_camel:ident => $attr:expr),+ } => {

        /// The Ev enum restricts element-creation to only valid event names, as defined here:
        /// https://developer.mozilla.org/en-US/docs/Web/Evs
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum At {
            $(
                $attr_camel,
            )+
            Custom(String)
        }

        impl At {
            pub fn as_str(&self) -> &str {
                match self {
                    $ (
                        At::$attr_camel => $attr,
                    ) +
                    At::Custom(val) => &val
                }
            }
        }

        impl From<&str> for At {
            fn from(attr: &str) -> Self {
                match attr {
                    $ (
                          $attr => At::$attr_camel,
                    ) +
                    _ => {
                        At::Custom(attr.to_owned())
                    }
                }
            }
        }
        impl From<String> for At {
            fn from(attr: String) -> Self {
                match attr.as_ref() {
                    $ (
                          $attr => At::$attr_camel,
                    ) +
                    _ => {
                        At::Custom(attr)
                    }
                }
            }
        }

    }
}

/// Comprehensive list: https://www.w3schools.com/tags/ref_attributes.asp
make_attrs! {
    // Missing data-*
    Accept => "accept", AcceptCharset => "accept-charset", AccessKey => "accesskey", Action => "action",
    Alt => "alt", Async => "async", AutoComplete => "autocomplete", AutoFocus => "autofocus",
    AutoPlay => "autoplay", Charset => "charset", Checked => "checked", Cite => "cite", Class => "class",
    Color => "color", Cols => "cols", ColSpan => "colspan", Content => "content", ContentEditable => "contenteditable",
    Controls => "controls", Coords => "coords", Data => "data", DateTime => "datetime", Default => "default",
    Defer => "defer", Dir => "dir", DirName => "dirname", Disabled => "disabled", Download => "download",
    Draggable => "draggable", DropZone => "dropzone", EncType => "enctype", For => "for", Form => "form",
    FormAction => "formaction", Headers => "headers", Height => "height", Hidden => "hidden", High => "high",
    Href => "href", Hreflang => "hreflang", HttpEquiv => "http-equiv", Id => "id", IsMap => "ismap",
    Kind => "kind", Label => "label", Lang => "lang", List => "list", Loop => "loop", Low => "low",
    Max => "max", MaxLength => "maxlength", Media => "media", Method => "method", Min => "min", Multiple => "multiple",
    Muted => "muted", Name => "name", NoValidate => "novalidate", OnAbort => "onabort", OnAfterPrint => "onafterprint",
    OnBeforePrint => "onbeforeprint", OnBeforeUnload => "onbeforeunload", OnBlur => "onblur", OnCanPlay => "oncanplay",
    OnCanPlayThrough => "oncanplaythrough", OnChange => "onchange", OnClick => "onclick", OnContextMenu => "oncontextmenu",
    OnCopy => "oncopy", OnCueChange => "oncuechange", OnCut => "oncut", OnDblClick => "ondblclick",
    OnDrag => "ondrag", OnDragend => "ondragend", OnDragEnter => "ondragenter", OnDragLeave => "ondragleave",
    OnDragOver => "ondragover", OnDragStart => "ondragstart", OnDrop => "ondrop", OnDurationChange => "ondurationchange",
    OnEmptied => "onemptied", OnEnded => "onended", OnError => "onerror", OnFocus => "onfocus",
    OnHashChange => "onhashchange", OnInput => "oninput", OnInvalid => "oninvalid", OnKeyDown => "onkeydown",
    OnKeyPress => "onkeypress", OnKeyUp => "onkeyup", OnLoad => "onload", OnLoadedData => "onloadeddata",
    OnLoadedMetaData => "onloadedmetadata", OnLoadStart => "onloadstart", OnMouseDown => "onmousedown",
    OnMouseMove => "onmousemove", OnMouseOut => "onmouseout", OnMouseOver => "onmouseover", OnMouseUp => "onmouseup",
    OnMouseWheel => "onmousewheel", OnOffline => "onoffline", OnOnline => "ononline", OnPageHide => "onpagehide",
    OnPageShow => "onpageshow", OnPaste => "onpaste", OnPause => "onpause", OnPlay => "onplay",
    OnPlaying => "onplaying", OnPopstate => "onpopstate", OnProgress => "onprogress", OnRateChangen => "onratechange",
    OnRest => "onreset", OnResize => "onresize", OnScroll => "onscroll", OnSearch => "onsearch",
    OnSeeked => "onseeked", OnSeeking => "onseeking", OnSelect => "onselect", OnStalled => "onstalled",
    OnStorage => "onstorage", OnSubmit => "onsubmit", Onsuspend => "onsuspend", OnTimeUpdate => "ontimeupdate",
    OnToggle => "ontoggle", OnUnload => "onunload", OnVolumeChange => "onvolumechange", OnWaiting => "onwaiting",
    OnWheel => "onwheel", Open => "open", Optimum => "optimum", Pattern => "pattern", PlaceHolder => "placeholder",
    Poster => "poster", Preload => "preload", ReadOnly => "readonly", Rel => "rel", Required => "required",
    Reversed => "reversed", Rows => "rows", RowSpan => "rowspan", Sandbox => "sandbox", Scope => "scope",
    Selected => "selected", Shape => "shape", Size => "size", Span => "span", SpellCheck => "spellcheck",
    Src => "src", SrcDoc => "srcdoc", SrcLang => "srclang", SrcSet => "srcset", Start => "start",
    Step => "step", Style => "style", TabIndex => "tabindex", Target => "target", Title => "title",
    Translate => "translate", Type => "type", UseMap => "usemap", Value => "value", Width => "width",
    Wrap => "wrap",

    // Special, for iding dummy els:
    Dummy => "dummy-element"
}

/// Similar to tag population.
/// // Tod: DRY with At (almost identical), Ev, and similar to Tag.
macro_rules! make_styles {
    // Create shortcut macros for any element; populate these functions in this module.
    { $($st_camel:ident => $st:expr),+ } => {

        /// The Ev enum restricts element-creation to only valid event names, as defined here:
        /// https://developer.mozilla.org/en-US/docs/Web/Evs
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum St {
            $(
                $st_camel,
            )+
            Custom(String)
        }

        impl St {
            pub fn as_str(&self) -> &str {
                match self {
                    $ (
                        St::$st_camel => $st,
                    ) +
                    St::Custom(val) => &val
                }
            }
        }

        impl From<&str> for St {
            fn from(st: &str) -> Self {
                match st {
                    $ (
                          $st => St::$st_camel,
                    ) +
                    _ => {
                        crate::log(&format!("Can't find this attribute: {}", st));
                        St::Background
                    }
                }
            }
        }
        impl From<String> for St {
            fn from(st: String) -> Self {
                match st.as_ref() {
                    $ (
                          $st => St::$st_camel,
                    ) +
                    _ => {
                        crate::log(&format!("Can't find this attribute: {}", st));
                        St::Background
                    }
                }
            }
        }

    }
}

// todo finish and implement this.
/// Comprehensive list: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference
/// Most common: https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Properties_Reference
make_styles! {
   AdditiveSymbols => "additive-symbols", AlignContent => "align-content", AlignItems => "align-items",
   AlignSelf => "align-self", All => "all", Angle => "angle", Animation => "animation", AnimationDelay => "animation-delay",
   AnimationDirection => "animation-direction", AnimationDuration => "animation-duration",

   AnimationFillMode => "animation-fill-mode", AnimationIterationCount => "animation-iteration-count",
   AnimationName => "animation-name", AnimationPlayState => "animation-play-state",

   // Most common
   Background => "background", BackgroundAttachment => "background-attachment", BackgroundColor => "background-color",
   BackgroundImage => "background-image", BackgroundPosition => "background-position", BackgroundRepeat => "background-repeat",
   Border => "border", BorderBottom => "border-bottom", BorderBottomColor => "border-bottom-color",
   BorderBottomStyle => "border-bottom-style", BorderBottomWidth => "border-bottom-width", BorderColor => "border-color"


}

/// A thinly-wrapped HashMap holding DOM attributes
#[derive(Clone, Debug, PartialEq)]
pub struct Attrs {
    pub vals: HashMap<At, String>,
}

//impl PartialEq for Attrs {
//    fn eq(&self, other: &Self) -> bool {
//        for (key, val) in &self.vals {
//            if key == "Checked".into() {117
//                continue
//            }
//            match other.vals.get(key).unwrap() {
//                Some(other_val) => {
//                    if val != other_val { return false }
//                },
//                None => return false,
//            }
//        }
//        // todo iter through remaining other keys.
//        true
//    }
//}

impl Attrs {
    pub fn new(vals: HashMap<At, String>) -> Self {
        Self { vals }
    }

    pub fn empty() -> Self {
        Self {
            vals: HashMap::new(),
        }
    }

    /// Convenience function. Ideal when there's one id, and no other attrs.
    /// Generally called with the id! macro.
    pub fn from_id(name: &str) -> Self {
        let mut result = Self::empty();
        result.add(At::Id, name);
        result
    }

    /// Create an HTML-compatible string representation
    pub fn to_string(&self) -> String {
        self.vals
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k.as_str(), v))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Add a new key, value pair
    pub fn add(&mut self, key: At, val: &str) {
        self.vals.insert(key, val.to_string());
    }

    // Add multiple values for a single attribute. Useful for classes.
    pub fn add_multiple(&mut self, key: At, items: Vec<&str>) {
        // We can't loop through self.add, single the value we need is a single,
        // concatonated string.
        self.add(key, &items.join(" "));
    }

    /// Combine with another Attrs; if there's a conflict, use the other one.
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (key, val) in &other.vals {
            result.vals.insert(key.clone(), val.clone());
        }
        result
    }
}

/// Handle Style separately from Attrs, since it commonly involves multiple parts,
/// and has a different semantic meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    // todo enum for key?
    pub vals: HashMap<String, String>,
}

impl Style {
    pub fn new(vals: HashMap<String, String>) -> Self {
        let mut new_vals = HashMap::new();
        for (key, val) in vals.into_iter() {
            // Handle automatic conversion to string with "px" appended, for integers.
            let val_backup = val.clone();
            match val.parse::<u32>() {
                Ok(_) => new_vals.insert(key, val_backup + "px"),
                Err(_) => new_vals.insert(key, val_backup),
            };
        }

        Self { vals: new_vals }
    }

    pub fn empty() -> Self {
        Self {
            vals: HashMap::new(),
        }
    }

    /// Output style as a string, as would be set in the DOM as the attribute value
    /// for 'style'. Eg: "display: flex; font-size: 1.5em"
    pub fn to_string(&self) -> String {
        if self.vals.keys().len() > 0 {
            self.vals
                .iter()
                .map(|(k, v)| format!("{}:{}", k, v))
                .collect::<Vec<_>>()
                .join(";")
        } else {
            String::new()
        }
    }

    pub fn add(&mut self, key: &str, val: &str) {
        self.vals.insert(key.to_string(), val.to_string());
    }

    /// Combine with another Style; if there's a conflict, use the other one.
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (key, val) in &other.vals {
            result.vals.insert(key.clone(), val.clone());
        }
        result
    }
}

/// Similar to tag population.
macro_rules! make_events {
    // Create shortcut macros for any element; populate these functions in this module.
    { $($event_camel:ident => $event:expr),+ } => {

        /// The Ev enum restricts element-creation to only valid event names, as defined here:
        /// https://developer.mozilla.org/en-US/docs/Web/Evs
        #[derive(Clone, Debug, PartialEq)]
        pub enum Ev {
            $(
                $event_camel,
            )+
        }

        impl Ev {
            pub fn as_str(&self) -> &str {
                match self {
                    $ (
                        Ev::$event_camel => $event,
                    ) +
                }
            }
        }

        impl From<&str> for Ev {
            fn from(event: &str) -> Self {
                match event {
                    $ (
                          $event => Ev::$event_camel,
                    ) +
                    _ => {
                        crate::log(&format!("Can't find this event: {}", event));
                        Ev::Click
                    }
                }
            }
        }

        impl From<String> for Ev {
            fn from(event: String) -> Self {
                match event.as_ref(){
                    $ (
                          $event => Ev::$event_camel,
                    ) +
                    _ => {
                        crate::log(&format!("Can't find this event: {}", event));
                        Ev::Click
                    }
                }
            }
        }

        impl ToString for Ev {
            fn to_string( & self ) -> String {
                match self {
                    $ (
                        Ev::$ event_camel => $ event.into(),
                    ) +

                }
            }
        }
    }
}

/// Comprehensive list: https://developer.mozilla.org/en-US/docs/Web/Evs
make_events! {
    Cached => "cached", Error => "error", Abort => "abort", Load => "load", BeforeUnload => "beforeunload",
    Unload => "unload", Online => "online", Offline => "offline", Focus => "focus", Blur => "blur",
    Open => "open", Message => "message", Close => "close", PageHide => "pagehide",
    PageShow => "pageshow", PopState => "popstate", AnimationStart => "animationstart", AnimationEnd => "animationend",
    AnimationIteration => "animationiteration", TransitionStart => "transtionstart", TransitionEnd => "transitionend",
    TranstionRun => "transitionrun",

    Rest => "rest", Submit => "submit", BeforePrint => "beforeprint", AfterPrint => "afterprint",
    CompositionStart => "compositionstart", CompositionUpdate => "compositionupdate", CompositionEnd => "compositionend",

    FullScreenChange => "fullscreenchange", FullScreenError => "fullscreenerror", Resize => "resize",
    Scroll => "scroll", Cut => "cut", Copy => "copy", Paste => "paste",

    KeyDown => "keydown",
    KeyPress => "keypress", AuxClick => "auxclick", Click => "click", ContextMenu => "contextmenu", DblClick => "dblclick",
    MouseDown => "mousedown", MouseEnter => "mouseenter", MouseLeave => "mouseleave",
    MouseMove => "mousemove", MouseOver => "mouseover", MouseOut => "mouseout", MouseUp => "mouseup",
    PointerLockChange => "pointerlockchange", PointerLockError => "pointerlockerror", Select => "select",
    Wheel => "wheel",

    Drag => "drag", DragEnd => "dragend", DragEnter => "dragenter", DragStart => "dragstart", DragLeave => "dragleave",
    DragOver => "dragover", Drop => "drop",

    AudioProcess => "audioprocess", CanPlay => "canplay", CanPlayThrough => "canplaythrough", Complete => "complete",
    DurationChange => "durationchange", Emptied => "emptied", Ended => "ended", LoadedData => "loadeddata",
    LoadedMetaData => "loadedmetadata", Pause => "pause", Play => "play", Playing => "playing", RateChagne => "ratechange",
    Seeked => "seeked", Seeking => "seeking", Stalled => "stalled", Suspend => "suspend", TimeUpdate => "timeupdate",
    VolumeChange => "volumechange",

    // todo finish this

    Change => "change",

    Input => "input"
}

/// Populate tags using a macro, to reduce code repetition.
/// The tag enum primarily exists to ensure only valid elements are allowed.
/// We leave out non-body tags like html, meta, title, and body.
macro_rules! make_tags {
    // Create shortcut macros for any element; populate these functions in this module.
    { $($tag_camel:ident => $tag:expr),+ } => {

        /// The Tag enum restricts element-creation to only valid tags, as defined here:
        /// https://developer.mozilla.org/en-US/docs/Web/HTML/Element
        #[derive(Clone, Debug, PartialEq)]
        pub enum Tag {
            Custom(String),
            $(
                $tag_camel,
            )+
        }

        impl Tag {
            pub fn as_str(&self) -> &str {
                match self {
                    Tag::Custom(name) => &name,
                    $ (
                        Tag::$tag_camel => $tag,
                    ) +
                }
            }
        }
    }
}

/// Comprehensive lists:
/// - https://developer.mozilla.org/en-US/docs/Web/HTML/Element
/// - https://developer.mozilla.org/en-US/docs/Web/SVG/Element
/// Grouped here by category on Mozilla's pages, linked above.
make_tags! {
    Address => "address", Article => "article", Aside => "aside", Footer => "footer",
    Header => "header", H1 => "h1",
    H2 => "h2", H3 => "h3", H4 => "h4", H5 => "h5", H6 => "h6",
    Hgroup => "hgroup", Main => "main", Nav => "nav", Section => "section",

    BlockQuote => "blockquote",
    Dd => "dd", Dir => "dir", Div => "div", Dl => "dl", Dt => "dt", FigCaption => "figcaption", Figure => "figure",
    Hr => "hr", Li => "li", Ol => "ol", P => "p", Pre => "pre", Ul => "ul",

    A => "a", Abbr => "abbr",
    B => "b", Bdi => "bdi", Bdo => "bdo", Br => "br", Cite => "cite", Code => "code", Data => "data",
    Dfn => "dfn", Em => "em", I => "i", Kbd => "kbd", Mark => "mark", Q => "q", Rb => "rb",
    Rp => "rp", Rt => "rt", Rtc => "rtc", Ruby => "ruby", S => "s", Samp => "samp", Small => "small",
    Span => "span", Strong => "strong", Sub => "sub", Sup => "sup", Time => "time", Tt => "tt",
    U => "u", Var => "var", Wbr => "wbr",

    Area => "area", Audio => "audio", Img => "img", Map => "map", Track => "track", Video => "video",

    Applet => "applet", Embed => "embed", Iframe => "iframe",
    NoEmbed => "noembed", Object => "object", Param => "param", Picture => "picture", Source => "source",

    Canvas => "canvas", NoScript => "noscript", Script => "Script",

    Del => "del", Ins => "ins",

    Caption => "caption", Col => "col", ColGroup => "colgroup", Table => "table", Tbody => "tbody",
    Td => "td", Tfoot =>"tfoot", Th => "th", Thead => "thead", Tr => "tr",

    Button => "button", DataList => "datalist", FieldSet => "fieldset", Form => "form", Input => "input",
    Label => "label", Legend => "legend", Meter => "meter", OptGroup => "optgroup", Option => "option",
    Output => "output", Progress => "progress", Select => "select", TextArea => "textarea",

    Details => "details", Dialog => "dialog", Menu => "menu", MenuItem => "menuitem", Summary => "summary",

    Content => "content", Element => "element", Shadow => "shadow", Slot => "slot", Template => "template",

    // -------- SVG Tags -------- //

    // Animation elements
    Animate => "animate", AnimateColor => "animateColor", AnimateMotion => "animateMotion",
    AnimateTransform => "animateTransform", Discard => "discard", Mpath => "mpath", Set => "set",

    // Shape elements
    Circle => "circle", Ellipse => "ellipse", Line => "line", Polygon => "polygon",
    Polyline => "polyline", Rect => "rect", Mesh => "mesh", Path => "path",

    // Container elements
    Defs => "defs", G => "g", Marker => "marker", Mask => "mask", MissingGlyph => "missing-glyph",
    Pattern => "pattern", Svg => "svg", Switch => "switch", Symbol => "symbol", Unknown => "unknown",

    // Descriptive elements
    Desc => "desc", Metadata => "metadata", Title => "title",

    // Filter primitive elements
    FeBlend             => "feBlend",
    FeColorMatrix       => "feColorMatrix",
    FeComponentTransfer => "feComponentTransfer",
    FeComposite         => "feComposite",
    FeConvolveMatrix    => "feConvolveMatrix",
    FeDiffuseLighting   => "feDiffuseLighting",
    FeDisplacementMap   => "feDisplacementMap",
    FeDropShadow        => "feDropShadow",
    FeFlood             => "feFlood",
    FeFuncA             => "feFuncA",
    FeFuncB             => "feFuncB",
    FeFuncG             => "feFuncG",
    FeFuncR             => "feFuncR",
    FeGaussianBlur      => "feGaussianBlur",
    FeImage             => "feImage",
    FeMerge             => "feMerge",
    FeMergeNode         => "feMergeNode",
    FeMorphology        => "feMorphology",
    FeOffset            => "feOffset",
    FeSpecularLighting  => "feSpecularLighting",
    FeTile              => "feTile",
    FeTurbulence        => "feTurbulence",

    // Light source elements
    FeDistantLight => "feDistantLight", FePointLight => "fePointLight",  FeSpotLight => "feSpotLight",

    // Font elements
    Font => "font",
    FontFace => "font-face",
    FontFaceFormat => "font-face-format",
    FontFaceName => "font-face-name",
    FontFaceSrc => "font-face-src",
    FontFaceUri => "font-face-uri",
    HKern => "hkern",
    VKern => "vkern",

    // Gradient elements
    LinearGradient => "linearGradient", MeshGradient => "meshGradient",
    RadialGradient => "radialGradient", Stop => "stop",

    // Graphics elements
    Image => "image",

    // Graphics referencing elements
    Use => "use",

    // Paint server elements
    Hatch => "hatch", SolidColor => "solidcolor",

    // Text content elements
    AltGlyph => "altGlyph", AltGlyphDef => "altGlyphDef", AltGlyphItem => "altGlyphItem", Glyph => "glyph",
    GlyphRef => "glyphRef", TextPath => "textPath", Text => "text", TRef => "tref", TSpan => "tspan",

    // Uncategorized elements
    ClipPath => "clipPath", ColorProfile => "color-profile", Cursor => "cursor", Filter => "filter",
    ForeignObject => "foreignObject", HatchPath => "hatchpath", MeshPatch => "meshpatch", MeshRow => "meshrow",
    Style => "style", View => "view"
}

/// An component in our virtual DOM.
#[derive(Debug)]
pub struct El<Ms: 'static> {
    // Ms is a message type, as in part of TEA.
    // We call this 'El' instead of 'Element' for brevity, and to prevent
    // confusion with web_sys::Element.

    // Core attributes that correspond to the DOM element.
    pub tag: Tag,
    pub attrs: Attrs,
    pub style: Style,
    pub listeners: Vec<Listener<Ms>>,
    // Text should be None unless raw_html or text_node is true.
    pub text: Option<String>,
    pub children: Vec<El<Ms>>,

    // Things that get filled in later, to assist with rendering.
    pub id: Option<u32>, // todo maybe not optional...
    pub nest_level: Option<u32>,
    pub el_ws: Option<web_sys::Node>,

    // todo temp?
    //    pub key: Option<u32>,
    pub raw_html: bool,
    pub text_node: bool,
    pub namespace: Option<Namespace>,

    // Controlled means we keep its text input (value) in sync with the model.
    pub controlled: bool,
    // static: bool
    // static_to_parent: bool
    // ancestors: Vec<u32>  // ids of parent, grandparent etc.

    // Lifecycle hooks
    pub hooks: LifecycleHooks,
}

type HookFn = Box<FnMut(&web_sys::Node)>;

#[derive(Default)]
pub struct LifecycleHooks {
    pub did_mount: Option<HookFn>,
    pub did_update: Option<HookFn>,
    pub will_unmount: Option<HookFn>,
}

fn fmt_hook_fn<T>(h: &Option<T>) -> &'static str {
    match h {
        Some(_) => "Some(.. a dynamic handler ..)",
        None => "None",
    }
}

impl fmt::Debug for LifecycleHooks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LifecycleHooks {{ did_mount:{:?}, did_update:{:?}, will_unmount:{} }}",
            fmt_hook_fn(&self.did_mount),
            fmt_hook_fn(&self.did_update),
            fmt_hook_fn(&self.will_unmount)
        )
    }
}

impl<Ms> El<Ms> {
    /// Create an empty element, specifying only the tag
    pub fn empty(tag: Tag) -> Self {
        Self {
            tag,
            attrs: Attrs::empty(),
            style: Style::empty(),
            listeners: Vec::new(),
            text: None,
            children: Vec::new(),

            id: None,
            nest_level: None,
            el_ws: None,

            raw_html: false,
            text_node: false,
            namespace: None,

            controlled: false,

            // static: false,
            // static_to_parent: false,
            hooks: LifecycleHooks::default(),
        }
    }

    pub fn new_text(text: &str) -> Self {
        let mut result = Self::empty(Tag::Span);
        result.text_node = true;
        result.text = Some(text.into());
        result
    }

    /// Create an empty SVG element, specifying only the tag
    pub fn empty_svg(tag: Tag) -> Self {
        let mut el = El::empty(tag);
        el.namespace = Some(Namespace::Svg);
        el
    }

    /// Create an element that will display markdown from the text you pass to it, as HTML
    pub fn from_markdown(markdown: &str) -> Self {
        let parser = pulldown_cmark::Parser::new(markdown);
        let mut html_text = String::new();
        pulldown_cmark::html::push_html(&mut html_text, parser);
        //
        // todo: Syntect crate is currently bugged with wasm target.
        //        let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
        //        let sr = ss.find_syntax_by_token("rust").unwrap();
        //        let ts = syntect::highlighting::Theme::default();
        //
        //        let replacer = |match_group| {
        //            syntect::html::highlighted_html_for_string(
        //                match_group, &ss, sr, &ts
        //            )
        //        };

        //        let replacer = |match_grp: &regex::Captures| match_grp.name("code").unwrap().as_str();

        //        let re = Regex::new(r"<code>(?P<code>.*?)</code>").expect("Error creating Regex");
        //         re.replace_all(&html_text, replacer);

        //        crate::log(&html_text);
        //
        //        let highlighted_html = syntect::html::highlighted_html_for_string(text,  ss, sr, ts);

        let mut result = Self::empty(Tag::Span);
        result.raw_html = true;
        result.text = Some(html_text);
        result
    }

    /// Create an element that will display raw HTML
    pub fn from_html(html: &str) -> Self {
        let mut result = Self::empty(Tag::Span);
        result.raw_html = true;
        result.text = Some(html.into());
        result
    }

    /// Add a new child to the element
    pub fn add_child(&mut self, element: El<Ms>) {
        self.children.push(element);
    }

    /// Add an attribute (eg class, or href)
    pub fn add_attr(&mut self, key: String, val: String) {
        self.attrs.vals.insert(key.into(), val);
    }

    /// Add a new style (eg display, or height)
    pub fn add_style(&mut self, key: String, val: String) {
        self.style.vals.insert(key, val);
    }

    /// Replace the element's text node. (ie between the HTML tags)
    pub fn set_text(&mut self, text: &str) {
        self.text = Some(text.into())
    }

    /// Output the HTML of this node, including all its children, recursively.
    fn _html(&self) -> String {
        let text = self.text.clone().unwrap_or_default();

        let opening = String::from("<")
            + self.tag.as_str()
            + &self.attrs.to_string()
            + " style=\""
            + &self.style.to_string()
            + ">\n";

        let inner = self
            .children
            .iter()
            .fold(String::new(), |result, child| result + &child._html());

        let closing = String::from("\n</") + self.tag.as_str() + ">";

        opening + &text + &inner + &closing
    }

    /// This is used to provide access to el_ws while recursively appending children to it.
    pub fn quick_clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            attrs: Attrs::empty(),
            style: Style::empty(),
            listeners: Vec::new(),
            text: None,
            children: Vec::new(),
            //            key: None,
            id: None,
            nest_level: None,
            el_ws: self.el_ws.clone(),
            raw_html: self.raw_html,
            text_node: self.text_node,
            namespace: self.namespace.clone(),
            controlled: self.controlled,
            hooks: LifecycleHooks::default(),
        }
    }

    /// Dummy elements are used when logic must return an El due to the type
    /// system, but we don't want to render anything.
    pub fn is_dummy(&self) -> bool {
        if let Tag::Del = self.tag {
            if self.attrs.vals.get(&At::Dummy).is_some() {
                return true;
            }
        }
        false
    }
}

/// Allow the user to clone their Els. Note that there's no easy way to clone the
/// closures within listeners or lifestyle hooks, so we ommit them.
impl<Ms> Clone for El<Ms> {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            attrs: self.attrs.clone(),
            style: self.style.clone(),
            text: self.text.clone(),
            children: self.children.clone(),
            //            key: self.key,
            id: self.id,
            nest_level: self.nest_level,
            el_ws: self.el_ws.clone(),
            listeners: Vec::new(),
            raw_html: self.raw_html,
            text_node: self.text_node,
            namespace: self.namespace.clone(),
            controlled: self.controlled,
            hooks: LifecycleHooks::default(),
        }
    }
}

impl<Ms> PartialEq for El<Ms> {
    fn eq(&self, other: &Self) -> bool {
        // todo Again, note that the listeners check only checks triggers.
        // Don't check children.
        self.tag == other.tag &&
        self.attrs == other.attrs &&
        self.style == other.style &&
        self.text == other.text &&
        self.listeners == other.listeners &&
        // TOdo not sure how nest-level should be used. Is it a given based on
        // todo how we iterate? Sanity-check for now.
        self.nest_level == other.nest_level
    }
}

//impl <Ms: Clone + 'static>crate::vdom::DomEl<Ms> for El<Ms> {
//    type Tg = Tag;
//    type At = Attrs;
//    type St = Style;
//    type Ls = Listener<Ms>;
//    type Tx = String;
//
//    fn tag(self) -> Tag {
//        self.tag
//    }
//    fn attrs(self) -> Attrs {
//        self.attrs
//    }
//    fn style(self) -> Style {
//        self.style
//    }
//    fn listeners(self) -> Vec<Listener<Ms>> {
//        self.listeners
//    }
//    fn text(self) -> Option<String> {
//        self.text
//    }
//    fn children(self) -> Vec<Self> {
//        self.children
//    }
//    fn did_mount(self) -> Option<Box<FnMut(&web_sys::Element)>> {
//        self.did_mount
//    }
//    fn did_update(self) -> Option<Box<FnMut(&web_sys::Element)>> {
//        self.did_mount
//    }
//    fn will_unmount(self) -> Option<Box<FnMut(&web_sys::Element)>> {
//        self.did_mount
//    }
//    fn websys_el(self) -> Option<web_sys::Element> {
//        self.el_ws
//    }
//    fn id(self) -> Option<u32> {
//        self.id
//    }
//    fn raw_html(self) -> bool {
//        self.raw_html
//    }
//        fn namespace(self) -> Option<Namespace> {
//        self.namespace
//    }
//
//    fn empty(self) -> Self {
//        self.empty()
//    }
//
//    fn set_id(&mut self, id: Option<u32>) {
//        self.id = id
//    }
//    fn set_websys_el(&mut self, el_ws: Option<web_sys::Element>) {
//        self.el_ws = el_ws
//}

//    fn make_websys_el(&mut self, document: &web_sys::Document) -> web_sys::Element {
//        crate::websys_bridge::make_websys_el(self, document)
//    }
//}

pub struct DidMount {
    actions: Box<FnMut(&web_sys::Node)>,
}

pub struct DidUpdate {
    actions: Box<FnMut(&web_sys::Node)>,
}

pub struct WillUnmount {
    actions: Box<FnMut(&web_sys::Node)>,
}

/// Aconstructor for DidMount, to be used in the API
pub fn did_mount(mut actions: impl FnMut(&web_sys::Node) + 'static) -> DidMount {
    let closure = move |el: &web_sys::Node| actions(el);
    DidMount {
        actions: Box::new(closure),
    }
}

/// A constructor for DidUpdate, to be used in the API
pub fn did_update(mut actions: impl FnMut(&web_sys::Node) + 'static) -> DidUpdate {
    let closure = move |el: &web_sys::Node| actions(el);
    DidUpdate {
        actions: Box::new(closure),
    }
}

/// A constructor for WillUnmount, to be used in the API
pub fn will_unmount(mut actions: impl FnMut(&web_sys::Node) + 'static) -> WillUnmount {
    let closure = move |el: &web_sys::Node| actions(el);
    WillUnmount {
        actions: Box::new(closure),
    }
}

#[cfg(test)]
use crate as seed;
// required for macros to work.
//    use crate::prelude::*;
use super::*;
use crate::{attrs, div, h1, p, section, span};
//use wasm_bindgen_test::*;  // todo suddenly error about undec type/mod
//use wasm_bindgen_test::wasm_bindgen_test_configure;
pub mod tests {
    //    wasm_bindgen_test_configure!(run_in_browser);

    #[derive(Clone)]
    enum Msg {
        Placeholder,
    }

    // todo now that we use text nodes, same problem as nested
    //    #[wasm_bindgen_test]
    //    pub fn single() {
    //        let expected = "<div>test</div>";
    //
    //        let mut el: El<Msg> = div!["test"];
    //        crate::vdom::setup_els(&crate::util::document(), &mut el, 0, 0);
    //        assert_eq!(expected, el.el_ws.unwrap()
    //            .dyn_ref::<web_sys::Element>().unwrap()
    //            .outer_html());
    //    }

    // todo children are not showing up not sure why.
    //    #[wasm_bindgen_test]
    //    pub fn nested() {
    //        let expected = "<section><div><div><h1>huge success</h1></div><p>\
    //        I'm making a note here</p></div><span>This is a triumph</span></section>";
    //
    //        let mut el: El<Msg> = section![
    //            div![
    //                div![
    //                    h1![ "huge success" ]
    //                ],
    //                p![ "I'm making a note here" ]
    //            ],
    //            span![ "This is a triumph" ]
    //        ];
    //
    //        crate::vdom::setup_els(&crate::util::document(), &mut el, 0, 0);
    ////        assert_eq!(expected, el.el_ws.unwrap().first_element_child().unwrap().outer_html());
    //        assert_eq!(expected, el.el_ws.unwrap().outer_html());
    //    }

    // todo now that we use text nodes, same problem as nested
    //    #[wasm_bindgen_test]
    //    pub fn attrs() {
    //        let expected = "<section src=\"https://seed-rs.org\" class=\"biochemistry\">ok</section>";
    //        let expected2 = "<section class=\"biochemistry\" src=\"https://seed-rs.org\">ok</section>";
    //
    //        let mut el: El<Msg> = section![
    //            attrs! {"class" => "biochemistry"; "src" => "https://seed-rs.org"},
    //            "ok"
    //        ];
    //
    //        crate::vdom::setup_els(&crate::util::document(), &mut el, 0, 0);
    //        assert!(
    //            expected == el.clone().el_ws.unwrap()
    //                .dyn_ref::<web_sys::Element>().unwrap()
    //                .outer_html()
    //                || expected2 == el.el_ws.unwrap()
    //                .dyn_ref::<web_sys::Element>().unwrap()
    //                .outer_html()
    //        );
    //    }
}
