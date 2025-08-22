# Makefile for rs-voice-toolkit version management and publishing

# Current version from workspace
CURRENT_VERSION := $(shell grep '^version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

# Extract major, minor, patch components
VERSION_MAJOR := $(shell echo "$(CURRENT_VERSION)" | cut -d. -f1)
VERSION_MINOR := $(shell echo "$(CURRENT_VERSION)" | cut -d. -f2)
VERSION_PATCH := $(shell echo "$(CURRENT_VERSION)" | cut -d. -f3)

# Calculate next version (bump minor by default)
NEXT_MINOR := $(shell echo $$(($(VERSION_MINOR) + 1)))
NEXT_VERSION := $(VERSION_MAJOR).$(NEXT_MINOR).0

# Crate directories
CRATES := audio tts stt
MAIN_CRATE := voice-toolkit

.PHONY: all bump-version publish-all publish-crates publish-main clean help

all: bump-version publish-all

# Bump version for all crates and update dependencies
bump-version:
	@echo "Current version: $(CURRENT_VERSION)"
	@echo "Bumping to version: $(NEXT_VERSION)"
	
	# Update workspace version
	@sed -i '' 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_VERSION)"/' Cargo.toml
	
	# Update each crate's version
	@for crate in $(CRATES); do \
		sed -i '' 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_VERSION)"/' "$$crate/Cargo.toml"; \
		echo "Updated $$crate to version $(NEXT_VERSION)"; \
	done
	
	# Update main crate version
	@sed -i '' 's/version = "$(CURRENT_VERSION)"/version = "$(NEXT_VERSION)"/' "$(MAIN_CRATE)/Cargo.toml"
	
	# Update dependencies in stt crate
	@sed -i '' 's/audio_utils = { package = "rs-voice-toolkit-audio", version = "$(CURRENT_VERSION)"/audio_utils = { package = "rs-voice-toolkit-audio", version = "$(NEXT_VERSION)"/' "stt/Cargo.toml"
	
	# Update dependencies in main crate
	@for crate in $(CRATES); do \
		sed -i '' 's/rs-voice-toolkit-$$crate = {.*version = "$(CURRENT_VERSION)"/rs-voice-toolkit-$$crate = { version = "$(NEXT_VERSION)"/' "$(MAIN_CRATE)/Cargo.toml"; \
	done
	
	@echo "All versions updated to $(NEXT_VERSION)"

# Publish all crates in correct order
publish-all: publish-crates publish-main

# Publish sub-crates (audio, tts, stt)
publish-crates:
	@echo "Publishing sub-crates..."
	@for crate in $(CRATES); do \
		echo "Publishing $$crate..."; \
		cd "$$crate" && cargo publish --allow-dirty; \
		if [ $$? -ne 0 ]; then \
			echo "Failed to publish $$crate"; \
			exit 1; \
		fi; \
		echo "$$crate published successfully"; \
		echo "Waiting 30 seconds for crates.io to update..."; \
		sleep 30; \
	done
	@echo "All sub-crates published successfully"

# Publish main crate (voice-toolkit)
publish-main:
	@echo "Publishing main crate $(MAIN_CRATE)..."
	@cd "$(MAIN_CRATE)" && cargo publish --allow-dirty
	@if [ $$? -eq 0 ]; then \
		echo "$(MAIN_CRATE) published successfully"; \
	else \
		echo "Failed to publish $(MAIN_CRATE)"; \
		exit 1; \
	fi

# Clean up any temporary files
clean:
	@echo "Cleaning up..."
	@find . -name "*.bk" -delete

# Help message
help:
	@echo "Available targets:"
	@echo "  bump-version    - Bump all crate versions to next minor version"
	@echo "  publish-crates  - Publish audio, tts, stt crates"
	@echo "  publish-main    - Publish voice-toolkit crate"
	@echo "  publish-all     - Bump versions and publish all crates"
	@echo "  clean           - Clean temporary files"
	@echo "  help            - Show this help message"
	@echo ""
	@echo "Usage examples:"
	@echo "  make bump-version    # Just bump versions"
	@echo "  make publish-all     # Bump versions and publish everything"
	@echo "  make publish-crates  # Only publish sub-crates"
	@echo "  make publish-main    # Only publish main crate"