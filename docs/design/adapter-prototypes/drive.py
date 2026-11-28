"""Drive one prototype adapter in a pseudo-terminal.

usage: SCENARIOS='[[name, typed, [keys], expect_popup], ...]' python3 drive.py <this dir> <repo> <shell>

The repo needs a release build of the engine. Each scenario types text, presses
Tab, sends the keys to the picker and prints the gathered count, the engine's
reply and the resulting command line.
"""
import os, pty, sys, time, select, re, shutil, tempfile, subprocess
S, repo, shell = sys.argv[1], sys.argv[2], sys.argv[3]
home = tempfile.mkdtemp(prefix='zacrs-fish.')
os.makedirs(home + '/.config/fish')
shutil.copy(S + '/fish/config.fish', home + '/.config/fish/config.fish')
shutil.copy(S + '/bash/bashrc', home + '/.bashrc')
proto = home + '/proto'; os.makedirs(proto)
shutil.copy(S + '/cursor_pos.py', proto)
env = {'PATH': os.environ['PATH'], 'HOME': home, 'XDG_CONFIG_HOME': home + '/.config', 'TERM': 'xterm-256color',
       'XDG_RUNTIME_DIR': proto, 'ZACRS_BIN': repo + '/target/release/zsh-autocomplete-rs', 'ZACRS_PROTO_DIR': proto,
       'LANG': 'C.UTF-8', 'SHELL_TYPE': 'prompt_toolkit', 'XONSH_HISTORY_BACKEND': 'dummy', 'POWERSHELL_TELEMETRY_OPTOUT': '1', 'POWERSHELL_UPDATECHECK': 'Off', 'COLUMNS': '100', 'LINES': '24'}
pid, fd = pty.fork()
if pid == 0:
    os.chdir(repo)
    argv = {'fish': ['fish', '-i'], 'bash': ['bash', '--noprofile', '--rcfile', home + '/.bashrc', '-i'], 'xonsh': ['xonsh', '--rc', S + '/xonsh/rc.xsh', '-i'], 'elvish': ['elvish', '-rc', S + '/elvish/rc.elv'], 'nu': ['nu', '--config', S + '/nu/config.nu'], 'pwsh': ['pwsh', '-NoLogo', '-NoProfile', '-NoExit', '-File', S + '/pwsh/proto.ps1']}[shell]
    os.execvpe(argv[0], argv, env)
import fcntl, termios, struct
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 100, 0, 0))
out = b''
def pump(t=0.05):
    global out
    end = time.time() + t
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.02)
        if not r: continue
        try: data = os.read(fd, 65536)
        except OSError: return
        out += data
        # answer the terminal queries fish and the engine make
        if b'\x1b[c' in data: os.write(fd, b'\x1b[?62;22c')
        if b'\x1b[6n' in data: os.write(fd, b'\x1b[5;10R')
        if b'\x1b[5n' in data: os.write(fd, b'\x1b[0n')
        if b'\x1b[?u' in data: os.write(fd, b'\x1b[?0u')
        if b'\x1b[>0q' in data or b'\x1b[>q' in data: os.write(fd, b'\x1bP>|proto\x1b\\')
def wait(pred, t=8, what=''):
    end = time.time() + t
    while time.time() < end:
        pump()
        if pred(): return True
    print('TIMEOUT waiting for', what); return False
def send(b): os.write(fd, b)
def read(name):
    try: return open(proto + '/' + name).read()
    except FileNotFoundError: return None
plain = lambda: re.sub(rb'\x1b\[[0-9;?]*[A-Za-z]', b'', out)
ok = wait(lambda: b'> ' in plain(), t=25, what='prompt'); pump(1.0)
results = {}
def scenario(name, typed, keys, expect_popup):
    global out
    send(b'\x15'); pump(0.2)            # ctrl-u: clear the line
    for f in ('gathered', 'reply', 'probe'):
        try: os.remove(proto + '/' + f)
        except FileNotFoundError: pass
    out = b''
    send(typed.encode()); pump(0.3)
    send(b'\t')
    got = wait(lambda: read('gathered') is not None, what=name + ' gather')
    popup = wait(lambda: '┌'.encode() in out, t=4, what=name + ' popup') if expect_popup else None
    shown = plain().decode(errors='replace')
    for k in keys:
        pump(0.3); send(k)
    wait(lambda: read('reply') is not None, t=5, what=name + ' reply')
    pump(0.3); send(b'\x19')
    wait(lambda: read('probe') is not None, t=3, what=name + ' probe')
    results[name] = dict(gathered=(read('gathered') or '').strip(), popup=popup,
                         reply=(read('reply') or '').strip().replace('\n', ' / '), line=(read('probe') or '').strip(),
                         popup_text=[l for l in re.findall(r'│[^│\n]*│', shown)][:4])
import json
for name, typed, keys, popup in json.loads(os.environ['SCENARIOS']):
    scenario(name, typed, [k.encode('latin1') for k in keys], popup)
for k, v in results.items(): print(k, '=>', v)
send(b'\x04'); time.sleep(0.2)
try: os.kill(pid, 9)
except OSError: pass
subprocess.run(['rm', '-rf', home])
