#!/usr/bin/env python3
"""Compare two Zsh adapters using one debug binary and a reused private daemon.

Linux/macOS, Python 3.9+, Zsh and a built zacrs binary are required.
Terminal redraw and control keys are included in workload timings. This does
not measure candidate latency: automatic completion drawing is suppressed.
"""
import argparse
import json, os, pathlib, pty, select, signal, statistics, subprocess, tempfile, time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("baseline", type=pathlib.Path, help="baseline plugin entry point")
parser.add_argument("--binary", type=pathlib.Path, default=pathlib.Path("target/debug/zsh-autocomplete-rs"))
parser.add_argument("--samples", type=int, default=15)
parser.add_argument("--output", type=pathlib.Path, default=pathlib.Path("/tmp/zacrs-zle-bench.json"))
args = parser.parse_args()
if args.samples < 3:
    parser.error("--samples must be at least 3")
ROOT = pathlib.Path.cwd()
BIN = args.binary.resolve()
VARIANTS = {'baseline': args.baseline.resolve(), 'candidate': ROOT / 'shell/zsh-autocomplete-rs.plugin.zsh'}
RC = r'''
HISTFILE="$HOME/history"
autoload -Uz compinit add-zle-hook-widget
compinit -D
source "$BENCH_PLUGIN"
if [[ $BENCH_MODE == hot-disabled ]]; then
    zle -A self-insert _bench_saved
    _bench_outer() { zle _bench_saved -w; }
    zle -N self-insert _bench_outer
    print -rl -- '[completion]' 'expand_abbreviation_on_space = false' '[abbreviations]' 'gs = "git status"' > "$XDG_CONFIG_HOME/zacrs/config.toml"
    source "$BENCH_PLUGIN"
fi
_zacrs_line_pre_redraw() { :; }
_bench_probe() { print -r -- "$(( ++_bench_count )):$BUFFER" > "$HOME/probe"; }
_bench_ready() { print ready > "$HOME/ready"; }
_bench_defer() { _zacrs_defer_redraw; }
zle -N _bench_probe
zle -N _bench_defer
bindkey '^]' _bench_probe
bindkey '^O' _bench_defer
add-zle-hook-widget line-init _bench_ready
PROMPT='B> '
RPROMPT=''
'''
def wait(fd, path, expected, timeout=10):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if select.select([fd], [], [], .0002)[0]:
            try: os.read(fd, 65536)
            except OSError: pass
        if path.exists() and path.read_text() == expected:
            return
    raise RuntimeError(f'timeout {path}: {path.read_text() if path.exists() else "missing"}; expected {expected!r}')

results = {}
with tempfile.TemporaryDirectory(prefix='zacrs-166-bench.') as tmp:
    home = pathlib.Path(tmp)
    config = home / 'config/zacrs'
    config.mkdir(parents=True)
    env = {'PATH':os.environ['PATH'], 'HOME':tmp, 'ZDOTDIR':tmp, 'XDG_CONFIG_HOME':str(home/'config'), 'XDG_CACHE_HOME':str(home/'cache'), 'XDG_RUNTIME_DIR':tmp, 'TERM':'xterm-256color','LC_ALL':'C.UTF-8','ZACRS_BIN':str(BIN)}
    (home/'.zshrc').write_text(RC)
    try:
        # Starting the daemon is excluded; every timed startup reuses it.
        (config/'config.toml').write_text('[abbreviations]\ngs = "git status"\n')
        subprocess.run([str(BIN),'daemon','ensure'],env=env,check=True,stdout=subprocess.DEVNULL)
        # Alternate revisions within each repetition to reduce temporal bias.
        for mode in ['off','on','hot-disabled']:
            for rep in range(args.samples + 3):
                for name, plugin in VARIANTS.items():
                    (config/'config.toml').write_text('[completion]\nexpand_abbreviation_on_space = '+('false' if mode=='off' else 'true')+'\n[abbreviations]\ngs = "git status"\n')
                    for f in ['ready','probe']:
                        (home/f).unlink(missing_ok=True)
                    start=time.perf_counter_ns()
                    pid, fd=pty.fork()
                    if pid==0:
                        os.execvpe('zsh',['zsh','-di'],dict(env, BENCH_PLUGIN=str(plugin),BENCH_MODE=mode))
                    try:
                        wait(fd,home/'ready','ready\n')
                        startup=(time.perf_counter_ns()-start)/1e6
                        measured={'startup_ms':startup}
                        for counter,(label,payload,events) in enumerate([
                            ('ordinary_us',b'x'*1000+b'\x15',1000),
                            ('ineligible_space_us',b'"gs" \x15'*100,100),
                            ('eligible_space_us',b'gs \x15'*100,100),
                            ('deferred_batch_us',b'\x0f'+b'x'*1000+b'\x15',1000),
                        ],1):
                            start=time.perf_counter_ns()
                            os.write(fd,payload+b'\x1d')
                            wait(fd,home/'probe',f'{counter}:\n')
                            measured[label]=(time.perf_counter_ns()-start)/1000/events
                        if rep>=3:
                            results.setdefault(mode,{}).setdefault(name,[]).append(measured)
                    finally:
                        os.kill(pid,signal.SIGKILL)
                        os.close(fd)
                        os.waitpid(pid,0)
                if rep in [2,args.samples + 2]: print(mode,rep,flush=True)
    finally:
        subprocess.run([str(BIN),'daemon','stop'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
summary={}
for mode,variants in results.items():
    summary[mode]={}
    for name,samples in variants.items():
        summary[mode][name]={}
        for metric in samples[0]:
            values=sorted(s[metric] for s in samples)
            summary[mode][name][metric]={'median':statistics.median(values),'min':min(values),'max':max(values),'p10':values[round(.1*(len(values)-1))],'p90':values[round(.9*(len(values)-1))]}
args.output.write_text(json.dumps({'samples':results,'summary':summary},indent=2))
print(json.dumps(summary,indent=2))
