use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use csv::WriterBuilder;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Stylize,
    symbols::{self, border},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Gauge, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget, Wrap,
    },
};
use std::{collections::HashMap, io};

use crate::{DistanceRecord, Stop, visualize, write_csv};

#[derive(Debug, Default)]
pub struct App {
    pub progress: u16,
    pub result: Option<Result<Vec<(f64, Stop, Stop)>, String>>,
    saved_distances: HashMap<usize, DistanceRecord>,

    state: ListState,
    exit: bool,
}

enum Save {
    All,
    None,
    Selected
}

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                self.handle_key_event(key);
            }
        }
        Ok(())
    }

    fn exit(&mut self, save: Save) {
        self.exit = true;

        match save {
            Save::All => {
                if let Some(result) = &self.result {
                    if let Ok(result) = result {
                        let mapped = result.into_iter().map(|(distance, stop_a, stop_b)|DistanceRecord {
                            length: *distance,
                            stop_a_name: stop_a.name.clone(),
                            stop_a_lat: stop_a.lat,
                            stop_a_long: stop_a.long,
                            stop_b_name: stop_b.name.clone(),
                            stop_b_lat: stop_b.lat,
                            stop_b_long: stop_b.long,
                        }).collect::<Vec<DistanceRecord>>();
    
                        write_csv::write_csv(mapped).unwrap()
                    }

                }

            }
            Save::None => {},
            Save::Selected => {
                let values:  Vec<DistanceRecord> = self.saved_distances.values().cloned().collect();

                write_csv::write_csv(values).unwrap()
            }
        }

        
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(Save::Selected),
            KeyCode::Char('x') => self.exit(Save::None),
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Enter => self.visualize(),
            KeyCode::Char(' ') => self.save(),
            KeyCode::Char('a') => self.exit(Save::All),
            _ => {}
        }
    }

    fn select_none(&mut self) {
        self.state.select(None);
    }

    fn select_next(&mut self) {
        self.state.select_next();
    }
    fn select_previous(&mut self) {
        self.state.select_previous();
    }

    fn select_first(&mut self) {
        self.state.select_first();
    }

    fn select_last(&mut self) {
        self.state.select_last();
    }

    fn visualize(&mut self) {
        if let (Some(i), Some(result)) = (self.state.selected(), &self.result) {
            if let Ok(result) = result {
                let (_, stop_a, stop_b) = result[i].clone();
                let _ = visualize::visualize_distance(&stop_a, &stop_b);
            }
        }
    }

    fn save(&mut self) {
        if let (Some(i), Some(result)) = (self.state.selected(), &self.result) {
            if let Ok(result) = result {
                let (distance, stop_a, stop_b) = result[i].clone();

                if self.saved_distances.remove(&i).is_none() {
                    self.saved_distances.insert(i, DistanceRecord {
                        length: distance,
                        stop_a_name: stop_a.name,
                        stop_a_lat: stop_a.lat,
                        stop_a_long: stop_a.long,
                        stop_b_name: stop_b.name,
                        stop_b_lat: stop_b.lat,
                        stop_b_long: stop_b.long,
                    });
                }
            }
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        if let Some(result) = self.result.clone() {
            match result {
                Ok(distances) => {
                    let items: Vec<ListItem> = distances
                        .into_iter()
                        .enumerate()
                        .map(|(i, (distance, stop_a, stop_b))| {
                            let save_string = if self.saved_distances.get(&i).is_some() {
                                "☑"
                            } else {
                                " "
                            };

                            let line = Line::from(vec![
                                format!("{} {:2}. ", 
                                    save_string ,
                                    i + 1,
                                ).gray(),
                                format!("{:6.2}m ", distance * 1000.0).yellow(),
                                stop_a.name.dark_gray(),
                                " to ".into(),
                                stop_b.name.dark_gray(),
                            ]);

                            ListItem::from(line)
                        })
                        .collect();

                    let list = List::new(items)
                        .highlight_symbol(">")
                        .highlight_spacing(HighlightSpacing::Always)
                        .block(Block::bordered());

                    StatefulWidget::render(list, area, buf, &mut self.state);
                }
                Err(e) => {
                    let error = Line::from(vec!["Error: ".red(), e.into()]);
                    let error_text = Text::from(Line::from(error));

                    Paragraph::new(error_text).render(area, buf);
                }
            }
        }
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let info = if let (Some(i), Some(result)) = (self.state.selected(), self.result.clone()) {
            match result {
                Ok(stops) => {
                    let (distance, stop_a, stop_b) = &stops[i];

                    let distance_in_meters = distance * 1000.0;

                    vec![
                        Line::from(vec![
                            format!("{}. ", i + 1).into(),
                            format!("{:.2}m\n", distance_in_meters).yellow().into(),
                        ]),
                        format!("{} #{:?}", stop_a.name, stop_a.stop_number).into(),
                        format!("\t\tat {},{}", stop_a.lat, stop_a.long)
                            .dark_gray()
                            .into(),
                        format!("{} #{:?}", stop_b.name, stop_b.stop_number).into(),
                        format!("\t\tat {},{}", stop_b.lat, stop_b.long)
                            .dark_gray()
                            .into(),
                        Line::from(vec![" Visualise ".into(), "<Enter> ".blue().bold()])
                            .alignment(Alignment::Right),
                        Line::from(vec![" Save ".into(), "<Space> ".blue().bold()])
                            .alignment(Alignment::Right),
                    ]
                }
                Err(_) => vec![Line::from("")],
            }
        } else {
            vec![Line::from("Nothing selected...")]
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Stops").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .padding(Padding::horizontal(1));

        Paragraph::new(info)
            .block(block)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl Widget for &mut App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let instructions = Line::from(vec![
            " Save and Quit ".into(), "<Q> ".blue().bold(),
            " Quit without Saving ".into(), "<X> ".blue().bold(),
            " Save All and Quit ".into(), "<A> ".blue().bold(),
        
        ]);

        use Constraint::{Length, Percentage};
        let layout = Layout::vertical([Length(1), Length(1), Percentage(100), Length(1)]);
        let [header_area, gauge_area, main_area, footer_area] = layout.areas(area);

        let layout = Layout::horizontal([Percentage(50), Percentage(50)]);
        let [list_area, item_area] = layout.areas(main_area);

        let title = Line::from("The Most Useless Tram Stop");
        let title_text = Text::from(Line::from(title));

        Paragraph::new(title_text)
            .centered()
            .render(header_area, buf);

        Paragraph::new(instructions).render(footer_area, buf);

        Gauge::default()
            .percent(self.progress)
            .render(gauge_area, buf);

        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
    }
}
