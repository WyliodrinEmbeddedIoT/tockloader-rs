// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use crate::{
    board, state_store::{Action, BoardConnectionStatus, State}, ui_management::components::{input_box, probe_info, Component, ComponentRender, InputBox, ProbeInfo}
};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use probe_rs::probe::{self, list::Lister, Probe};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Margin}, prelude::Direction, style::{Color, Modifier, Style, Stylize}, symbols::scrollbar, text::{self, Line, Text}, widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap}
};

use tokio::sync::mpsc::UnboundedSender;
use tokio_serial::{SerialPort, SerialPortInfo};

struct Properties {
    error_message: Option<String>,
}

impl From<&State> for Properties {
    fn from(state: &State) -> Self {
        let error_message =
            if let BoardConnectionStatus::Errored { err } = &state.board_connection_status {
                Some(err.clone())
            } else {
                None
            };

        Properties { error_message }
    }
}

/// Struct that handles setup of the console application
pub struct SetupPage {
    input_box: InputBox,
    action_sender: UnboundedSender<Action>,
    properties: Properties,
    scrollbar_state: ScrollbarState,
    scroll_position: usize,
}

impl SetupPage {
    fn set_port(&mut self) {
        // should update the port
        if self.input_box.is_empty() {
            return;
        }

        let port = self.input_box.text();
        let _ = self.action_sender.send(Action::ConnectToBoard {
            port: port.to_string(),
        });
    }
}

impl Component for SetupPage {
    fn new(state: &State, screen_idx: Option<usize>, action_sender: UnboundedSender<Action>) -> Self
    where
        Self: Sized,
    {
        let available_ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(error) => panic!("ports not found! : {}",error),
        };

        let input_box = InputBox::new(state, screen_idx, action_sender.clone());

        let mut scroll_position = 0;
        let mut scrollbar_state = ScrollbarState::new(available_ports.len()).position(scroll_position);

        SetupPage {
            action_sender: action_sender.clone(),
            input_box,
            properties: Properties::from(state),
            scrollbar_state,
            scroll_position,
        }
        .update_with_state(state)
    }

    fn name(&self) -> &str {
        "Setup page"
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        self.input_box.handle_key_event(key);

        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Enter => {
                self.set_port();
            }
            KeyCode::Char('c') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    let _ = self.action_sender.send(Action::Exit);
                }
            }
            KeyCode::Up => {
                self.scroll_position = self.scroll_position.saturating_sub(1);
                self.scrollbar_state =
                self.scrollbar_state .position(self.scroll_position);
            }
            KeyCode::Down => {
                self.scroll_position = self.scroll_position.saturating_add(1);
                self.scrollbar_state =
                self.scrollbar_state .position(self.scroll_position);
            }
            KeyCode::PageUp => {
                self.scroll_position = self.scroll_position.saturating_sub(1);
                self.scrollbar_state =
                self.scrollbar_state .position(self.scroll_position);
            }
            KeyCode::PageDown => {
                self.scroll_position = self.scroll_position.saturating_add(1);
                self.scrollbar_state =
                self.scrollbar_state .position(self.scroll_position);
            }
            _ => {}
        }
    }

    fn update_with_state(self, state: &State) -> Self
    where
        Self: Sized,
    {
        Self {
            properties: Properties::from(state),
            input_box: self.input_box,
            action_sender: self.action_sender,
            scrollbar_state: self.scrollbar_state,
            scroll_position: self.scroll_position,
        }
    }

    fn handle_mouse_event(&mut self, _event: crossterm::event::MouseEvent) {}
}

impl ComponentRender<()> for SetupPage {

    fn render(&self, frame: &mut ratatui::prelude::Frame, _properties: ()) {

        let [_, serial_position_v, _] = *Layout::default()
            .horizontal_margin(4)
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(4),
                Constraint::Min(2),
                Constraint::Percentage(70),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };


        let [_, serial_position_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((0)),
                Constraint::Min(2),
                Constraint::Percentage(70),
            ])
            .split(serial_position_v)
        else {
            panic!("adfikjge")
        };

        let available_ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(error) => panic!("ports not found! : {}",error),
        };

        let mut text = "".to_owned();
        for n in 0..available_ports.len()
        {
            let serial_info =format!("Port[{n}](Name:{:#?}, Type:{:#?}), \n",available_ports[n].port_name, available_ports[n].port_type);
            text = text + &serial_info; 
            
        };

        let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .fg(Color::Yellow)
                .title(format!(" Serial ports - {} ",available_ports.len())),
        )
        .scroll((self.scroll_position as u16, 0));
        
        frame.render_widget(paragraph, serial_position_h);

        frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .symbols(scrollbar::VERTICAL)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
            serial_position_h.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut self.scrollbar_state.clone()
        );

        let [_, boards_position_v, _] = *Layout::default()
        .horizontal_margin(4)
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Min(2),
            Constraint::Percentage(46),
        ])
        .split(frame.size())
         else {
            panic!("adfikjge")
         };


        let [_, boards_position_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Min(2),
                Constraint::Percentage(32),
            ])
            .split(boards_position_v)
        else {
            panic!("adfikjge")
        };

        let mut  boards_number = 0;
        let lister = Lister::new();
        let probe_list = lister.list_all();

        let mut nr_probes = probe_list.len();

        let mut probeinfo_list: Vec<ProbeInfo> = vec![];
        
        for n in 0..available_ports.len()
        {
            if available_ports[n].port_name == format!("/dev/ttyACM{boards_number}")
            {
                let probe = ProbeInfo {number: boards_number, port: n, port_name: available_ports[n].port_name.clone(), port_probe: probe_list[nr_probes-1].identifier.clone()};
                
                probeinfo_list.push(probe);
                boards_number += 1;
                nr_probes-=1;
            }
        }

        let mut text_probes: String = "".to_owned();

        for n in 0..probeinfo_list.len()
        {
            let temp_info =format!("{}. Port[{}]: Name:{:?}, Board:{},",probeinfo_list[n].number()+1,probeinfo_list[n].port(),probeinfo_list[n].port_name(), probeinfo_list[n].port_probe());
            text_probes = text_probes + &temp_info; 
        }


        let paragraph = Paragraph::new(format!("   {}", text_probes))
        .style(Style::default().fg(Color::Cyan))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .fg(Color::Yellow)
                .title(format!(" Number of boards found: {} ",probe_list.len())).title_style(Style::default().fg(Color::Blue)),
        );

        frame.render_widget(paragraph, boards_position_h);
        


        let [_, help_text_v, _] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(2),
            Constraint::Percentage(20),
        ])
        .split(frame.size())
        else {
            panic!("adfikjge")
        };



        let [_, help_text_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(2),
                Constraint::Min(2),
                Constraint::Percentage(80),
            ])
            .split(help_text_v)
        else {
            panic!("adfikjge")
        };

        let [_, panic_v, _] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Min(2),
            Constraint::Percentage(5),
        ])
        .split(frame.size())
        else {
            panic!("adfikjge")
        };



        let [_, panic_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Min(2),
                Constraint::Percentage(20),
            ])
            .split(panic_v)
        else {
            panic!("adfikjge")
        };



        let help_text = Paragraph::new(Text::from("Use ▲ ▼ PageUp PageDown to scroll.  "));
        frame.render_widget(help_text, help_text_h);

        let error = if let Some(error) = &self.properties.error_message {
            Text::from(format!("Error: {}", error))
        } else {
            Text::from("")
        };
        let error_message = Paragraph::new(error).wrap(Wrap { trim: true }).style(
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::SLOW_BLINK | Modifier::ITALIC),
        );

        frame.render_widget(error_message, panic_h);
    }
}
