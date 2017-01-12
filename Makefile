BIN=teensy
OUTDIR=target/thumbv7em-none-eabi/release
HEX=$(OUTDIR)/$(BIN).hex
ELF=$(OUTDIR)/$(BIN)

all:: $(HEX)

$(HEX): $(ELF)
	arm-none-eabi-objcopy -O ihex $(ELF) $(HEX)

.PHONY: $(ELF)
$(ELF):
	~/.cargo/bin/xargo build --target=thumbv7em-none-eabi --release

flash: $(HEX)
	teensy-loader-cli -w -mmcu=mk20dx256 $(HEX) -v
