# 逆向脚本（可直接跑）

全部只依赖标准库（`ctypes` 直调 `hid.dll` / `setupapi.dll`），Windows + Python 3.8+ 即可。

```powershell
python rzclient.py        # 打印设备信息 + 当前状态
python scan_all.py        # 全类目 getter 扫描（class 0x00..0x10 × id 0x80..0x8F）
python sniff_long.py 200  # 被动嗅探 200 秒，抓 Synapse 自己发出的帧
python probe_edition.py   # 读 editionId / 硬件版本等标识
python probe_wired.py     # 插线时枚举并逐个探测所有控制接口
```

## 核心技巧：共享 Feature Report 缓冲区

`HidD_GetFeature` 读的是设备共享的那一块 Feature Report 缓冲区。Synapse 写完命令后，
帧还留在缓冲区里 —— 所以只申请 `GENERIC_WRITE` 权限打开句柄（读权限被厂商驱动独占），
高频读就能看到 Synapse 发出的每一条命令。不用驱动、不用注入、不用管理员。

`sniff_long.py` 就是这么抓出校准命令（`class 0x0B`）的：

```
+ 26.25s  cls=0x0B id=0x0B size=4   st=0x01 args=00040101
+ 31.52s  cls=0x0B id=0x05 size=10  st=0x01 args=00040e08000000000000
```

`st=0x01` = 请求本身（设备还没处理完），`st=0x02` = 应答。

## 注意

* 无线鼠标休眠后所有命令会超时，**探测必须重试**（脚本里都有 3 次重试和 50 ms 间隔）；
* 嗅探时先关掉本项目的 GUI，否则它自己的轮询帧会混进来。
