use crate::screen::Screen;
use crate::theme;
use ratatui::prelude::*;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub(crate) fn ui(frame: &mut Frame, screen: &Screen) {
    let main_ui = Paragraph::new(main_ui_lines(screen)).scroll((screen.scroll, 0));

    if let Some(ref cmd) = screen.command {
        let output_lines = format_command(cmd);

        let layout = Layout::new(
            Direction::Vertical,
            [
                Constraint::Min(1),
                Constraint::Length(output_lines.len() as u16 + 1),
            ],
        )
        .split(frame.size());

        frame.render_widget(main_ui, layout[0]);
        frame.render_widget(command_popup(output_lines), layout[1]);
    } else {
        frame.render_widget(main_ui, frame.size());
    }
}

fn main_ui_lines(screen: &Screen) -> Vec<Line> {
    let mut highlight_depth = None;

    screen
        .collapsed_items_iter()
        .flat_map(|(i, item)| {
            let mut text = if let Some((ref text, style)) = item.display {
                use ansi_to_tui::IntoText;
                let mut text = text.into_text().expect("Couldn't read ansi codes");
                text.patch_style(style);
                text
            } else {
                Text::raw("")
            };

            if screen.is_collapsed(item) {
                text.lines
                    .last_mut()
                    .expect("No last line found")
                    .spans
                    .push("…".into());
            }

            for line in text.lines.iter_mut() {
                let padding = (screen.size.0 as usize).saturating_sub(line.width());

                line.spans.push(Span::styled(
                    " ".repeat(padding),
                    line.spans.first().unwrap().style,
                ));
            }

            if screen.cursor == i {
                highlight_depth = Some(item.depth);
            } else if highlight_depth.is_some_and(|hd| hd >= item.depth) {
                highlight_depth = None;
            }

            if highlight_depth.is_some() {
                for line in &mut text.lines {
                    for span in &mut line.spans {
                        if span.style.bg.is_none() {
                            span.style.bg = Some(if screen.cursor == i {
                                theme::CURRENT_THEME.highlight
                            } else {
                                theme::CURRENT_THEME.dim_highlight
                            })
                        }
                    }
                }
            }

            text
        })
        .collect::<Vec<_>>()
}

fn format_command(cmd: &crate::command::IssuedCommand) -> Vec<Line> {
    Text::styled(
        format!(
            "$ {}{}",
            cmd.args,
            if cmd.finish_acked { "" } else { "..." }
        ),
        Style::new().fg(theme::CURRENT_THEME.command),
    )
    .lines
    .into_iter()
    .chain(
        Text::raw(
            String::from_utf8(cmd.output.clone()).expect("Error turning command output to String"),
        )
        .lines,
    )
    .collect::<Vec<Line>>()
}

fn command_popup(output_lines: Vec<Line>) -> Paragraph {
    Paragraph::new(output_lines).block(
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(theme::CURRENT_THEME.highlight))
            .border_type(ratatui::widgets::BorderType::Plain),
    )
}
