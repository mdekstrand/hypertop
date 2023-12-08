//! View code.

use anyhow::Result;
use ratatui::prelude::*;

use crate::{backend::MonitorBackend, MonitorState};

use self::{banner::render_banner, quicklook::render_quicklook};

mod banner;
mod meter;
mod quicklook;

pub fn render_screen<B>(frame: &mut Frame, state: &MonitorState<B>) -> Result<()>
where
    B: MonitorBackend,
{
    let layout = Layout::new()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Min(0),
        ])
        .split(frame.size());
    render_banner(frame, state, layout[0])?;
    render_quicklook(frame, state, layout[2])?;
    Ok(())
}
