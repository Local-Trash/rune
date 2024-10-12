use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;

use current_platform::CURRENT_PLATFORM;
use ratatui::prelude::Rect;
use rune::input;
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use toml::Table;

use crate::{
    action::Action,
    cli::{Cli, CliCommand},
    components::Component,
    config::Config,
    mode::Mode,
    settings::Settings,
    tui,
};

pub struct App {
    pub config: Config,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub should_quit: bool,
    pub should_suspend: bool,
    pub mode: Mode,
    pub command: Option<CliCommand>,
    pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
    pub fn new(cli: Cli) -> Result<Self> {
        let config = Config::new()?;
        let mode = Mode::Home;
        let command = cli.command;

        Ok(Self {
            tick_rate: 4.0,
            frame_rate: 60.0,
            should_quit: false,
            should_suspend: false,
            config,
            command,
            mode,
            last_tick_key_events: Vec::new(),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        match &self.command {
            Some(CliCommand::New(new)) => crate::commands::new::new(new).await?,
            Some(CliCommand::Test) => {
                let input_path = Path::new("../test-game/dist/.rune/input/");
                let binary = std::fs::read(input_path.join("test-game.wasm")).unwrap();
                rune::runtime::test(input_path.to_path_buf(), binary).await;
            }
            Some(CliCommand::Run { release }) => {
                crate::commands::run::run(release).await?;
            }
            Some(CliCommand::Build { release }) => {
                crate::commands::build::build(release).await?;
            }
            Some(CliCommand::Bundle { target, release }) => {
                crate::commands::build::build(release).await?;
                crate::commands::bundle::bundle(target, release).await?;
            }
            Some(CliCommand::Upgrade) => crate::commands::upgrade::upgrade().await?,
            Some(CliCommand::Docs) => crate::commands::docs::docs(&self.config, &self.mode).await?,
            None => {}
        }

        Ok(())
    }
}
