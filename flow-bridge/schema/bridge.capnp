@0xa89d1aec630d3d37;

interface Bridge {
    physRead @0 (address :UInt64, length :UInt64) -> (data :Data);
    physWrite @1 (address :UInt64, data: Data) -> (length :UInt64);

    virtRead @2 (arch: UInt8, dtb :UInt64, address :UInt64, length :UInt64) -> (data: Data);
    virtWrite @3 (arch: UInt8, dtb: UInt64, address :UInt64, data: Data) -> (length :UInt64);

    readRegisters @4 () -> (data: Data);
}
