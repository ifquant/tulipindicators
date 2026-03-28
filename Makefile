.PHONY: FORCE all clean veryclean rust-check

TI_ENABLE_CLIPPY ?= 1

Makefile: ;

all: FORCE
	$(MAKE) -C c all

rust-check:
	cd rust && cargo fmt --all --check
	cd rust && cargo test
	@if [ "$(TI_ENABLE_CLIPPY)" = "0" ]; then \
		echo "Skipping cargo clippy because TI_ENABLE_CLIPPY=0"; \
	else \
		cd rust && cargo clippy --all-targets --all-features; \
	fi

clean:
	$(MAKE) -C c clean
	rm -f *.a *.o *.ca
	rm -f benchmark benchmark_contract cli example1 example2 fuzzer sample smoke smoke_amal

veryclean:
	$(MAKE) -C c veryclean
	rm -f *.a *.o *.ca
	rm -f benchmark benchmark_contract cli example1 example2 fuzzer sample smoke smoke_amal

FORCE:

%: FORCE
	$(MAKE) -C c $@
