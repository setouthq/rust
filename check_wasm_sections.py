#!/usr/bin/env python3
import sys
import struct

def read_leb128(data, pos):
    """Read an unsigned LEB128 value"""
    result = 0
    shift = 0
    while True:
        byte = data[pos]
        pos += 1
        result |= (byte & 0x7F) << shift
        if (byte & 0x80) == 0:
            break
        shift += 7
    return result, pos

def parse_wasm(filename):
    with open(filename, 'rb') as f:
        data = f.read()

    # Check magic number
    if data[0:4] != b'\x00asm':
        print("Not a valid WASM file")
        return

    # Check version
    version = struct.unpack('<I', data[4:8])[0]
    print(f"WASM version: {version}")

    pos = 8
    while pos < len(data):
        section_id = data[pos]
        pos += 1

        # Read section size
        size, pos = read_leb128(data, pos)
        section_data = data[pos:pos+size]

        if section_id == 0:  # Custom section
            # Read name length
            name_len, name_pos = read_leb128(section_data, 0)
            name = section_data[name_pos:name_pos+name_len].decode('utf-8', errors='ignore')
            print(f"\nCustom section: '{name}' (size: {size} bytes)")

            # Print first 100 bytes of data
            data_start = name_pos + name_len
            content = section_data[data_start:data_start+100]
            print(f"Content (first 100 bytes): {content}")
            if name == ".rustc_proc_macro_decls":
                print(f"FOUND METADATA SECTION!")
                print(f"Full content: {section_data[data_start:].decode('utf-8', errors='ignore')}")

        pos += size

    print("\nDone parsing WASM file")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python check_wasm_sections.py <wasm_file>")
        sys.exit(1)

    parse_wasm(sys.argv[1])
