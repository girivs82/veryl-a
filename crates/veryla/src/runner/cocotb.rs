use crate::runner::Runner;
use futures::prelude::*;
use log::{error, info};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::runtime::Runtime;
use tokio_util::codec::{FramedRead, LinesCodec};
use veryla_metadata::Metadata;
use veryla_parser::resource_table::{self, PathId, StrId};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum State {
    Idle,
    Info,
    Warning,
    Error,
    Fatal,
}

pub enum CocotbSource {
    Embed(StrId),
    Include(StrId),
}

pub struct Cocotb {
    source: CocotbSource,
    state: State,
    success: bool,
}

impl Cocotb {
    pub fn new(source: CocotbSource) -> Self {
        Self {
            source,
            state: State::Idle,
            success: true,
        }
    }

    pub fn runner(self) -> Box<dyn Runner> {
        Box::new(self) as Box<dyn Runner>
    }

    fn parse_line(&mut self, line: &str) {
        self.debug(line);

        if !line.starts_with("                ") {
            self.state = State::Idle;
        }

        match self.state {
            State::Idle => {
                if line.ends_with("failed") {
                    self.error(line);
                    self.state = State::Error;
                } else if line.starts_with("     0.00ns INFO") {
                    self.info(line);
                    self.state = State::Info;
                } else if line.starts_with("     0.00ns WARNING") {
                    self.warning(line);
                    self.state = State::Warning;
                } else if line.starts_with("     0.00ns ERROR") {
                    self.error(line);
                    self.state = State::Error;
                } else if line.starts_with("     0.00ns CRITICAL") {
                    self.fatal(line);
                    self.state = State::Fatal;
                }
            }
            State::Info => {
                self.info(line);
            }
            State::Warning => {
                self.warning(line);
            }
            State::Error => {
                self.error(line);
            }
            State::Fatal => {
                self.fatal(line);
            }
        }
    }

    async fn parse(&mut self, mut child: Child) -> Result<()> {
        let stdout = child.stdout.take().unwrap();
        let mut reader = FramedRead::new(stdout, LinesCodec::new());
        while let Some(line) = reader.next().await {
            let line = line.into_diagnostic()?;
            self.parse_line(&line);
        }
        Ok(())
    }
}

impl Runner for Cocotb {
    fn run(
        &mut self,
        metadata: &Metadata,
        test: StrId,
        top: Option<StrId>,
        path: PathId,
        _wave: bool,
    ) -> Result<bool> {
        self.success = true;

        let temp_dir = tempfile::tempdir().into_diagnostic()?;

        info!("Executing test ({})", test);

        let src_path = temp_dir.path().join(format!("{}.py", test));

        match self.source {
            CocotbSource::Embed(x) => {
                let src_text = x.to_string();
                let src_text = src_text.strip_prefix("{{{").unwrap();
                let src_text = src_text.strip_suffix("}}}").unwrap();

                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&src_path)
                    .into_diagnostic()?;
                file.write_all(src_text.as_bytes()).into_diagnostic()?;
                file.flush().into_diagnostic()?;
            }
            CocotbSource::Include(x) => {
                let include_path = resource_table::get_path_value(path).unwrap();
                let include_path = include_path
                    .parent()
                    .unwrap()
                    .join(x.to_string().trim_matches('"'));
                fs::copy(include_path, src_path).into_diagnostic()?;
            }
        }

        let file_list = fs::read_to_string(metadata.filelist_path()).into_diagnostic()?;
        let mut sources = String::new();
        for line in file_list.lines() {
            sources.push_str(&format!("\"{line}\","));
        }
        sources = format!("[{}]", sources.strip_suffix(',').unwrap());

        let module = format!("{}_{}", metadata.project.name, top.unwrap());

        let runner_path = temp_dir.path().join("runner.py");
        let runner_text = format!(
            r#"
import cocotb
import cocotb.runner

sources = {sources}

runner = cocotb.runner.get_runner("verilator")
runner.build(
    verilog_sources=sources,
    hdl_toplevel="{module}",
    always=True,
)

runner.test(
    hdl_toplevel="{module}",
    test_module="{test},",
)
"#
        );

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&runner_path)
            .into_diagnostic()?;
        file.write_all(runner_text.as_bytes()).into_diagnostic()?;
        file.flush().into_diagnostic()?;

        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let compile = Command::new("python3")
                .arg("runner.py")
                .current_dir(temp_dir.path())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .into_diagnostic()
                .wrap_err("Failed to run \"python3\"")?;

            self.parse(compile).await
        })?;

        if self.success {
            info!("Succeeded test ({})", test);
            Ok(true)
        } else {
            error!("Failed test ({})", test);
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "Cocotb"
    }

    fn failure(&mut self) {
        self.success = false;
    }
}
