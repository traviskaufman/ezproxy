import ctypes
import sys

RUSAGE_INFO_V4 = 4
USER_TIME, SYSTEM_TIME, INSTRUCTIONS = 0, 1, 29
NANOSECONDS_PER_SECOND = 1_000_000_000


class RusageInfoV4(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_uint8 * 16), ("counters", ctypes.c_uint64 * 35)]


libsystem = ctypes.CDLL("/usr/lib/libSystem.B.dylib")


def timebase_ticks_per_second():
    frequency = ctypes.c_uint64()
    size = ctypes.c_size_t(ctypes.sizeof(frequency))
    libsystem.sysctlbyname(
        b"hw.tbfrequency", ctypes.byref(frequency), ctypes.byref(size), None, 0
    )
    return frequency.value


pid = int(sys.argv[1])
usage = RusageInfoV4()
if libsystem.proc_pid_rusage(pid, RUSAGE_INFO_V4, ctypes.byref(usage)) != 0:
    sys.exit(f"proc_pid_rusage failed for pid {pid}")
cpu_ticks = usage.counters[USER_TIME] + usage.counters[SYSTEM_TIME]
cpu_nanoseconds = cpu_ticks * NANOSECONDS_PER_SECOND // timebase_ticks_per_second()
print(cpu_nanoseconds, usage.counters[INSTRUCTIONS])
