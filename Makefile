all: target/release/vm

target/release/vm:
	cargo build --release

unit-tests: target/release/vm
	cargo test

integration-tests: target/release/vm
	cd tests; chmod +x test.sh; ./test.sh

clean:
	rm target/release/vm

.PHONY: unit-tests integration-tests clean
