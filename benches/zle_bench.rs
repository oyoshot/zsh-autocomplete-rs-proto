//! Native ZLE measurements. Run with `cargo bench --bench zle_bench -- --help`.
use clap::Parser;
use serde::Serialize;
use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[path = "zle/pty.rs"]
mod pty;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type Sample = BTreeMap<String, f64>;
type Samples = BTreeMap<String, Vec<Sample>>;
type Results = BTreeMap<String, Samples>;
static INTERRUPTED: AtomicBool = AtomicBool::new(false);
extern "C" fn on_signal(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}
fn interrupted() -> Result<()> {
    if INTERRUPTED.load(Ordering::Relaxed) {
        Err("benchmark interrupted".into())
    } else {
        Ok(())
    }
}

#[derive(Parser)]
#[command(
    about = "Compare Zsh adapters with a shared binary and isolated HOME/runtime. Requires Zsh and Git; Linux/macOS."
)]
struct Args {
    /// Baseline entry point with adjacent helpers (defaults to the current adapter)
    baseline: Option<PathBuf>,
    #[arg(long, default_value = "target/release/zsh-autocomplete-rs")]
    binary: PathBuf,
    #[arg(long, default_value_t = 15, value_parser = clap::value_parser!(u32).range(3..))]
    samples: u32,
    #[arg(long, default_value_t = 3)]
    warmups: u32,
    #[arg(long, default_value = "keys", value_parser = ["keys", "startup"])]
    workload: String,
    #[arg(long, default_value = "zsh")]
    zsh: PathBuf,
    /// Startup only
    #[arg(long, default_value = "both", value_parser = ["plain", "compiled", "both"])]
    format: String,
    /// Startup only; cold means no running daemon, not cold filesystem caches
    #[arg(long, default_value = "reused", value_parser = ["reused", "cold", "both"])]
    daemon: String,
    /// Startup only
    #[arg(long, default_value = "all", value_parser = ["off", "enter", "space", "both", "all"])]
    expansion: String,
    #[arg(long, default_value = "/tmp/zacrs-zle-bench.toml")]
    output: PathBuf,
    // Cargo supplies this to harness=false benchmarks.
    #[arg(long, hide = true)]
    bench: bool,
}

/// Bound setup/cleanup commands too; daemon descendants may retain stdout.
fn checked(command: &mut Command) -> Result<String> {
    let description = format!(
        "{:?} {:?}",
        command.get_program(),
        command.get_args().collect::<Vec<_>>()
    );
    let mut log = tempfile::tempfile()?;
    let mut child = command
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log.try_clone()?))
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("command timed out: {description}").into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    log.rewind()?;
    let mut output = String::new();
    log.read_to_string(&mut output)?;
    if !status.success() {
        return Err(format!("{description} failed: {output}").into());
    }
    Ok(output.trim().to_owned())
}

struct Session {
    home: tempfile::TempDir,
    env: BTreeMap<OsString, OsString>,
    binary: PathBuf,
    zsh: PathBuf,
}
impl Session {
    fn new(binary: PathBuf, zsh: PathBuf) -> Result<Self> {
        // Keep the socket path below the macOS sockaddr_un limit.
        let home = tempfile::Builder::new()
            .prefix("zacrs-zle.")
            .tempdir_in("/tmp")?;
        let root = home.path();
        fs::create_dir_all(root.join("config/zacrs"))?;
        fs::create_dir(root.join("run"))?;
        let mut env = BTreeMap::new();
        for (name, path) in [
            ("HOME", root.to_owned()),
            ("ZDOTDIR", root.to_owned()),
            ("XDG_CONFIG_HOME", root.join("config")),
            ("XDG_CACHE_HOME", root.join("cache")),
            ("XDG_DATA_HOME", root.join("data")),
            ("XDG_STATE_HOME", root.join("state")),
            ("XDG_RUNTIME_DIR", root.join("run")),
            ("ZACRS_BIN", binary.clone()),
        ] {
            env.insert(name.into(), path.into_os_string());
        }
        env.insert("PATH".into(), std::env::var_os("PATH").unwrap_or_default());
        env.insert("TERM".into(), "xterm-256color".into());
        env.insert("LC_ALL".into(), "C".into());
        Ok(Self {
            home,
            env,
            binary,
            zsh,
        })
    }
    fn command(&self, program: impl AsRef<OsStr>) -> Command {
        let mut command = Command::new(program);
        command.env_clear().envs(&self.env);
        command
    }
    fn daemon(&self, action: &str) -> Result<()> {
        checked(self.command(&self.binary).args(["daemon", action]))?;
        Ok(())
    }
    fn config(&self, enter: bool, space: bool) -> Result<()> {
        fs::write(
            self.home.path().join("config/zacrs/config.toml"),
            format!(
                "[completion]\nexpand_abbreviation_on_enter = {enter}\nexpand_abbreviation_on_space = {space}\n[abbreviations]\ngs = \"git status\"\n"
            ),
        )?;
        Ok(())
    }
    fn shell(&self, plugin: &Path, mode: &str, enter: bool, space: bool) -> Result<pty::Shell> {
        pty::Shell::spawn(
            self.command(&self.zsh)
                .arg("-di")
                .env("BENCH_PLUGIN", plugin)
                .env("BENCH_MODE", mode)
                .env("BENCH_ENTER", if enter { "1" } else { "0" })
                .env("BENCH_SPACE", if space { "1" } else { "0" }),
        )
    }
    fn stage(&self, entry: &Path, name: &str, form: &str) -> Result<PathBuf> {
        let directory = self.home.path().join(name).join(form);
        fs::create_dir_all(&directory)?;
        for source in sources(entry)? {
            fs::copy(&source, directory.join(source.file_name().unwrap()))?;
        }
        let plugin = directory.join(entry.file_name().ok_or("missing entry filename")?);
        if !plugin.is_file() {
            return Err(format!("missing adapter: {}", entry.display()).into());
        }
        if form == "compiled" {
            checked(
                self.command(&self.zsh)
                    .args([
                        "-dfc",
                        "for f in \"$1\"/*.zsh; do zcompile -U \"$f\" || exit; done",
                        "_",
                    ])
                    .arg(&directory),
            )?;
        }
        Ok(plugin)
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        if let Err(error) = self.daemon("stop") {
            eprintln!("private daemon cleanup: {error}");
        }
    }
}
fn sources(entry: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for item in fs::read_dir(entry.parent().ok_or("entry has no parent")?)? {
        let path = item?.path();
        if path.extension().is_some_and(|ext| ext == "zsh") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
fn choices<'a>(value: &'a str, all: &str, variants: &[&'a str]) -> Vec<&'a str> {
    if value == all {
        variants.to_vec()
    } else {
        vec![value]
    }
}

fn startup(
    args: &Args,
    session: &Session,
    adapters: &BTreeMap<String, PathBuf>,
) -> Result<Results> {
    let formats = choices(&args.format, "both", &["plain", "compiled"]);
    let lifecycles = choices(&args.daemon, "both", &["reused", "cold"]);
    let expansions = choices(&args.expansion, "all", &["off", "enter", "space", "both"]);
    let mut staged = BTreeMap::new();
    for (name, entry) in adapters {
        for form in &formats {
            staged.insert((name.as_str(), *form), session.stage(entry, name, form)?);
        }
    }
    checked(session.command(&session.zsh).args([
        "-dfc",
        "autoload -Uz compinit; compinit -i -d \"$HOME/compdump\"",
    ]))?;
    fs::write(
        session.home.path().join(".zshrc"),
        include_str!("zle/startup.zsh"),
    )?;
    let mut results = Results::new();
    for expansion in expansions {
        let enter = matches!(expansion, "enter" | "both");
        let space = matches!(expansion, "space" | "both");
        session.config(enter, space)?;
        for lifecycle in &lifecycles {
            session.daemon("stop")?;
            if *lifecycle == "reused" {
                session.daemon("ensure")?;
            }
            for form in &formats {
                let group = format!("{expansion}/{lifecycle}/{form}");
                let mut samples = Samples::new();
                let order = ["none", "baseline", "candidate"];
                for rep in 0..u64::from(args.samples) + u64::from(args.warmups) {
                    for offset in 0..3 {
                        interrupted()?;
                        let name = order[(rep as usize + offset) % 3];
                        if *lifecycle == "cold"
                            && session.home.path().join("run/zacrs.sock").exists()
                        {
                            return Err("cold sample has a running daemon".into());
                        }
                        let plugin = if name == "none" {
                            Path::new("")
                        } else {
                            &staged[&(name, *form)]
                        };
                        let measured = {
                            let mut shell = session.shell(plugin, "", enter, space)?;
                            let source = shell.ready()?;
                            Sample::from([
                                ("source_ms".into(), source),
                                (
                                    "ready_ms".into(),
                                    shell.started.elapsed().as_secs_f64() * 1000.0,
                                ),
                            ])
                        };
                        if *lifecycle == "cold" {
                            session.daemon("stop")?;
                        }
                        if rep >= u64::from(args.warmups) {
                            samples.entry(name.into()).or_default().push(measured);
                        }
                    }
                }
                println!("{group}: {} samples per adapter", args.samples);
                results.insert(group, samples);
            }
            session.daemon("stop")?;
        }
    }
    Ok(results)
}

fn keys(args: &Args, session: &Session, adapters: &BTreeMap<String, PathBuf>) -> Result<Results> {
    let staged = adapters
        .iter()
        .map(|(name, entry)| Ok((name.clone(), session.stage(entry, name, "plain")?)))
        .collect::<Result<BTreeMap<_, _>>>()?;
    fs::write(
        session.home.path().join(".zshrc"),
        include_str!("zle/keys.zsh"),
    )?;
    session.config(false, false)?;
    session.daemon("ensure")?;
    let mut results = Results::new();
    for mode in ["off", "on", "hot-disabled"] {
        let mut samples = Samples::new();
        for rep in 0..u64::from(args.samples) + u64::from(args.warmups) {
            // Retain the original key benchmark order and workloads.
            for (name, plugin) in &staged {
                interrupted()?;
                session.config(false, mode != "off")?;
                let ready = session.home.path().join("ready");
                let probe = session.home.path().join("probe");
                for file in [&ready, &probe] {
                    if file.exists() {
                        fs::remove_file(file)?;
                    }
                }
                let mut shell = session.shell(plugin, mode, false, mode != "off")?;
                shell.probe(&ready, "ready\n")?;
                let mut measured = Sample::from([(
                    "startup_ms".into(),
                    shell.started.elapsed().as_secs_f64() * 1000.0,
                )]);
                let ordinary = [vec![b'x'; 1000], vec![21]].concat();
                let workloads = [
                    ("ordinary_us", ordinary.clone(), 1000.0),
                    ("ineligible_space_us", b"\"gs\" \x15".repeat(100), 100.0),
                    ("eligible_space_us", b"gs \x15".repeat(100), 100.0),
                    ("deferred_batch_us", [vec![15], ordinary].concat(), 1000.0),
                ];
                for (index, (metric, mut payload, events)) in workloads.into_iter().enumerate() {
                    payload.push(29);
                    let start = Instant::now();
                    shell.send(&payload)?;
                    shell.probe(&probe, &format!("{}:\n", index + 1))?;
                    measured.insert(metric.into(), start.elapsed().as_secs_f64() * 1e6 / events);
                }
                if rep >= u64::from(args.warmups) {
                    samples.entry(name.clone()).or_default().push(measured);
                }
            }
        }
        println!("{mode}: {} samples per adapter", args.samples);
        results.insert(mode.into(), samples);
    }
    session.daemon("stop")?;
    Ok(results)
}

#[derive(Serialize)]
struct Stats {
    n: usize,
    mean: f64,
    sd: f64,
    median: f64,
    min: f64,
    max: f64,
    p10: f64,
    p90: f64,
}
fn distribution(mut values: Vec<f64>) -> Stats {
    values.sort_by(f64::total_cmp);
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    Stats {
        n,
        mean,
        sd: (values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt(),
        median: (values[(n - 1) / 2] + values[n / 2]) / 2.0,
        min: values[0],
        max: values[n - 1],
        p10: values[(0.1 * (n - 1) as f64).round_ties_even() as usize],
        p90: values[(0.9 * (n - 1) as f64).round_ties_even() as usize],
    }
}
type Summary = BTreeMap<String, BTreeMap<String, BTreeMap<String, Stats>>>;
fn summarize(results: &Results) -> Summary {
    results
        .iter()
        .map(|(group, samples)| {
            let mut variants = samples.clone();
            let mut pairs = vec![("candidate", "baseline")];
            if samples.contains_key("none") {
                pairs.extend([("candidate", "none"), ("baseline", "none")]);
            }
            for (name, reference) in pairs {
                let differences = samples[name]
                    .iter()
                    .zip(&samples[reference])
                    .map(|(a, b)| {
                        a.iter()
                            .map(|(metric, value)| (metric.clone(), value - b[metric]))
                            .collect()
                    })
                    .collect();
                variants.insert(format!("{name}_minus_{reference}"), differences);
            }
            let stats = variants
                .iter()
                .map(|(name, values)| {
                    let metrics = values[0]
                        .keys()
                        .map(|metric| {
                            (
                                metric.clone(),
                                distribution(values.iter().map(|sample| sample[metric]).collect()),
                            )
                        })
                        .collect();
                    (name.clone(), metrics)
                })
                .collect();
            (group.clone(), stats)
        })
        .collect()
}
#[derive(Serialize)]
struct Adapter {
    entry: PathBuf,
    revision: Option<String>,
    git_blob: BTreeMap<String, String>,
}
fn git_hash(path: &Path) -> Result<String> {
    checked(Command::new("git").arg("hash-object").arg(path))
}
fn adapter(entry: &Path) -> Result<Adapter> {
    let revision = Command::new("git")
        .arg("-C")
        .arg(entry.parent().unwrap())
        .args(["rev-parse", "HEAD"])
        .output()?;
    Ok(Adapter {
        entry: entry.to_owned(),
        revision: revision
            .status
            .success()
            .then(|| String::from_utf8_lossy(&revision.stdout).trim().to_owned()),
        git_blob: sources(entry)?
            .iter()
            .map(|file| {
                Ok((
                    file.file_name().unwrap().to_string_lossy().into_owned(),
                    git_hash(file)?,
                ))
            })
            .collect::<Result<_>>()?,
    })
}
#[derive(Serialize)]
struct Metadata {
    command: Vec<String>,
    platform: String,
    zsh: PathBuf,
    zsh_version: String,
    binary: PathBuf,
    binary_git_blob: String,
    adapters: BTreeMap<String, Adapter>,
    warmups: u32,
    samples: u32,
    workload: String,
    method: String,
}
#[derive(Serialize)]
struct Report {
    metadata: Metadata,
    samples: Results,
    summary: Summary,
}
fn main() -> Result<()> {
    let args = Args::parse();
    // SAFETY: handlers only set a lock-free atomic; cleanup runs on the main thread.
    unsafe {
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
    }
    let binary = args.binary.canonicalize()?;
    let zsh = if args.zsh.components().count() > 1 {
        args.zsh.canonicalize()?
    } else {
        std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
            .map(|dir| dir.join(&args.zsh))
            .find(|path| path.is_file())
            .ok_or("Zsh not found on PATH")?
            .canonicalize()?
    };
    let candidate =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("shell/zsh-autocomplete-rs.plugin.zsh");
    let adapters: BTreeMap<String, PathBuf> = BTreeMap::from([
        (
            "baseline".into(),
            args.baseline
                .as_ref()
                .unwrap_or(&candidate)
                .canonicalize()?,
        ),
        ("candidate".into(), candidate),
    ]);
    let metadata = Metadata {
        command: std::env::args().collect(), platform: checked(Command::new("uname").arg("-a"))?,
        zsh_version: checked(Command::new(&zsh).arg("--version"))?, zsh: zsh.clone(),
        binary_git_blob: git_hash(&binary)?, binary: binary.clone(),
        adapters: adapters.iter().map(|(name, entry)| Ok((name.clone(), adapter(entry)?))).collect::<Result<_>>()?,
        samples: args.samples, warmups: args.warmups, workload: args.workload.clone(),
        method: if args.workload == "startup" {
            "fresh interactive PTY (80x24); warm common compdump; EPOCHREALTIME source interval; PTY creation through final line-init; rotating order; same binary; cold excludes stop time, not filesystem caches"
        } else {
            "fresh interactive PTY (80x24); compinit -D; reused daemon; queued input through probe file; completion drawing suppressed; baseline then candidate"
        }.into(),
    };
    let session = Session::new(binary, zsh)?;
    let samples = if args.workload == "startup" {
        startup(&args, &session, &adapters)?
    } else {
        keys(&args, &session, &adapters)?
    };
    let summary = summarize(&samples);
    for (group, variants) in &summary {
        for (variant, metrics) in variants {
            for (metric, stats) in metrics {
                println!(
                    "{group} {variant} {metric}: {:.3} ± {:.3}",
                    stats.mean, stats.sd
                );
            }
        }
    }
    fs::write(
        &args.output,
        toml::to_string_pretty(&Report {
            metadata,
            samples,
            summary,
        })?,
    )?;
    Ok(())
}
