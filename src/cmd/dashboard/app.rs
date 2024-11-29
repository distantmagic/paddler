const ITEM_HEIGHT: usize = 6;
const INFO_TEXT: [&str; 1] = [
    "(Esc) quit | (↑) move up | (↓) move down"
];

use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState};
use ratatui::{DefaultTerminal, Frame};
use io::Result as ioResult;

use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::errors::result::Result;

use super::ui::TableColors;

pub struct App {
    pub state: TableState,
    items: Vec<UpstreamPeer>,
    pub longest_item_lens: (u16, u16, u16, u16, u16, u16),
    pub scroll_state: ScrollbarState,
    pub colors: TableColors,
}

impl App {
    pub fn new(upstream_peer_pool: UpstreamPeerPool) -> Result<Self> {
        let agents = upstream_peer_pool.agents.read().map(|agents_guard| agents_guard.clone())?;
        
        Ok(Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(agents.clone())?,
            scroll_state: ScrollbarState::new((agents.len() - 1) * ITEM_HEIGHT),
            colors: TableColors::new(),
            items: agents,
        })
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new();
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> ioResult<()> {
        loop {
            terminal.try_draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                        KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) -> ioResult<()> {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(3), Constraint::Length(3)]);
        let rects = vertical.split(frame.area());

        self.set_colors();

        self.render_table(frame, rects[0])?;
        self.render_scrollbar(frame, rects[0]);
        self.render_ticks(frame, rects[1]);
        self.render_footer(frame, rects[2]);

        Ok(())
    }

    fn render_ticks(&mut self, frame: &mut Frame, area: Rect) {
        let tick = 0;
    
        let info_footer = Paragraph::new(format!("current tick: {}", tick))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg)
                    .white(),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
    
    fn render_table(&mut self, frame: &mut Frame, area: Rect) -> ioResult<()> {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);
    
        let header = ["Name", "Issue", "Llamacpp address", "Last update", "Idle slots", "Processing slots"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1)
            .white();
    
        let rows = self.items.iter().enumerate().map(|(_i, agent)| {
            let color = self.colors.normal_row_color;
            let mut items: [String; 6] = Default::default();

            match ref_array(agent.clone()) {
                Ok(array) => items = array,
                _ => ()
            }

            items.into_iter()
                .map(|content| Cell::from(Text::from(format!("\n{content}\n")).white()))
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg).bg(color))
                .height(4)
        });
        
        let bar = " █ ";
        let t = Table::new(
            rows,
            [
                Constraint::Min(self.longest_item_lens.0),
                Constraint::Min(self.longest_item_lens.1),
                Constraint::Min(self.longest_item_lens.2),
                Constraint::Min(self.longest_item_lens.3),
                Constraint::Min(self.longest_item_lens.4),
                Constraint::Min(self.longest_item_lens.5),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .highlight_symbol(Text::from(vec![
            "".into(),
            bar.into(),
            bar.into(),
            "".into(),
        ]))
        .bg(self.colors.buffer_bg)
        .highlight_spacing(HighlightSpacing::Always)
        .column_spacing(10);
        frame.render_stateful_widget(t, area, &mut self.state);

        Ok(())
    }
    
    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
    
    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
            .style(
                Style::new()
                    .fg(self.colors.row_fg)
                    .bg(self.colors.buffer_bg)
                    .white(),
            )
            .centered()
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .border_style(Style::new().fg(self.colors.footer_border_color)),
            );
        frame.render_widget(info_footer, area);
    }
    
}

fn constraint_len_calculator(items: Vec<UpstreamPeer>) -> Result<(u16, u16, u16, u16, u16, u16)> {
    let mut name = 0;
    for item in &items {
        if let Some(agent_name) = item.agent_name.clone() {
            if agent_name.len() > name {
                name += agent_name.len()
            }
        }
    }

    let mut error = 0;
    for item in &items {
        if let Some(agent_error) = item.error.clone() {
            if agent_error.len() > error {
                error += agent_error.len()
            }
        }
    }

    let mut addr = 0;
    for item in &items {
        if item.external_llamacpp_addr.to_string().len() > addr {
            addr += item.external_llamacpp_addr.to_string().len()
        }
    }

    let mut slots_idle = 0;
    for item in &items {
        if item.slots_idle.to_string().len() > slots_idle {
            slots_idle += item.slots_idle.to_string().len()
        }
    }

    let mut slots_processing = 0;
    for item in &items {
        if item.slots_processing.to_string().len() > slots_processing {
            slots_processing += item.slots_processing.to_string().len()
        }
    }

    let mut last_update = 0;
    for item in &items {
        if systemtime_strftime(item.last_update)?.len() > last_update {
            last_update += systemtime_strftime(item.last_update)?.len()
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    Ok((
        name as u16,
        error as u16,
        last_update as u16,
        addr as u16,
        slots_idle as u16,
        slots_processing as u16,
    ))
}

fn systemtime_strftime(dt: SystemTime) -> Result<String> {
    let date_as_secs = dt.duration_since(UNIX_EPOCH)?.as_secs().to_string();

    return Ok(date_as_secs);   
}

pub fn ref_array(peer: UpstreamPeer) -> Result<[String; 6]> {
    let has_issue = match peer.error.clone() {
        Some(issue) => issue,
        None => String::from("None"),
    };

    let has_name = match peer.agent_name.clone() {
        Some(issue) => issue,
        None => String::from(""),
    };

    let date_as_string = systemtime_strftime(peer.last_update)?;

    Ok([
        has_name,
        has_issue,
        peer.external_llamacpp_addr.to_string().clone(),
        date_as_string,
        peer.slots_idle.to_string(),
        peer.slots_processing.to_string(),
    ])
}
