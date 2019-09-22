@0xa89d1aec630d3d37;

interface Bridge {
    readPhysicalMemory @0 (address :UInt64, length :UInt64) -> (data :Data);
    writePhysicalMemory @1 (address :UInt64, data: Data) -> (length :UInt64);

    readRegisters @2 () -> (data: Data);
}