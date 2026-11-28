# Minimal xonsh adapter prototype (prompt_toolkit shell). Candidates and the
# range they replace come from xonsh's own prompt_toolkit completer.
import os, subprocess
from prompt_toolkit.completion import CompleteEvent
from prompt_toolkit.application import run_in_terminal
$XONSH_SHOW_TRACEBACK = True
$PROMPT = '> '
$RIGHT_PROMPT = ''
$BOTTOM_TOOLBAR = ''
_dir = os.environ['ZACRS_PROTO_DIR']

@events.on_ptk_create
def _zacrs_bind(bindings, **kw):
    @bindings.add('c-i', eager=True)
    def _tab(event):
        buf = event.current_buffer
        doc = buf.document
        comps = list(buf.completer.get_completions(doc, CompleteEvent(completion_requested=True)))
        open(_dir + '/gathered', 'w').write(str(len(comps)))
        if not comps:
            return
        start = comps[0].start_position            # negative offset of the word start
        prefix = doc.text_before_cursor[len(doc.text_before_cursor) + start:]
        tsv = ''.join(f"{c.text}\t{c.display_meta_text or ''}\t\n" for c in comps) + 'END\n'
        size = event.app.output.get_size()
        def pick():
            pos = subprocess.run(['python3', _dir + '/cursor_pos.py'], capture_output=True, text=True).stdout.split()
            out = subprocess.run([os.environ['ZACRS_BIN'], 'complete', '--prefix', prefix, '--cursor-row', pos[0],
                                  '--cursor-col', pos[1], '--cols', str(size.columns), '--rows', str(size.rows)],
                                 input=tsv, stdout=subprocess.PIPE, text=True).stdout
            open(_dir + '/reply', 'w').write(out)
            done = out.split('\n')[0].split(' ', 2)
            if len(done) > 2 and done[0] == 'DONE' and done[1] in ('0', '2') and done[2]:
                buf.delete_before_cursor(-start)
                buf.insert_text(done[2])
        run_in_terminal(pick)

    @bindings.add('c-y', eager=True)
    def _probe(event):
        b = event.current_buffer
        open(_dir + '/probe', 'w').write(f"{b.text}|{b.cursor_position}")
