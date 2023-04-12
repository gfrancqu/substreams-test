ENDPOINT ?= mainnet.sol.streamingfast.io:443

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml map_block -s 187865737 -t +10

.PHONY: protogen
protogen:
	substreams protogen ./substreams.yaml --exclude-paths="google,sf/substreams/v1"

.PHONY: package
package:
	substreams pack ./substreams.yaml
