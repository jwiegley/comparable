TARGET = $(HOME)/dfinity/fixtures/rs/comparable

all: clippy

build: comparable/src/lib.rs
	cargo build

check:
	cargo test

test: build
	cargo test --doc
	cargo test

clippy: test
	cargo clippy -j40 --tests -- -D clippy::all

clean:
	rm -f comparable/src/lib.rs
	find . -name '*~' -print0 | xargs -0 /bin/rm -f

docs: comparable/src/lib.rs
	(cd comparable; cargo test --doc)
	(cd comparable; cargo doc)

docs-open: docs
	(cd comparable; cargo doc --open)

comparable/src/lib.rs: README.md comparable/src/lib.rs.in
	cat <(cat README.md | sed 's%^%//! %') comparable/src/lib.rs.in > $@

publish:
	@rsync -av --delete --delete-excluded					\
		--exclude 'Makefile'						\
		--exclude '*~' --exclude '/Cargo.*' --exclude 'default.nix'	\
		--exclude '.git' --exclude '.direnv' --exclude '.envrc*'	\
		./ $(TARGET)/
	@echo ""
	@echo "Delta files have been published to $(TARGET)"
