// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use super::{Component, ComponentRender};
use crate::state_store::{Action, State};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    layout::Margin, prelude::Rect, style::{Color, Style, Styled, Stylize}, widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState}
};
use tokio::sync::mpsc::UnboundedSender;
use tokio_serial::SerialPortInfo;
use std::fmt::Debug;

pub struct OutputBox {
    content: Vec<SerialPortInfo>,
}

impl OutputBox {
    pub fn content(&self) -> &Vec<SerialPortInfo> {
        &self.content
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn new<>(
        _state: &State,
        _screen_idx: Option<usize>,
        _action_sender: UnboundedSender<Action>,
    ) -> Self {

        let available_ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(error) => panic!("ports not found!"),
        };

        Self {
            content: available_ports,
        }
    }


    
}

pub struct RenderProperties {
    pub title: String,
    pub area: Rect,
    pub border_color: Color,
    pub show_cursor: bool,
}

impl ComponentRender<RenderProperties> for OutputBox {
    fn render(&self, frame: &mut ratatui::prelude::Frame, properties: RenderProperties) {
        let mut text: String = "".to_owned();

        for n in 0..self.content.len()
        {
            let serial_info =format!("Port[{n}](Name:{:#?}, Type:{:#?}), \n",self.content[n].port_name, self.content[n].port_type);
            text = text + &serial_info; 
            
        }

        let output = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .fg(properties.border_color)
                    .title(properties.title.clone()),
            );

        
    }
}
