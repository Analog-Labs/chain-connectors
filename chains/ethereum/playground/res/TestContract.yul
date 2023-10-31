object "TestContract" {
    code {
        // Store the creator in slot zero.
        sstore(0, caller())

        // Deploy the contract
        datacopy(0, dataoffset("Runtime"), datasize("Runtime"))
        return(0, datasize("Runtime"))
    }
    object "Runtime" {
        code {
            // Dispatcher
            switch selector()
            case 0x185c38a4 /* "revertWithMessage()" */ {
                revertWithMessage()
            }
            case 0xc06a97cb /* "revertWithoutMessage()" */ {
                revertWithoutMessage()
            }
            case 0xb1ae6db0 /* "invalidOpcode()" */ {
                invalidOpcode()
            }
            case 0x8c192bc9 /* "stackUnderflow()" */ {
                stackUnderflow()
            }
            case 0xddf91d0c /* stackOverflow() */ {
                stackOverflow()
            }
            case 0xd502dc8e /* invalidJumpDest() */ {
                invalidJumpDest()
            }
            case 0x31fe52e8 /* outOfGas() */ {
                outOfGas()
            }
            case 0x71663476 /* resultTest() */ {
                resultTest()
            }
            case 0xfe3fb5c7 /* sha256Precompiled() */ {
                sha256Precompiled()
            }
            case 0x4fae1f41 /* ripemd160Precompiled() */ {
                ripemd160Precompiled()
            }
            case 0x6cd5c39b /* deployContract() */ {
                let addr := deployContract()
                mstore(0, addr)
                return(12, 20)
            }
            case 0xa90bc19e /* deployCopy() */ {
                let addr := deployCopy()
                mstore(0, addr)
                return(12, 20)
            }
            case 0x800d8e2b /* deployCopy(uint256) */ {
                let salt := calldataload(4)
                let addr := deployCopy2(salt)
                mstore(0, addr)
                return(12, 20)
            }
            case 0xea879634 /* getCode() */ {
                let size := getCode(0)
                return(0, size)
            }
            case 0xf509c2d2 /* getCodeWithConstructor() */ {
                let size := getCodeWithConstructor(0)
                return(0, size)
            }
            case 0x677342ce /* sqrt(uint256) */ {
                let result := sqrt(calldataload(4))
                mstore(0, result)
                return(0, 32)
            }
            default {
                revert(0, 0)
            }

            /* ---------- calldata decoding functions ----------- */
            function selector() -> s {
                s := div(calldataload(0), 0x100000000000000000000000000000000000000000000000000000000)
            }

            // Revert with message "something is wrong"
            function revertWithMessage() {
                mstore(0x00, 0x08c379a000000000000000000000000000000000000000000000000000000000) // Function selector for Error(string)
                mstore(0x04, 0x0000000000000000000000000000000000000000000000000000000000000020) // Data offset
                mstore(0x24, 0x0000000000000000000000000000000000000000000000000000000000000012) // String length
                mstore(0x44, 0x736f6d657468696e672069732077726f6e670000000000000000000000000000) // String data "something is wrong"
                revert(0, 0x60)
            }

            // Revert with empty message
            function revertWithoutMessage() {
                revert(0, 0)
            }
            
            // Trigger Invalid OPCODE error
            function invalidOpcode() {
                // https://ethereum.org/en/developers/docs/evm/opcodes/
                verbatim_0i_0o(hex"A5") // Doesn't exists A5 opcode
                return(0, 0x0)
            }

            // Trigger Stack Underflow error
            function stackUnderflow() {
                verbatim_1i_0o(hex"56", dataoffset("UnderflowCode")) // JUMP
                return(0, 0)
            }

            // Trigger Stack Overflow error
            function stackOverflow() {
                verbatim_1i_0o(hex"56", dataoffset("OverflowCode")) // JUMP
                return(0, 0)
            }

            // Trigger Invalid JUMPDEST error
            function invalidJumpDest() {
                verbatim_1i_0o(hex"56", dataoffset("InvalidJumpCode")) // JUMP
                return(0, 0)
            }

            // Trigger out of gas error
            function outOfGas() {
                verbatim_1i_0o(hex"56", dataoffset("OutOfGasCode")) // JUMP
                return(0, 0)
            }

            // Calls the SHA256 precompiled contract
            function sha256Precompiled() {
                let size := sub(calldatasize(), 4)
                calldatacopy(
                    0,    // out-memory ptr
                    4,    // calldata offset
                    size  // calldata size
                )
                let result := staticcall(
                    gas(), // Gas available
                    0x02,  // contract address (sha256 precompiled)
                    0,     // in-memory ptr
                    size,  // in-memory size
                    0,     // out-memory ptr
                    32     // out-memory size
                )
                if iszero(result) {
                    revert(0, 0x20)
                }
                return(0, 0x20)
            }

            // Calls the RIPEMD-160 precompiled contract
            function ripemd160Precompiled() {
                let size := sub(calldatasize(), 4)
                calldatacopy(
                    0,    // out-memory ptr
                    4,    // calldata offset
                    size  // calldata size
                )
                let result := staticcall(
                    gas(), // Gas the call can use
                    0x03,  // contract address (RIPEMD-160 precompiled)
                    0,     // in-memory ptr
                    size,  // in-memory size
                    0,     // out-memory ptr
                    32     // out-memory size
                )
                if iszero(result) {
                    revert(0, 0x20)
                }
                return(12, 20)
            }

            // Deploy a copy of self a contract
            function deployCopy() -> addr {
                // Copy the constructor to memory
                let size := getCodeWithConstructor(0)
                addr := create(
                    callvalue(), // value to transfer to the contract
                    0,   // in-memory ptr
                    size // in-memory size
                )
                if iszero(addr) {
                    revert(0, size)
                }
            }

            // Create a contract using CREATE2 opcode
            function deployCopy2(salt) -> addr {
                // Copy the constructor to memory
                let size := getCodeWithConstructor(0)
                addr := create2(
                    callvalue(),// value to transfer to the contract
                    0,          // in-memory ptr
                    size,       // in-memory size
                    salt        // salt
                )
                if iszero(addr) {
                    revert(0, size)
                }
            }

            // Deploy a contract based on calldata parameters
            function deployContract() -> addr {
                // read codesize
                let size := sub(calldatasize(), 4)

                // Store constructor
                let constructor := or(
                    // PUSH2 [codesize] PUSH1 0 DUP2 CALLER DUP3 SSTORE PUSH1 0x0e DUP3 CODECOPY RETURN
                    0x610000600081338255600e8239f3000000000000000000000000000000000000,
                    shl(232, size)
                )

                // Store bytecode
                calldatacopy(0, 14, size)

                // Deploy contract
                addr := create(
                    callvalue(),  // value to transfer to the contract
                    0,            // in-memory ptr
                    add(size, 14) // in-memory size
                )
                if iszero(addr) {
                    revert(0, size)
                }
            }

            // Return self bytecode
            function getCode(ptr) -> size {
                // Copy the code to memory
                codecopy(
                    ptr, // out-memory ptr
                    0, // code offset
                    codesize() // code size
                )
                size := codesize()
            }

            // Return the constructor for deploying a copy of self.
            function getCodeWithConstructor(ptr) -> size {
                // Copy the code to memory
                size := getCode(add(ptr, 14))

                // Load code size into constructor
                let constructor := or(
                    // PUSH2 [codesize] PUSH1 0 DUP2 CALLER DUP3 SSTORE PUSH1 0x0e DUP3 CODECOPY RETURN
                    0x610000600081338255600e8239f3000000000000000000000000000000000000,
                    shl(232, size)
                )

                // Store constructor in memory between bytes [ptr, ..ptr + 13]
                mstore(0, or(constructor, mload(ptr)))

                // Add constructor size to final bytecode size
                size := add(size, 13)
            }

            /// Returns the maximum result size possible given the available gas
            ///
            /// # Description
            /// Calculate the gas cost of memory expansion given the number of 256-bit words `x`:
            /// f(x) -> 3 * x + (x^2 / 512)
            ///
            /// Calculate the number of 256-bit words given the available gas `y`:
            /// f(y) -> sqrt(512 * (y + 1152)) - 768
            /// 
            /// reference: https://ethereum.github.io/yellowpaper/paper.pdf Appendix H
            function resultTest() {
                // Current available GAS minus the gas necessary to calculate the sqrt (~664 gas units).
                let g := sub(gas(), 664)
                // Calculate the number of 256-bit words given the available gas `g`
                let words := sub(sqrt(shl(9, add(g, 1152))), 768)
                // Convert 256-bit words to bytes by multiplying it by 32
                let size := shl(5, words)
                return(0, size)
            }

            // Square Root (worst case gas cost is 555)
            // https://ethereum-magicians.org/t/eip-7054-gas-efficient-square-root-calculation-with-binary-search-approach/14539
            function sqrt(x) -> res {
                let xx := x
                let r := 1
                if lt(0x100000000000000000000000000000000, xx) {
                    xx := shr(128, xx)
                    r := shl(64, r)
                }
                if lt(0x10000000000000000, xx) {
                    xx := shr(64, xx)
                    r := shl(32, r)
                }
                if lt(0x100000000, xx) {
                    xx := shr(32, xx)
                    r := shl(16, r)
                }
                if lt(0x10000, xx) {
                    xx := shr(16, xx)
                    r := shl(8, r)
                }
                if lt(0x100, xx) {
                    xx := shr(8, xx)
                    r := shl(4, r)
                }
                if lt(0x10, xx) {
                    xx := shr(4, xx)
                    r := shl(2, r)
                }
                if lt(0x8, xx) {
                    r := shl(1, r)
                }
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                r := shr(1, add(r, div(x, r)))
                res := div(x, r)
                if lt(r, res) {
                    res := r
                }
            }

            function require(condition) {
                if iszero(condition) { revert(0, 0) }
            }
        }
        data "InvalidJumpCode" hex"00" // STOP
        data "OutOfGasCode" hex"5B6003580356" // JUMPDEST PUSH1 0x03 PC SUB JUMP
        data "UnderflowCode" hex"5B010101010101" // JUMPDEST ADD ADD ADD ADD ADD ADD
        data "OverflowCode" hex"5B456004580356" // JUMPDEST GASLIMIT PUSH1 0x04 PC SUB JUMP
    }
}
