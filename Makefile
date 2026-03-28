.PHONY: FORCE all clean veryclean

Makefile: ;

all: FORCE
	$(MAKE) -C c all

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
