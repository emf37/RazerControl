# -*- coding: utf-8 -*-
"""读设备的 edition / 型号标识类字段。"""
import os
import sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rzclient as Z

p, _ = Z.find_control()
c = Z.RazerClient(p, timeout=0.7)


def rd(cls, cid, size, tries=3):
    for _ in range(tries):
        f, e = c.xfer(cls, cid, bytes(size), size)
        if f and e is None:
            return f[8:8 + size].hex()
        time.sleep(0.05)
    return 'timeout'


print('=== Edition / 型号标识 ===')
for cid, name in [(0x86, 'Edition Information'), (0x87, 'Extended FW'), (0x8E, 'Profile Polling'),
                  (0x8D, 'HW Version'), (0x0A, 'Device PID'), (0x24, 'MID Code')]:
    row = [(s, rd(0x00, cid, s, 1)) for s in (1, 2, 3, 4, 6)]
    print('  0x00/0x%02X %-20s %s' % (cid, name, '  '.join('%d:%s' % r for r in row)))

print()
print('=== USB 描述符里的信息（Windows 侧）===')
import ctypes
import razer as R
for path in R.enumerate_hid():
    i = R.info(path)
    if i and i['vid'] == 0x1532 and i['feature'] == 91:
        print('  product      :', i['product'])
        print('  manufacturer :', R.info(path)['product'])
        break
c.close()
