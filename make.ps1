# Windows Powershell Teensy 3.2 Rust build->flash
# The following parameters are available with automcomplete from the command line
# Example: ./make.ps1 -clean -flash

param (
    [switch]$clean = $false,
    [switch]$test = $false,
    [switch]$bench = $false,
    [switch]$flash = $false
)

# Change BIN to match the name of your executable
$BIN="teensy"
# Change TARGET if needed for a different device
$TARGET="thumbv7em-none-eabi"
$OUTDIR="target/$TARGET/release"
$HEX="$OUTDIR/$BIN.hex"
$ELF="$OUTDIR/$BIN"

if ($clean) {
	cargo clean
	if (-Not $?) {
		# Stop the script on error result from the previous command
		exit
	}
}

if ($test) {
	cargo test
	if (-Not $?) {
		exit
	}
}

if ($bench) {
	cargo bench
	if (-Not $?) {
		exit
	}
}

cargo build --target=$TARGET --release
if (-Not $?) {
	exit
}

arm-none-eabi-objcopy -O ihex $ELF $HEX
if (-Not $?) {
	exit
}

if ($flash) {
	teensy_loader_cli -w -mmcu=mk20dx256 $HEX -v
}
