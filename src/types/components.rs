//use std::default::default;
#[cfg(feature = "builder")]
use std::error;
#[cfg(feature = "builder")]
use std::fmt::{self, Display};

#[cfg(feature = "builder")]
use log::warn;
use serde::{Deserialize, Serialize};

#[cfg(feature = "builder")]
use crate::Builder;

use serde_repr::*;
use serde_with::*;

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// Message components are a framework for adding interactive elements to the messages your app or bot sends. They're accessible, customizable, and easy to use.
pub struct MessageComponent {
    /// Type of component
    r#type: ComponentType,
    style: Option<ComponentStyle>,
    label: Option<String>,
    emoji: Option<PartialEmoji>,
    custom_id: Option<String>,
    url: Option<String>,
    disabled: Option<bool>,
    options: Option<Vec<ComponentSelectOption>>,
    placeholder: Option<String>,
    min_values: Option<u8>,
    max_values: Option<u8>,
    components: Option<Vec<MessageComponent>>,

    // Text input specific
    min_length: Option<u16>,
    max_length: Option<u16>,
    required: Option<bool>,
    value: Option<String>,
}

impl Default for MessageComponent {
    fn default() -> Self {
        Self {
            r#type: ComponentType::ActionRow,
            style: None,
            label: None,
            custom_id: None,
            url: None,
            disabled: None,
            emoji: None,
            options: None,
            placeholder: None,
            max_values: None,
            min_values: None,
            components: None,
            min_length: None,
            max_length: None,
            required: None,
            value: None,
        }
    }
}

#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
#[non_exhaustive]
/// Represents a type of component
pub enum ComponentType {
    /// A container for other components
    ActionRow = 1,
    /// A clickable button
    Button = 2,
    /// A select menu for picking from choices
    SelectMenu = 3,
    /// A text input object
    TextInput = 4,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// A partial Emoji structure for Select Menu Options
pub struct PartialEmoji {
    id: u64,
    name: String,
    animated: bool,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
/// An option for select menu options
pub struct ComponentSelectOption {
    label: String,
    value: String,
    description: Option<String>,
    emoji: Option<PartialEmoji>,
    default: Option<bool>,
}

impl ComponentSelectOption {
    /// Sets the option label
    pub fn label(mut self, lab: impl Into<String>) -> Self {
        self.label = lab.into();
        self
    }
    /// Sets the option value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    /// Sets the option description
    pub fn description(mut self, des: impl Into<String>) -> Self {
        self.description = Some(des.into());
        self
    }
    /// Sets the default checked
    pub fn set_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// A select menu
pub struct ComponentSelectMenu {
    custom_id: String,
    options: Vec<ComponentSelectOption>,
    placeholder: Option<String>,
    min_values: u8,
    max_values: u8,
}

impl Default for ComponentSelectMenu {
    fn default() -> Self {
        Self {
            custom_id: String::new(),
            options: Vec::new(),
            placeholder: None,

            // documented defaults
            min_values: 1,
            max_values: 1,
        }
    }
}

impl From<ComponentSelectMenu> for MessageComponent {
    fn from(t: ComponentSelectMenu) -> Self {
        MessageComponent {
            r#type: ComponentType::SelectMenu,
            custom_id: Some(t.custom_id),
            options: Some(t.options),
            placeholder: t.placeholder,
            min_values: Some(t.min_values),
            max_values: Some(t.max_values),
            ..Default::default()
        }
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
/// A button
pub struct ComponentButton {
    style: Option<ComponentButtonStyle>,
    label: Option<String>,
    emoji: Option<PartialEmoji>,
    custom_id: Option<String>,
    url: Option<String>,
    disabled: Option<bool>,
}
#[cfg(feature = "builder")]
impl Default for ComponentButton {
    fn default() -> Self {
        Self {
            style: Some(ComponentButtonStyle::Secondary),
            label: None,
            emoji: None,
            custom_id: None,
            url: None,
            disabled: None,
        }
    }
}
#[cfg(feature = "builder")]
impl From<ComponentButton> for MessageComponent {
    fn from(t: ComponentButton) -> Self {
        let mut a: Option<u8> = None;
        if let Some(b) = t.style{
            a = Some(b.into());
        }
        MessageComponent {
            r#type: ComponentType::Button,
            style: a,
            label: t.label,
            emoji: t.emoji,
            custom_id: t.custom_id,
            url: t.url,
            disabled: t.disabled,
            ..Default::default()
        }
    }
}

/// Alias for generic styling
pub type ComponentStyle = u8;
#[derive(Clone, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
#[non_exhaustive]
/// How a button looks
pub enum ComponentButtonStyle {
    /// blurple
    Primary = 1,
    /// grey
    Secondary = 2,
    /// green
    Success = 3,
    /// red
    Danger = 4,
    /// grey with outgoing link icon
    Link = 5,
}

impl From<ComponentButtonStyle> for ComponentStyle{
    fn from(a: ComponentButtonStyle) -> Self {
        a as ComponentStyle
    }
}

/// Builder for creating a Component Action Row

#[cfg(feature = "builder")]
#[derive(Clone, Default, PartialEq, Debug)]
pub struct ComponentRowBuilder {
    obj: MessageComponent,
}
#[cfg(feature = "builder")]
impl Builder<MessageComponent> for ComponentRowBuilder {
    type Error = std::convert::Infallible;

    fn build(self) -> Result<MessageComponent, Self::Error> {
        Ok(self.obj)
    }
}

#[cfg(feature = "builder")]
impl ComponentRowBuilder {
    /// Add a component to this row
    pub fn add_component(mut self, component: impl Into<MessageComponent>) -> Self{
        match self.obj.components.as_mut() {
            None => {
                self.obj.components = Some(vec![component.into()]);
            }
            Some(c) => {
                c.push(component.into());
            }
        }
        self
    }

    /// Add a button
    pub fn add_button(mut self, button: ComponentButton) -> Self {
        match self.obj.components.as_mut() {
            None => {
                self.obj.components = Some(vec![button.into()]);
            }
            Some(c) => {
                c.push(button.into());
            }
        }
        self
    }

    /// Add a select menu to the row
    pub fn add_select_menu(mut self, menu: ComponentSelectMenu) -> Self {
        match self.obj.components.as_mut() {
            None => {
                self.obj.components = Some(vec![menu.into()]);
            }
            Some(c) => {
                c.push(menu.into());
            }
        }
        self
    }
    /// Add a textbox to the row
    pub fn add_textbox(mut self, textbox: ComponentTextBox) -> Self{
        match self.obj.components.as_mut(){
            None => {
                self.obj.components = Some(vec![textbox.into()]);
            }
            Some(c) =>{
                c.push(textbox.into());
            }
        }
        self
    }

    #[deprecated(since = "0.1.9", note = "Use the `build()` function instead")]
    /// Finish building this row (returns a [`MessageComponent`])
    pub fn finish(self) -> MessageComponent {
        self.obj
    }
}
#[cfg(feature = "builder")]
#[derive(Clone, PartialEq, Debug)]
/// Builder for making an button component
pub struct ComponentButtonBuilder {
    obj: ComponentButton,
}

#[allow(clippy::field_reassign_with_default)]
#[cfg(feature = "builder")]
impl Default for ComponentButtonBuilder {
    fn default() -> Self {
        let ob = ComponentButton::default();
        Self { obj: ob }
    }
}
#[cfg(feature = "builder")]
impl ComponentButtonBuilder {
    /// Finish building this button
    #[deprecated(since = "0.1.9", note = "Use the `build()` function instead")]
    pub fn finish(self) -> ComponentButton {
        match self.obj.clone().style.unwrap() {
            ComponentButtonStyle::Link => {
                if self.obj.url.is_none() {
                    warn!("The button style is set to 'Link', but no url was specified.");
                }
            }
            _ => {
                if self.obj.custom_id.is_none() {
                    warn!("No custom_id was supplied for this button!")
                }
            }
        }
        self.obj
    }

    /// Set the button style. Takes a [`ComponentButtonStyle`]
    pub fn style(mut self, s: &ComponentButtonStyle) -> Self {
        self.obj.style = Some(s.clone());
        self
    }
    /// Set the button label
    pub fn label(mut self, l: impl Into<String>) -> Self {
        let lab = l.into();

        self.obj.label = Some(lab);
        self
    }
    /// Set a custom id (required for all styles except `ComponentButtonStyle::Link`)
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        let i = id.into();

        self.obj.custom_id = Some(i);
        self
    }
    /// Set a URL (required if style is set to `ComponentButtonStyle::Link`)
    pub fn url(mut self, url: impl Into<String>) -> Self {
        let u = url.into();

        self.obj.url = Some(u);
        self
    }
    /// Disables/deactivates a button
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.obj.disabled = Some(disabled);
        self
    }
}

#[cfg(feature = "builder")]
#[derive(Debug)]
/// An error that occurred when building a Component
pub enum ComponentBuilderError {
    /// The component had no specified style
    NoStyle,
    /// The component was a Link without a specified URL
    LinkWithoutUrl,
    /// The component was a Button without a specified custom ID
    NoCustomId,
}

#[cfg(feature = "builder")]
impl Display for ComponentBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentBuilderError::NoStyle => write!(f, "style is none"),
            ComponentBuilderError::LinkWithoutUrl => write!(
                f,
                "the button style is set to 'Link', but no url was specified"
            ),
            ComponentBuilderError::NoCustomId => {
                write!(f, "no custom ID specified for this button")
            }
        }
    }
}

#[cfg(feature = "builder")]
impl error::Error for ComponentBuilderError {}

#[cfg(feature = "builder")]
impl Builder<ComponentButton> for ComponentButtonBuilder {
    type Error = ComponentBuilderError;

    fn build(self) -> Result<ComponentButton, Self::Error> {
        let style = self
            .obj
            .style
            .as_ref()
            .ok_or(ComponentBuilderError::NoStyle)?;

        match style {
            ComponentButtonStyle::Link => {
                if self.obj.url.is_none() {
                    return Err(ComponentBuilderError::LinkWithoutUrl);
                }
            }
            _ => {
                if self.obj.custom_id.is_none() {
                    return Err(ComponentBuilderError::NoCustomId);
                }
            }
        }
        Ok(self.obj)
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Debug, Default)]
/// Builder pattern for creating menu components.
pub struct ComponentSelectMenuBuilder {
    obj: ComponentSelectMenu,
}

#[cfg(feature = "builder")]
impl ComponentSelectMenuBuilder {
    /// The custom developer identifier. **SETTING THIS IS MANDATORY!**
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.obj.custom_id = id.into();
        self
    }

    /// custom placeholder text if nothing is selected, max 100 characters
    pub fn placeholder(mut self, ph: impl Into<String>) -> Self {
        let p = ph.into();

        if p.chars().count() > 100 {
            warn!("Menu placeholder exceeded 100 characters, ignoring");
            return self;
        }
        self.obj.placeholder = Some(p);
        self
    }

    /// Add a menu choice
    pub fn add_option(mut self, opt: ComponentSelectOption) -> Self {
        if self.obj.options.len() >= 25 {
            warn!("This menu already contains 25 elements, ignoring");
            return self;
        }
        self.obj.options.push(opt);
        self
    }

    /// The minimum number of items that must be chosen; default 1, min 0, max 25
    pub fn min_values(mut self, min: impl Into<u8>) -> Self {
        self.obj.min_values = min.into();
        self
    }

    /// the maximum number of items that can be chosen; default 1, max 25
    pub fn max_values(mut self, max: impl Into<u8>) -> Self {
        self.obj.max_values = max.into();
        self
    }
}

#[cfg(feature = "builder")]
#[derive(Debug)]
/// Represents an error that occurred when building a ComponentSelectMenu
pub enum ComponentSelectMenuBuilderError {
    /// There was no Custom ID supplied with this menu
    EmptyCustomId,
    /// There were over 25 options supplied for this menu
    Over25MenuOptions,
    /// There were over 25 min_values supplied for this menu
    Over25MinValues,
    /// There were over 25 max_values supplied for this menu
    Over25MaxValues,
}

#[cfg(feature = "builder")]
impl Display for ComponentSelectMenuBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentSelectMenuBuilderError::EmptyCustomId => write!(f, "custom_id is empty"),
            ComponentSelectMenuBuilderError::Over25MenuOptions => {
                write!(f, "over 25 menu options supplied")
            }
            ComponentSelectMenuBuilderError::Over25MinValues => {
                write!(f, "over 25 min_values options supplied")
            }
            ComponentSelectMenuBuilderError::Over25MaxValues => {
                write!(f, "over 25 max_values options supplied")
            }
        }
    }
}

#[cfg(feature = "builder")]
impl error::Error for ComponentSelectMenuBuilderError {}

#[cfg(feature = "builder")]
impl Builder<ComponentSelectMenu> for ComponentSelectMenuBuilder {
    type Error = ComponentSelectMenuBuilderError;

    fn build(self) -> Result<ComponentSelectMenu, Self::Error> {
        if self.obj.custom_id.is_empty() {
            return Err(ComponentSelectMenuBuilderError::EmptyCustomId);
        }
        if self.obj.options.len() > 25 {
            return Err(ComponentSelectMenuBuilderError::Over25MenuOptions);
        }
        if self.obj.min_values > 25 {
            return Err(ComponentSelectMenuBuilderError::Over25MinValues);
        }
        if self.obj.max_values > 25 {
            return Err(ComponentSelectMenuBuilderError::Over25MaxValues);
        }
        Ok(self.obj)
    }
}


#[derive(Clone, Debug, Default)]
pub enum ComponentTextBoxStyle {
    #[default] Short = 1, 
    Paragraph = 2,
}

impl From<ComponentTextBoxStyle> for ComponentStyle{
    fn from(a: ComponentTextBoxStyle) -> Self {
        a as ComponentStyle
    }
}

#[derive(Clone, Debug, Default)]
/// A textbox component, where users can put text in
pub struct ComponentTextBox {
    placeholder: Option<String>,
    custom_id: String,
    label: String,
    style: ComponentTextBoxStyle,
    min_length: Option<u16>,
    max_length: Option<u16>,
    required: Option<bool>,
}



impl From<ComponentTextBox> for MessageComponent {
    fn from(t: ComponentTextBox) -> Self {
        MessageComponent {
            r#type: ComponentType::TextInput,
            label: Some(t.label),
            custom_id: Some(t.custom_id),
            style: Some(t.style.into()),
            min_length: t.min_length,
            max_length: t.max_length,
            required: t.required,
            ..Default::default()
        }
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Debug, Default)]
/// Build a textbox component
pub struct ComponentTextBoxBuilder {
    obj: ComponentTextBox,
}

#[cfg(feature = "builder")]
impl ComponentTextBoxBuilder {

    /// Sets the custom ID of this text thing. **MANDATORY**
    pub fn custom_id(mut self, id: impl Into<String>) -> Self{
        let pid = id.into();

        self.obj.custom_id = pid;
        self
    }

    /// Set the textbox label. **MANDATORY**
    pub fn label(mut self, label: impl Into<String>) -> Self{
        self.obj.label = label.into();
        self
    }

    /// Sets the style of this textbox. See [`ComponentTextBoxStyle`] for types
    pub fn style(mut self, style: ComponentTextBoxStyle) -> Self{
        self.obj.style = style;
        self
    }

    /// Sets the placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        let placeholder = placeholder.into();
        self.obj.placeholder = Some(placeholder);
        self
    }
    /// Sets the minimum amount of characters that need to be inserted
    pub fn min_length(mut self, minimum_length: u16) -> Self {
        if minimum_length < 1 || minimum_length > 4000 {
            warn!("Minimum length for this text box exceeds the limits, ignoring");
            return self;
        }

        self.obj.min_length = Some(minimum_length);
        self
    }

    /// Sets the maximum amount of characters that may be inserted
    ///
    /// **The maximum settable length is 4000 characters**
    pub fn max_length(mut self, maximum_length: u16) -> Self {
        if maximum_length < 2 || maximum_length > 4000 {
            warn!("Maximum length for this text box exceeds the limits, ignoring");
            return self;
        }

        self.obj.max_length = Some(maximum_length);
        self
    }

    /// Sets if this textbox is required to fill
    pub fn required(mut self, is_required: bool) -> Self {
        self.obj.required = Some(is_required);
        self
    }
}

#[cfg(feature = "builder")]
impl Builder<MessageComponent> for ComponentTextBoxBuilder {
    type Error = ComponentTextBoxBuilderError;

    fn build(self) -> Result<MessageComponent, Self::Error> {

        if self.obj.custom_id.is_empty() {
            return Err(ComponentTextBoxBuilderError::MissingCustomId);
        }
        if self.obj.label.is_empty(){
            return Err(ComponentTextBoxBuilderError::MissingLabel);
        }
        if let Some(ml) = self.obj.min_length.as_ref() {
            if ml > &4000 {
                return Err(ComponentTextBoxBuilderError::MinimumLengthTooHigh);
            }
        }
        if let Some(ml) = self.obj.max_length.as_ref() {
            if ml < &1 {
                return Err(ComponentTextBoxBuilderError::MaximumLengthTooLow);
            }
            if ml > &4000 {
                return Err(ComponentTextBoxBuilderError::MaximumLengthTooHigh);
            }
            if let Some(minl) = self.obj.min_length.as_ref() {
                if minl > ml {
                    return Err(ComponentTextBoxBuilderError::MinimumGreaterThanMaximum);
                }
            }
        }
        Ok(self.obj.into())
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Debug)]
/// Errors that arise when building the textbox component.
pub enum ComponentTextBoxBuilderError {
    /// The required custom id isn't set!
    MissingCustomId,
    /// Missing a label
    MissingLabel,
    /// The minimum length is set too high (> 4000)
    MinimumLengthTooHigh,
    /// The maximum length is set too high (> 4000)
    MaximumLengthTooHigh,
    /// The maximum length is set too low (< 1)
    MaximumLengthTooLow,
    /// The minimum required length is set to a higher value than the set maximum.
    MinimumGreaterThanMaximum,
}

#[cfg(feature = "builder")]
impl error::Error for ComponentTextBoxBuilderError {}

#[cfg(feature = "builder")]
impl Display for ComponentTextBoxBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentTextBoxBuilderError::MinimumLengthTooHigh => {
                write!(f, "Minimum length exceeded 4000 characters")
            }
            ComponentTextBoxBuilderError::MaximumLengthTooHigh => {
                write!(f, "Maximum length exceeded 4000 characters")
            }
            ComponentTextBoxBuilderError::MaximumLengthTooLow => {
                write!(f, "Maximum length is below 1")
            }
            ComponentTextBoxBuilderError::MinimumGreaterThanMaximum => {
                write!(f, "The minimum length is greater than the maximum length")
            }
            ComponentTextBoxBuilderError::MissingCustomId => {
                write!(f, "The required custom id has not been set for this text box")
            }
            ComponentTextBoxBuilderError::MissingLabel => {
                write!(f, "The required label is not set!")
            }
        }
    }
}
