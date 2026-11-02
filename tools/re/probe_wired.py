# -*- coding: utf-8 -*-
"""插线后：枚举所有控制接口，逐个探测（含充电状态）。"""
import os
import sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import razer as R
import rzclient as Z

print('=== 枚举 ===')
ctrl = []
for p in R.enumerate_hid():
    i = R.info(p)
    if i and i['vid'] == 0x1532 and i['feature'] == 91:
        ctrl.append((p, i))
        print('  %04X:%04X  %s' % (i['pid'], i['ver'], i['product']))

for p, i in ctrl:
    print()
    print('=== %04X:%04X 探测 ===' % (i['pid'], i['ver']))
    try:
        c = Z.RazerClient(p, timeout=0.6)
    except OSError as e:
        print('  open failed:', e)
        continue

    def rd(cls, cid, size, tries=3):
        for _ in range(tries):
            f, e = c.xfer(cls, cid, bytes(size), size)
            if f and e is None:
                return f[8:8 + size].hex()
            time.sleep(0.05)
        return 'timeout'

    print('  firmware 0x00/0x81 :', rd(0x00, 0x81, 2))
    print('  mode     0x00/0x84 :', rd(0x00, 0x84, 2))
    print('  dpi      0x04/0x85 :', rd(0x04, 0x85, 7))
    print('  battery  0x07/0x80 :', rd(0x07, 0x80, 2))
    print('  charge   0x07/0x84 :', rd(0x07, 0x84, 2))
    print('  0x07/0x8D           :', rd(0x07, 0x8D, 8))
    print('  0x07/0x85           :', rd(0x07, 0x85, 2))
    print('  0x05/0x81           :', rd(0x05, 0x81, 2))
    c.close()
