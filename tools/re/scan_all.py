# -*- coding: utf-8 -*-
"""全类目 getter 扫描（设备醒着）：找出所有可用命令及其返回。"""
import os
import sys, time
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import rzclient as Z

p, _ = Z.find_control()
c = Z.RazerClient(p, timeout=0.25)

ok = []
t0 = time.time()
for cls in range(0x00, 0x11):
    for cid in range(0x80, 0x90):
        f, e = c.xfer(cls, cid, bytes(2), 2)
        if f and e is None:
            ok.append((cls, cid, f[9:11].hex()))
# 补一轮已知特殊 size
for cls, cid, size in [(0x00, 0x82, 22), (0x04, 0x83, 38), (0x07, 0x8D, 8), (0x02, 0x84, 8)]:
    f, e = c.xfer(cls, cid, bytes(size), size)
    if f and e is None:
        ok.append((cls, cid, f[9:9 + size].hex()))
c.close()

print('elapsed %.1fs, supported: %d' % (time.time() - t0, len(ok)))
last = None
for cls, cid, v in sorted(ok):
    if cls != last:
        print('--- class 0x%02X ---' % cls)
        last = cls
    print('   0x%02X/0x%02X -> %s' % (cls, cid, v))
