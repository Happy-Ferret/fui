//! Contains `form` related concetps like `FormView`.
use std::rc::Rc;
use std::collections::HashMap;

use clap;
use cursive::Cursive;
use cursive::event::{Callback, Event, EventResult, Key, MouseButton, MouseEvent};
use cursive::view::{View, ViewWrapper};
use cursive::views::{Dialog, DialogFocus, LinearLayout};
use serde_json::map::Map;
use serde_json::value::Value;

use fields::FormField;

type OnSubmit = Option<Rc<Fn(&mut Cursive, Value)>>;
type OnCancel = Option<Rc<Fn(&mut Cursive)>>;

/// Aggregates `fields` and handles process of `submitting` (or `canceling`).
pub struct FormView {
    view: Dialog,

    fields: Vec<Box<FormField>>,
    on_submit: OnSubmit,
    on_cancel: OnCancel,
}
impl FormView {
    /// Creates a new `FormView` with two buttons `submit` and `cancel`.
    pub fn new() -> Self {
        let layout = Dialog::new()
            .content(LinearLayout::vertical())
            .button("Cancel", |_| {})
            .button("Submit (Ctrl+f)", |_| {});
        FormView {
            view: layout,
            fields: Vec::new(),
            on_submit: None,
            on_cancel: None,
        }
    }

    /// Appends `field` to field list.
    pub fn field<V: FormField + 'static>(mut self, field: V) -> Self {
        let widget = field.build_widget();
        self.view
            .get_content_mut()
            .as_any_mut()
            .downcast_mut::<LinearLayout>()
            .unwrap()
            .add_child(widget);
        self.fields.push(Box::new(field));
        self
    }

    /// Sets the function to be called when submit is triggered.
    pub fn set_on_submit<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive, Value) + 'static,
    {
        self.on_submit = Some(Rc::new(callback));
    }

    /// Sets the function to be called when submit is triggered.
    ///
    /// Chainable variant.
    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Cursive, Value) + 'static,
    {
        self.set_on_submit(callback);
        self
    }

    /// Sets the function to be called when cancel is triggered.
    pub fn set_on_cancel<F>(&mut self, callback: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.on_cancel = Some(Rc::new(callback));
    }

    /// Sets the function to be called when cancel is triggered.
    ///
    /// Chainable variant.
    pub fn on_cancel<F>(mut self, callback: F) -> Self
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.set_on_cancel(callback);
        self
    }

    /// Translates form's fields to [clap::Arg]
    ///
    /// [clap::Arg]: ../../clap/struct.Arg.html
    pub fn fields2clap_args(&self) -> Vec<clap::Arg> {
        let mut args = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            let arg = field.clap_arg();
            args.push(arg);
        }
        return args;
    }

    /// Translates [clap::ArgMatches] to [serde_json::Value] based on fields.
    ///
    /// [clap::ArgMatches]: ../../clap/struct.ArgMatches.html
    /// [serde_json::Value]: ../../serde_json/enum.Value.html
    pub fn clap_arg_matches2value(&self, arg_matches: &clap::ArgMatches) -> Value {
        let mut form_data = Map::with_capacity(self.fields.len());
        for field in self.fields.iter() {
            let data = field.clap_args2str(&arg_matches);
            match field.validate(data.as_ref()) {
                Ok(v) => {
                    form_data.insert(field.get_label().to_string(), v);
                }
                Err(e) => {
                    let msg = format!("ERROR: {:?}", e);
                    eprintln!("{}", msg);
                }
            }
        }
        Value::Object(form_data)
    }

    fn validate(&self) -> Result<Value, HashMap<String, String>> {
        let mut data = Map::with_capacity(self.fields.len());
        let mut errors = HashMap::with_capacity(self.fields.len());

        for (idx, field) in self.fields.iter().enumerate() {
            let view = self.view
                .get_content()
                .as_any()
                .downcast_ref::<LinearLayout>()
                .unwrap()
                .get_child(idx)
                .unwrap();
            let value = field.get_widget_manager().get_value(view);
            let label = field.get_label();
            match field.validate(value.as_ref()) {
                Ok(v) => {
                    data.insert(label.to_owned(), v);
                }
                Err(e) => {
                    errors.insert(label.to_owned(), e.to_owned());
                }
            }
        }

        if errors.is_empty() {
            Ok(Value::Object(data))
        } else {
            Err(errors)
        }
    }

    fn event_submit(&mut self) -> EventResult {
        match self.validate() {
            Ok(data_map) => {
                let opt_cb = self.on_submit
                    .clone()
                    .map(|cb| Callback::from_fn(move |c| cb(c, data_map.clone())));
                EventResult::Consumed(opt_cb)
            }
            Err(errors) => {
                // TODO: the event focus next required/invalid field?
                for (idx, field) in self.fields.iter().enumerate() {
                    let label = field.get_label();
                    let e = errors.get(label).map(|x| x.as_ref()).unwrap_or("");
                    // can't call method which returns suitable view because of ownership
                    //  * such method would get &mut self
                    //  * self.field gets &self
                    //  so this clash of &mut and &, illegal
                    //  possible solution is to use clone on WidgetManager (needs implementation)
                    //  or
                    //  form should only call field.validate and rest would be handled by field
                    //  which should solve this issue?
                    let mut view = self.view
                        .get_content_mut()
                        .as_any_mut()
                        .downcast_mut::<LinearLayout>()
                        .unwrap()
                        .get_child_mut(idx)
                        .unwrap();
                    field.get_widget_manager().set_error(view, e);
                }
                EventResult::Consumed(None)
            }
        }
    }

    fn event_cancel(&mut self) -> EventResult {
        let cb = self.on_cancel
            .clone()
            .map(|cb| Callback::from_fn(move |c| cb(c)));
        EventResult::Consumed(cb)
    }

    /// Sets `title` of the form on the top of it
    pub fn title(mut self, title: &str) -> Self {
        self.view.set_title(title);
        self
    }
}

impl ViewWrapper for FormView {
    wrap_impl!(self.view: Dialog);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Mouse {
                offset: _,
                position: _,
                event: MouseEvent::Press(btn),
            } => {
                if btn == MouseButton::Left {
                    self.with_view_mut(|v| v.on_event(event))
                        .unwrap_or(EventResult::Ignored);
                    match self.view.focus() {
                        DialogFocus::Button(0) => self.event_cancel(),
                        DialogFocus::Button(1) => self.event_submit(),
                        _ => EventResult::Ignored,
                    }
                } else {
                    EventResult::Ignored
                }
            }
            Event::Key(Key::Enter) => match self.view.focus() {
                DialogFocus::Button(0) => self.event_cancel(),
                DialogFocus::Button(1) => self.event_submit(),
                _ => self.with_view_mut(|v| v.on_event(event))
                    .unwrap_or(EventResult::Ignored),
            },
            // TODO: ctlr+enter binding?
            Event::CtrlChar('f') => self.event_submit(),
            _ => {
                // default behaviour from ViewWrapper
                self.with_view_mut(|v| v.on_event(event))
                    .unwrap_or(EventResult::Ignored)
            }
        }
    }
}
