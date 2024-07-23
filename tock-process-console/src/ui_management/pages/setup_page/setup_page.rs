// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::{
    sync::mpsc::{channel, Receiver, Sender},
    vec,
};

use crate::{
    board,
    state_store::{Action, BoardConnectionStatus, State},
    ui_management::components::{
        input_box, probe_info, Component, ComponentRender, InputBox, ProbeInfo,
    },
};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};
use probe_rs::probe::{self, list::Lister, Probe};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Margin},
    prelude::Direction,
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Styled, Stylize},
    symbols::scrollbar,
    text::{self, Line, Text},
    widgets::{
        Block, BorderType, Borders, List, ListDirection, ListItem, ListState, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Wrap,
    },
};

use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tokio_serial::{SerialPort, SerialPortInfo, SerialPortType, UsbPortInfo};

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

struct EventHandler {}

/// Struct that handles setup of the console application
pub struct SetupPage {
    input_box: InputBox,
    action_sender: UnboundedSender<Action>,
    properties: Properties,
    scrollbar_state_serial: ListState,
    scrollbar_state_boards: ListState,
    probeinfo_sender: Sender<Vec<String>>,
    probeinfo_receiver: Receiver<Vec<String>>,
    showed_serials: bool,
    showed_boards: bool,
}

impl SetupPage {
    fn set_port(&mut self) {
        let probeinfo = match self.probeinfo_receiver.try_recv() {
            Ok(probeinfo) => probeinfo,
            Err(error) => panic!("{}", error),
        };

        if probeinfo.is_empty() {
            print!("No Boards Found!")
        }

        let mut port_number = 0;

        if self.showed_boards == true {
            port_number = match self.scrollbar_state_boards.selected() {
                Some(num) => num,
                None => panic!("Error selecting the board!"),
            };
        }
        if self.showed_serials == true {
            port_number = match self.scrollbar_state_serial.selected() {
                Some(num) => num,
                None => panic!("Error selecting the board!"),
            };
        }

        let port = probeinfo[port_number].clone();
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
            Err(error) => panic!("ports not found! : {}", error),
        };

        let input_box = InputBox::new(state, screen_idx, action_sender.clone());

        let mut board_scroll_number = 0;
        for n in 0..available_ports.len() {
            if available_ports[n].port_name.starts_with("/dev/ttyACM") {
                board_scroll_number += 1;
            }
        }
        let mut scrollbar_state_serial = ListState::default();
        scrollbar_state_serial.select_first();

        let mut scrollbar_state_boards = ListState::default();
        scrollbar_state_boards.select_first();

        let mut showed_serials = false;
        let mut showed_boards = true;

        let (tx, rx) = channel();

        let probeinfo_sender = tx;
        let probeinfo_receiver = rx;

        SetupPage {
            action_sender: action_sender.clone(),
            input_box,
            properties: Properties::from(state),
            scrollbar_state_serial,
            scrollbar_state_boards,
            probeinfo_sender,
            probeinfo_receiver,
            showed_serials,
            showed_boards,
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

        let available_ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(error) => panic!("ports not found! : {}", error),
        };

        let mut board_scroll_number = 0;
        for n in 0..available_ports.len() {
            if available_ports[n].port_name.starts_with("/dev/ttyACM") {
                board_scroll_number += 1;
            }
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
            KeyCode::Char('a') => {
                if self.showed_serials == false {
                    self.showed_serials = true;
                    self.showed_boards = false;
                    self.scrollbar_state_serial.select_first();
                }
            }
            KeyCode::Char('b') => {
                if self.showed_boards == false {
                    self.showed_serials = false;
                    self.showed_boards = true;
                    self.scrollbar_state_boards.select_first();
                }
            }
            KeyCode::Up => {
                if self.showed_serials == true {
                    self.scrollbar_state_serial.select_previous()
                }
                else if self.showed_boards == true {
                    self.scrollbar_state_boards.select_previous()
                }
            }
            KeyCode::Down => {
                if self.showed_serials == true {
                    self.scrollbar_state_serial.select_next()
                }
                else if self.showed_boards == true {
                    self.scrollbar_state_boards.select_next()
                }
            }
            KeyCode::PageUp => {
                if self.showed_serials == true {
                    self.scrollbar_state_serial.select_previous()
                }
                else if self.showed_boards == true {
                    self.scrollbar_state_boards.select_previous()
                }
            }
            KeyCode::PageDown => {
                if self.showed_serials == true {
                    self.scrollbar_state_serial.select_next()
                }
                else if self.showed_boards == true {
                    self.scrollbar_state_boards.select_next()
                }
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
            scrollbar_state_serial: self.scrollbar_state_serial,
            scrollbar_state_boards: self.scrollbar_state_boards,
            probeinfo_sender: self.probeinfo_sender,
            probeinfo_receiver: self.probeinfo_receiver,
            showed_serials: self.showed_serials,
            showed_boards: self.showed_boards,
        }
    }

    fn handle_mouse_event(&mut self, _event: crossterm::event::MouseEvent) {}
}

impl ComponentRender<()> for SetupPage {
    fn render(&mut self, frame: &mut ratatui::prelude::Frame, _properties: ()) {
        let [_, serial_position_v, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Min(2),
                Constraint::Percentage(35),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };

        let [_, serial_position_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Min(2),
                Constraint::Percentage(18),
            ])
            .split(serial_position_v)
        else {
            panic!("adfikjge")
        };

        let available_ports = match tokio_serial::available_ports() {
            Ok(ports) => ports,
            Err(error) => panic!("ports not found! : {}", error),
        };

        let mut vec_serial: Vec<Text> = vec![];
        let mut vec_boards: Vec<Text> = vec![];
        let mut board_ports: Vec<String> = vec![];
        let mut serial_ports: Vec<String> = vec![];
        for n in 0..available_ports.len() {
            let mut k = 0;
            let product = match &available_ports[n].port_type {
                SerialPortType::UsbPort(usb) => {
                    k = 1;
                    usb.product.clone()
                }
                SerialPortType::PciPort => Some("PciPort".to_string()),
                SerialPortType::BluetoothPort => Some("BluetoothPort".to_string()),
                SerialPortType::Unknown => Some("Unknown".to_string()),
            };

            let temp_serial = format! {"Port[{n}](Name:{:#?}, Type:{}), \n", available_ports[n].port_name, Option::expect(product, "Port type not found!")};
            if k == 1 {
                vec_boards.push(temp_serial.clone().into());
                board_ports.push(available_ports[n].port_name.clone());
            }
            vec_serial.push(temp_serial.into());
            serial_ports.push(available_ports[n].port_name.clone());
        }

        if self.showed_serials == true {
            let list = List::new(vec_serial)
                .style(Style::default().fg(Color::Cyan))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .fg(Color::Yellow)
                        .title(format!(" Serial ports - {} ", available_ports.len()))
                        .title_style(Style::default().fg(Color::Blue)),
                )
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(list, serial_position_h, &mut self.scrollbar_state_serial);
        }

        if self.showed_boards == true {
            let boards_found = vec_boards.len();

            let list = List::new(vec_boards)
                .style(Style::default().fg(Color::Cyan))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .fg(Color::Yellow)
                        .title(format!(" Number of boards found: {}  ", boards_found))
                        .title_style(Style::default().fg(Color::Blue)),
                )
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(" > ")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(list, serial_position_h, &mut self.scrollbar_state_boards);
        }

        if self.showed_boards == true {
            match self.probeinfo_sender.send(board_ports) {
                Ok(_) => {}
                Err(error) => println!("{}", error),
            };
        }
        else if self.showed_serials == true {
            match self.probeinfo_sender.send(serial_ports) {
                Ok(_) => {}
                Err(error) => println!("{}", error),
            };
        }

        let [_, help_text_v, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(65),
                Constraint::Min(2),
                Constraint::Percentage(10),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };

        let [_, help_text_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Min(2),
                Constraint::Percentage(35),
            ])
            .split(help_text_v)
        else {
            panic!("adfikjge")
        };

        let [_, panic_v, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(75),
                Constraint::Min(2),
                Constraint::Percentage(10),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };

        let [_, panic_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Min(2),
                Constraint::Percentage(35),
            ])
            .split(panic_v)
        else {
            panic!("adfikjge")
        };

        let [_, show_text_v, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(69),
                Constraint::Min(2),
                Constraint::Percentage(10),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };

        let [_, show_text_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Min(2),
                Constraint::Percentage(35),
            ])
            .split(show_text_v)
        else {
            panic!("adfikjge")
        };

        if self.showed_boards == true {
            let show_text = Paragraph::new(Text::from("Press A to display all serial ports."));
            frame.render_widget(show_text, show_text_h);
        }
        else if self.showed_serials == true {
            let show_text = Paragraph::new(Text::from("Press B to display all boards found."));
            frame.render_widget(show_text, show_text_h);
        }

        let [_, enter_text_v, _] = *Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(72),
                Constraint::Min(2),
                Constraint::Percentage(10),
            ])
            .split(frame.size())
        else {
            panic!("adfikjge")
        };

        let [_, enter_text_h, _] = *Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(18),
                Constraint::Min(2),
                Constraint::Percentage(35),
            ])
            .split(enter_text_v)
        else {
            panic!("adfikjge")
        };

        let help_text = Paragraph::new(Text::from("Press Enter to select highlighted port."));
        frame.render_widget(help_text, enter_text_h);

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
