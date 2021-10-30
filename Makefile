TARGET = $(HOME)/dfinity/fixtures/rs/delta

all: build test

build:
	cargo build

check:
	cargo test

test:
	cargo test

clean:
	@find . -name '*~' -print0 | xargs -0 /bin/rm -f

publish:
	@rsync -av --delete --delete-excluded					\
		--exclude 'Makefile'						\
		--exclude '*~' --exclude '/Cargo.*' --exclude 'default.nix'	\
		--exclude '.git' --exclude '.direnv' --exclude '.envrc*'	\
		./ $(TARGET)/
	@echo ""
	@echo "Delta files have been published to $(TARGET)"
