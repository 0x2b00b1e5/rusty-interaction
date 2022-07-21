use std::{error::Error, fmt};

use serde::{Deserialize, Serialize};

use super::components::{MessageComponent, ComponentRowBuilder};
use serde_with::*;

#[cfg(feature = "builder")]
use crate::Builder;
#[cfg(feature = "builder")]
use log::warn;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
/// A modal is a popup form formed after an interaction.
/// After sending an [`InteractionResponseType::Modal`], it will send out a form for the user to fill.
/// After filling, Discord will send out an [`InteractionType::ModalSubmit`], where data processing will be done.
pub struct Modal {
    custom_id: String,
    title: String,
    components: Vec<MessageComponent>,
}

impl Modal {
    /// Get custom id
    pub fn get_custom_id(&self) -> String {
        self.custom_id.clone()
    }
    /// Get title
    pub fn get_title(&self) -> String {
        self.title.clone()
    }
    /// Get components
    pub fn get_components(&self) -> Vec<MessageComponent> {
        self.components.clone()
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Debug, Default)]
/// Build a modal
pub struct ModalBuilder {
    obj: Modal,
    comps : ComponentRowBuilder
}

#[cfg(feature = "builder")]
impl ModalBuilder {
    /// Sets the custom_id. This is mandatory!
    /// The custom id may not be more than 100 characters long
    ///
    /// ### Errors
    /// If the supplied custom id is above 100 characters, the library will print out a warning and ignore the request.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        if id.len() > 100 {
            warn!("Exceeding maximum id char count (100), ignoring");
            return self;
        }
        self.obj.custom_id = id;
        self
    }
    /// Sets the title. This is mandatory!
    pub fn title(mut self, title: impl Into<String>) -> Self {
        let title = title.into();
        self.obj.title = title;
        self
    }
    /// Adds a component to the form. You must supply at lease 1 component and no more than 5 components.
    ///
    /// ### Errors
    /// If the component count exceeds 5, this library will print out a warning and ignore the request.
    pub fn add_component(mut self, component: impl Into<MessageComponent>) -> Self {
        self.comps = self.comps.add_component(component);
        self
    }
}

#[cfg(feature = "builder")]
#[derive(Clone, Debug)]
/// Errors when building a modal
pub enum ModalConversionError {
    /// Missing a custom id
    MissingCustomId,
    /// Missing a title
    MissingTitle,
    /// Missing components, modals need atleast **one** component
    MissingComponents,
    /// Too much components. Modals may only have up to five components.
    TooMuchComponents,
}

#[cfg(feature = "builder")]
impl fmt::Display for ModalConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ModalConversionError::MissingCustomId => {
                write!(f, "Missing a custom id for modal!")
            }
            ModalConversionError::MissingTitle => {
                write!(f, "Missing a title for modal!")
            }
            ModalConversionError::MissingComponents => {
                write!(f, "Modal does not contain any components!")
            }
            ModalConversionError::TooMuchComponents => {
                write!(f, "Modal contains too much components!")
            }
        }
    }
}

#[cfg(feature = "builder")]
impl Error for ModalConversionError {}

#[cfg(feature = "builder")]
impl Builder<Modal> for ModalBuilder {
    type Error = ModalConversionError;

    fn build(mut self) -> Result<Modal, Self::Error> {
        if self.obj.custom_id.len() < 1 {
            return Err(ModalConversionError::MissingCustomId);
        }
        if self.obj.title.len() < 1 {
            return Err(ModalConversionError::MissingTitle);
        }
        /*if self.obj.components.len() < 1 {
            return Err(ModalConversionError::MissingComponents);
        }*/
        if self.obj.components.len() > 5 {
            return Err(ModalConversionError::TooMuchComponents);
        }

        if let Ok(v) = self.comps.build(){
            self.obj.components = vec![v];
        }
        else{
            return Err(ModalConversionError::MissingComponents);
        }

        return Ok(self.obj);
    }
}
