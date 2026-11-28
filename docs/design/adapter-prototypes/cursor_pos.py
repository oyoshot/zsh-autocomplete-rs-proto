# Prototype shortcut: ask the terminal where the cursor is. Prints "row col" (0-based).
import os, termios, tty, select, re
fd = os.open('/dev/tty', os.O_RDWR)
old = termios.tcgetattr(fd)
try:
    tty.setraw(fd)
    os.write(fd, b'\x1b[6n')
    buf = b''
    while not re.search(rb'\x1b\[(\d+);(\d+)R', buf):
        if not select.select([fd], [], [], 2)[0]:
            break
        buf += os.read(fd, 32)
finally:
    termios.tcsetattr(fd, termios.TCSANOW, old)
m = re.search(rb'\x1b\[(\d+);(\d+)R', buf)
print(int(m[1]) - 1 if m else 0, int(m[2]) - 1 if m else 0)
