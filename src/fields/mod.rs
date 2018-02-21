//! Includes `form's` building blocks, `fields`.
use cursive::view::AnyView;
use cursive::views;
use serde_json::value::Value;
use std::rc::Rc;
use validators::Validator;

mod autocomplete;
mod checkbox;
mod multiselect;
mod text;

pub use self::autocomplete::Autocomplete;
pub use self::checkbox::Checkbox;
pub use self::multiselect::Multiselect;
pub use self::text::Text;

/// Covers communication from `Field` to `Widget`.
pub trait WidgetManager {
    /// Builds container `view` with placeholders for `help`, `value`, `error`.
    fn build_widget(&self, label: &str, help: &str, initial: &str) -> Box<AnyView>;
    /// Gets `value` from widget.
    fn get_value(&self, view: &AnyView) -> String;
    /// Sets `error` on widget.
    fn set_error(&self, view: &mut AnyView, error: &str);
    /// Builds a `value` view
    fn build_value_view(&self, value: &str) -> Box<AnyView>;
}

/// Building block for `Form`s which stores `data` and `Widget`.
#[derive(Clone)]
pub struct Field<W: WidgetManager, T> {
    label: String,
    help: String,
    initial: T,
    validators: Vec<Rc<Validator>>,
    widget_manager: W,
}

impl<W: WidgetManager, T> Field<W, T> {
    /// Creates a new `Field` with given `label`, `widget_manager`, `initial`.
    pub fn new<IS: Into<String>>(label: IS, widget_manager: W, initial: T) -> Self {
        Field {
            label: label.into(),
            help: "".into(),
            initial: initial,
            validators: vec![],
            widget_manager: widget_manager,
        }
    }
    /// Sets `help` message for `field`.
    pub fn help<IS: Into<String>>(mut self, msg: IS) -> Self {
        self.help = msg.into();
        self
    }
    /// Append `validator`.
    pub fn validator<V: Validator + 'static>(mut self, validator: V) -> Self {
        self.validators.push(Rc::new(validator));
        self
    }
}

/// Covers communication from `Form` to `Field`.
pub trait FormField {
    /// Builds `widget` representing this `field`.
    fn build_widget(&self) -> Box<AnyView>;
    /// Validates `data`.
    fn validate(&self, data: &str) -> Result<Value, String>;
    /// Gets `field`'s label.
    fn get_label(&self) -> &str;
    /// Gets manager which controlls `widget`.
    fn get_widget_manager(&self) -> &WidgetManager;
}

fn format_annotation(label: &str, help: &str) -> String {
    if help.len() > 0 {
        format!("{:20}: {}", label, help)
    } else {
        format!("{:20}", label)
    }
}

/// Widget layout where `label` and `help` are in the same line.
pub fn label_with_help_layout(view: Box<AnyView>, label: &str, help: &str) -> Box<AnyView> {
    let text = format_annotation(label, help);
    let widget = views::LinearLayout::vertical()
        .child(views::TextView::new(text))
        .child(view)
        .child(views::TextView::new(""))
        .child(views::DummyView);

    Box::new(widget)
}
