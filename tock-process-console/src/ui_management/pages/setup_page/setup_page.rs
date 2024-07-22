// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use std::sync::mpsc::{channel, Receiver, Sender};

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
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    symbols::scrollbar,
    text::{self, Line, Text},
    widgets::{
        Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Wrap,
    },
};

use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tokio_serial::{SerialPort, SerialPortInfo, UsbPortInfo};

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
    scrollbar_state_serial: ScrollbarState,
    scroll_position_serial: usize,
    scrollbar_state_boards: ScrollbarState,
    scroll_position_boards: usize,
    scroll_serial: i16,
    scroll_boards: i16,
    probeinfo_sender: Sender<Vec<ProbeInfo>>,
    probeinfo_receiver: Receiver<Vec<ProbeInfo>>,
    // show_serial:,
    //show_boards:,
}

impl SetupPage {
    fn set_port(&mut self) {
        // should update the port

        //TODO SOMEHOW PASS PROBEINFO_LIST FROM THE RENDER FN
        //INPUT BOX HAS TO BECOME PROBEINFO_LIST USE self.SCROLL_BOARDS TO GET THE RIGHT value from probeinfo_list

        let probeinfo = self.probeinfo_receiver.try_recv().unwrap();
        // {
        //     Ok(probeinfo) => probeinfo,
        //     Err(error) => println!("{}", error)
        // };

        if probeinfo.is_empty() {
            print!("No Boards Found!")
        }

        let port = &probeinfo[self.scroll_boards as usize];
        let _ = self.action_sender.send(Action::ConnectToBoard {
            port: port.port_name().to_string(),
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
        let mut scroll_position_serial = 0;
        let mut scrollbar_state_serial =
            ScrollbarState::new(available_ports.len() - 14).position(scroll_position_serial);

        let mut scroll_position_boards = 0;
        let mut scrollbar_state_boards =
            ScrollbarState::new(board_scroll_number - 1).position(scroll_position_boards);

        let mut scroll_serial = 0;
        let mut scroll_boards = 0;

        let (tx, rx) = channel();

        let probeinfo_sender = tx;
        let probeinfo_receiver = rx;

        SetupPage {
            action_sender: action_sender.clone(),
            input_box,
            properties: Properties::from(state),
            scrollbar_state_serial,
            scroll_position_serial,
            scrollbar_state_boards,
            scroll_position_boards,
            scroll_serial,
            scroll_boards,
            probeinfo_sender,
            probeinfo_receiver,
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
            KeyCode::Up => {
                if self.scroll_boards > 0 {
                    self.scroll_position_boards = self.scroll_position_boards.saturating_sub(1);
                    self.scrollbar_state_boards = self
                        .scrollbar_state_boards
                        .position(self.scroll_position_boards);
                    self.scroll_boards -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll_boards < (board_scroll_number - 1) {
                    self.scroll_position_boards = self.scroll_position_boards.saturating_add(1);
                    self.scrollbar_state_boards = self
                        .scrollbar_state_boards
                        .position(self.scroll_position_boards);
                    self.scroll_boards += 1;
                }
            }
            KeyCode::PageUp => {
                if self.scroll_serial > 0 {
                    self.scroll_position_serial = self.scroll_position_serial.saturating_sub(1);
                    self.scrollbar_state_serial = self
                        .scrollbar_state_serial
                        .position(self.scroll_position_serial);
                    self.scroll_serial -= 1;
                }
            }
            KeyCode::PageDown => {
                if self.scroll_serial < (available_ports.len() - 14).try_into().unwrap() {
                    self.scroll_position_serial = self.scroll_position_serial.saturating_add(1);
                    self.scrollbar_state_serial = self
                        .scrollbar_state_serial
                        .position(self.scroll_position_serial);
                    self.scroll_serial += 1;
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
            scroll_position_serial: self.scroll_position_serial,
            scrollbar_state_boards: self.scrollbar_state_boards,
            scroll_position_boards: self.scroll_position_boards,
            scroll_serial: self.scroll_serial,
            scroll_boards: self.scroll_boards,
            probeinfo_sender: self.probeinfo_sender,
            probeinfo_receiver: self.probeinfo_receiver,
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
            Err(error) => panic!("ports not found! : {}", error),
        };

        let something = available_ports[0].port_type.clone();

        let mut text = "".to_owned();
        for n in 0..available_ports.len() {
            let serial_info = format!(
                "Port[{n}](Name:{:#?}, Type:{:#?}), \n",
                available_ports[n].port_name, available_ports[n].port_type
            );
            text = text + &serial_info;
        }

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .fg(Color::Yellow)
                    .title(format!(" Serial ports - {} ", available_ports.len())),
            )
            .scroll((self.scroll_position_serial as u16, 0));

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
            &mut self.scrollbar_state_serial.clone(),
        );

        let [_, boards_position_v, _] = *Layout::default()
            .horizontal_margin(4)
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(48),
                Constraint::Min(2),
                Constraint::Percentage(35),
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

        let mut boards_number = 0;
        let lister = Lister::new();
        let probe_list = lister.list_all();

        let mut nr_probes = probe_list.len();

        let mut probeinfo_list: Vec<ProbeInfo> = vec![];

        let mut text_probes: String = "".to_owned();

        let mut usb_boards_number = 0;

        for n in 0..available_ports.len() {
            let probe: ProbeInfo;
            if available_ports[n].port_name.starts_with("/dev/ttyACM")
            //|| available_ports[n].port_name.starts_with("/dev/ttyUSB")
            {
                if boards_number < probe_list.len() {
                    probe = ProbeInfo {
                        number: boards_number,
                        port: n,
                        port_name: available_ports[n].port_name.clone(),
                        port_probe: probe_list[nr_probes - 1].identifier.clone(),
                    };
                    nr_probes -= 1;
                } else {
                    probe = ProbeInfo {
                        number: boards_number,
                        port: n,
                        port_name: available_ports[n].port_name.clone(),
                        port_probe: "\"Unknown\"".to_owned(),
                    };
                }

                probeinfo_list.push(probe);
                boards_number += 1;
            } else if available_ports[n].port_name.starts_with("/dev/ttyUSB") {
                probe = ProbeInfo {
                    number: boards_number,
                    port: n,
                    port_name: available_ports[n].port_name.clone(),
                    port_probe: "\"Unknown\"".to_owned(),
                };
                probeinfo_list.push(probe);
                usb_boards_number += 1;
            }
        }

        for n in 0..probeinfo_list.len() {
            let temp_info = format!(
                " > {}. Port[{}]: Name:{:?}, Board:{},\n",
                probeinfo_list[n].number() + 1,
                probeinfo_list[n].port(),
                probeinfo_list[n].port_name(),
                probeinfo_list[n].port_probe()
            );
            text_probes = text_probes + &temp_info;
        }

        let paragraph = Paragraph::new(format!("{}", text_probes))
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .fg(Color::Yellow)
                    .title(format!(
                        " Number of boards found: {} ",
                        boards_number + usb_boards_number
                    ))
                    .title_style(Style::default().fg(Color::Blue)),
            )
            .scroll((self.scroll_position_boards as u16, 0));

        frame.render_widget(paragraph, boards_position_h);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .symbols(scrollbar::VERTICAL)
                .begin_symbol(Some(""))
                .end_symbol(Some("")),
            boards_position_h.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut self.scrollbar_state_boards.clone(),
        );

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

        match self.probeinfo_sender.send(probeinfo_list) {
            Ok(_) => {}
            Err(error) => println!("{}", error),
        };
    }
}
