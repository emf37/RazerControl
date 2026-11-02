# -*- coding: utf-8 -*-
"""Reference implementation of the Razer HID control protocol (reverse engineered).

Frame layout (90-byte payload, sent as report-id 0x00 + 90 bytes = 91 bytes feature report):
    [0]      status            (0x00 = new command)
    [1]      transaction_id
    [2:4]    remaining_packets LE16
    [4]      protocol_type     (0xFF)
    [5]      data_size
    [6]      command_class
    [7]      command_id
    [8:88]   arguments[80]
    [88]     crc = XOR(payload[2:88])
    [89]     reserved = 0x00
"""
import ctypes
import ctypes.wintypes as wt
import sys

hid = ctypes.WinDLL('hid.dll')
sa = ctypes.WinDLL('setupapi.dll', use_last_error=True)
k32 = ctypes.WinDLL('kernel32.dll', use_last_error=True)

GEN = wt.DWORD(0)
k32.CreateFileW.restype = wt.HANDLE
k32.CreateFileW.argtypes = [ctypes.c_wchar_p, wt.DWORD, wt.DWORD, ctypes.c_void_p, wt.DWORD, wt.DWORD, wt.HANDLE]
k32.CloseHandle.argtypes = [wt.HANDLE]
hid.HidD_GetFeature.restype = wt.BOOL
hid.HidD_GetFeature.argtypes = [wt.HANDLE, ctypes.c_void_p, wt.DWORD]
hid.HidD_SetFeature.restype = wt.BOOL
hid.HidD_SetFeature.argtypes = [wt.HANDLE, ctypes.c_void_p, wt.DWORD]
hid.HidD_GetAttributes.argtypes = [wt.HANDLE, ctypes.c_void_p]
hid.HidD_GetPreparsedData.argtypes = [wt.HANDLE, ctypes.POINTER(ctypes.c_void_p)]
hid.HidP_GetCaps.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
hid.HidD_FreePreparsedData.argtypes = [ctypes.c_void_p]
hid.HidD_GetProductString.argtypes = [wt.HANDLE, ctypes.c_void_p, wt.ULONG]
sa.SetupDiGetClassDevsW.restype = ctypes.c_void_p
sa.SetupDiGetClassDevsW.argtypes = [ctypes.c_void_p, ctypes.c_wchar_p, ctypes.c_void_p, wt.DWORD]
sa.SetupDiEnumDeviceInterfaces.restype = wt.BOOL
sa.SetupDiEnumDeviceInterfaces.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, wt.DWORD, ctypes.c_void_p]
sa.SetupDiGetDeviceInterfaceDetailW.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_void_p, wt.DWORD,
                                                ctypes.POINTER(wt.DWORD), ctypes.c_void_p]
INVALID = wt.HANDLE(-1).value


class DID(ctypes.Structure):
    _fields_ = [('cbSize', wt.DWORD), ('g', ctypes.c_ubyte * 16), ('Flags', wt.DWORD), ('Res', ctypes.c_void_p)]


class ATTRS(ctypes.Structure):
    _fields_ = [('Size', wt.ULONG), ('VendorID', wt.USHORT), ('ProductID', wt.USHORT), ('VersionNumber', wt.USHORT)]


class CAPS(ctypes.Structure):
    _fields_ = [('Usage', wt.USHORT), ('UsagePage', wt.USHORT), ('InputReportByteLength', wt.USHORT),
                ('OutputReportByteLength', wt.USHORT), ('FeatureReportByteLength', wt.USHORT),
                ('Reserved', wt.USHORT * 17), ('NumberLinkCollectionNodes', wt.USHORT),
                ('NumberInputButtonCaps', wt.USHORT), ('NumberInputValueCaps', wt.USHORT),
                ('NumberInputDataIndices', wt.USHORT), ('NumberOutputButtonCaps', wt.USHORT),
                ('NumberOutputValueCaps', wt.USHORT), ('NumberOutputDataIndices', wt.USHORT),
                ('NumberFeatureButtonCaps', wt.USHORT), ('NumberFeatureValueCaps', wt.USHORT),
                ('NumberFeatureDataIndices', wt.USHORT)]


def hid_guid():
    g = (ctypes.c_ubyte * 16)()
    hid.HidD_GetHidGuid(ctypes.byref(g))
    return g


def enumerate_hid():
    g = hid_guid()
    h = sa.SetupDiGetClassDevsW(ctypes.cast(g, ctypes.c_void_p), None, None, 0x12)
    if h in (None, INVALID):
        raise ctypes.WinError(ctypes.get_last_error())
    out, i = [], 0
    d = DID()
    d.cbSize = ctypes.sizeof(d)
    DETAIL_CB = 8 if ctypes.sizeof(ctypes.c_void_p) == 8 else 6
    while sa.SetupDiEnumDeviceInterfaces(h, None, ctypes.cast(g, ctypes.c_void_p), i, ctypes.byref(d)):
        i += 1
        need = wt.DWORD(0)
        sa.SetupDiGetDeviceInterfaceDetailW(h, ctypes.byref(d), None, 0, ctypes.byref(need), None)
        buf = ctypes.create_string_buffer(need.value)
        ctypes.cast(buf, ctypes.POINTER(wt.DWORD))[0] = DETAIL_CB
        if not sa.SetupDiGetDeviceInterfaceDetailW(h, ctypes.byref(d), buf, need.value, ctypes.byref(need), None):
            continue
        out.append(ctypes.wstring_at(ctypes.addressof(buf) + 4))
    return out


def open_dev(path, access=0x80000000 | 0x40000000):
    return k32.CreateFileW(path, access, 3, None, 3, 0, None)


def close(h):
    k32.CloseHandle(h)


def info(path):
    h = open_dev(path, 0)
    if h in (None, INVALID):
        return None
    a = ATTRS()
    a.Size = ctypes.sizeof(a)
    hid.HidD_GetAttributes(h, ctypes.byref(a))
    prep = ctypes.c_void_p()
    caps = CAPS()
    ok = hid.HidD_GetPreparsedData(h, ctypes.byref(prep)) and hid.HidP_GetCaps(prep, ctypes.byref(caps))
    if prep:
        hid.HidD_FreePreparsedData(prep)
    b = ctypes.create_string_buffer(256)
    prod = ''
    if hid.HidD_GetProductString(h, b, 256):
        prod = bytes(b).decode('utf-16-le', 'ignore').split('\x00')[0]
    close(h)
    return dict(path=path, vid=a.VendorID, pid=a.ProductID, ver=a.VersionNumber,
                usage=caps.Usage, usage_page=caps.UsagePage,
                feature=caps.FeatureReportByteLength, input=caps.InputReportByteLength, product=prod)


def crc(buf):
    c = 0
    for b in buf[2:88]:
        c ^= b
    return c


class Razer(object):
    def __init__(self, path, report_len=91, verbose=False):
        self.path = path
        self.report_len = report_len
        self.h = open_dev(path)
        if self.h in (None, INVALID):
            raise OSError('open failed %s err=%d' % (path, ctypes.get_last_error()))
        self.tid = 0
        self.verbose = verbose

    def close(self):
        close(self.h)

    def _xfer(self, cls, cid, args=b'', size=None, delay=0.0):
        payload = bytearray(90)
        payload[0] = 0x00
        self.tid = (self.tid + 1) % 256
        payload[1] = self.tid
        payload[4] = 0xFF
        payload[5] = len(args) if size is None else size
        payload[6] = cls
        payload[7] = cid
        payload[8:8 + len(args)] = args
        payload[88] = crc(payload)
        payload[89] = 0x00
        req = bytes([0x00]) + bytes(payload)
        if self.verbose:
            print('  >', req.hex())
        b = ctypes.create_string_buffer(req, len(req))
        ok = hid.HidD_SetFeature(self.h, b, len(req))
        if not ok:
            return None, 'set_feature failed err=%d' % ctypes.get_last_error()
        resp = ctypes.create_string_buffer(self.report_len)
        ok = hid.HidD_GetFeature(self.h, resp, self.report_len)
        if not ok:
            return None, 'get_feature failed err=%d' % ctypes.get_last_error()
        raw = bytes(resp)
        if self.verbose:
            print('  <', raw.hex())
        return raw, None

    def cmd(self, cls, cid, args=b'', size=None, verbose=None):
        raw, err = self._xfer(cls, cid, args, size)
        if err:
            return None, err
        frame = raw[1:]  # strip report id
        if len(frame) < 90:
            return frame, 'short frame %d' % len(frame)
        status = frame[0]
        if status != 0x02 and status != 0x00:
            return frame, 'status=0x%02x' % status
        return frame, None


if __name__ == '__main__':
    cand = []
    for p in enumerate_hid():
        i = info(p)
        if i and i['vid'] == 0x1532:
            cand.append(i)
    for c in cand:
        print('PID %04X usage %04X/%04X feat=%-3d in=%-3d %-28s %s' % (
            c['pid'], c['usage_page'], c['usage'], c['feature'], c['input'],
            c['product'][:28], c['path'].split('#')[-1][:40]))
