// use ratatui::{
//     layout::{Constraint, Margin, Rect},
//     style::{Modifier, Style, Stylize},
//     text::Text,
//     widgets::{
//         Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table,
//     },
//     Frame,
// };

// const INFO_TEXT: [&str; 1] = [
//     "(Esc) quit | (↑) move up | (↓) move down"
// ];

// fn render_ticks(&mut self, frame: &mut Frame, area: Rect) {
//     let tick = 0;

//     let info_footer = Paragraph::new(format!("current tick: {}", tick))
//         .style(
//             Style::new()
//                 .fg(self.colors.row_fg)
//                 .bg(self.colors.buffer_bg)
//                 .white(),
//         )
//         .centered()
//         .block(
//             Block::bordered()
//                 .border_type(BorderType::Double)
//                 .border_style(Style::new().fg(self.colors.footer_border_color)),
//         );
//     frame.render_widget(info_footer, area);
// }

// fn render_table(&mut self, frame: &mut Frame, area: Rect) {
//     let header_style = Style::default()
//         .fg(self.colors.header_fg)
//         .bg(self.colors.header_bg);
//     let selected_row_style = Style::default()
//         .add_modifier(Modifier::REVERSED)
//         .fg(self.colors.selected_row_style_fg);

//     let header = ["Name", "Issue", "llamacpp_address", "last_update", "idle_slots", "processing_slots"]
//         .into_iter()
//         .map(Cell::from)
//         .collect::<Row>()
//         .style(header_style)
//         .height(1)
//         .white();

//     let rows = self.items.iter().enumerate().map(|(_i, agent)| {
//         let color = self.colors.normal_row_color;

//         let item = agent.ref_array();
//         item.into_iter()
//             .map(|content| Cell::from(Text::from(format!("\n{content}\n")).white()))
//             .collect::<Row>()
//             .style(Style::new().fg(self.colors.row_fg).bg(color))
//             .height(4)
//     });

//     let bar = " █ ";
//     let t = Table::new(
//         rows,
//         [
//             Constraint::Min(self.longest_item_lens.0),
//             Constraint::Min(self.longest_item_lens.1),
//             Constraint::Min(self.longest_item_lens.2),
//             Constraint::Min(self.longest_item_lens.3),
//             Constraint::Min(self.longest_item_lens.4),
//             Constraint::Min(self.longest_item_lens.5),
//         ],
//     )
//     .header(header)
//     .row_highlight_style(selected_row_style)
//     .highlight_symbol(Text::from(vec![
//         "".into(),
//         bar.into(),
//         bar.into(),
//         "".into(),
//     ]))
//     .bg(self.colors.buffer_bg)
//     .highlight_spacing(HighlightSpacing::Always)
//     .column_spacing(10);
//     frame.render_stateful_widget(t, area, &mut self.state);
// }

// fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
//     frame.render_stateful_widget(
//         Scrollbar::default()
//             .orientation(ScrollbarOrientation::VerticalRight)
//             .begin_symbol(None)
//             .end_symbol(None),
//         area.inner(Margin {
//             vertical: 1,
//             horizontal: 1,
//         }),
//         &mut self.scroll_state,
//     );
// }

// fn render_footer(&self, frame: &mut Frame, area: Rect) {
//     let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
//         .style(
//             Style::new()
//                 .fg(self.colors.row_fg)
//                 .bg(self.colors.buffer_bg)
//                 .white(),
//         )
//         .centered()
//         .block(
//             Block::bordered()
//                 .border_type(BorderType::Double)
//                 .border_style(Style::new().fg(self.colors.footer_border_color)),
//         );
//     frame.render_widget(info_footer, area);
// }
