APP := fmp
BINARY := target/release/$(APP)
DESKTOP := data/com.$(APP).desktop
ICON_SRC := data/com.$(APP).png
ICON_NAME := com.$(APP).png

# Default install prefix for system-wide: /usr/local
PREFIX ?= /usr/local

# For user install, set PREFIX=~/.local
BIN_DIR := $(PREFIX)/bin
APPLICATIONS_DIR := $(PREFIX)/share/applications
ICON_DIR := $(PREFIX)/share/icons/hicolor/48x48/apps
DESKTOP_DB_CMD := update-desktop-database
ICON_CACHE_CMD := gtk-update-icon-cache

.PHONY: all build install install-system install-user uninstall clean

all: build

build:
	cargo build --release

install: build
	@if [ "$(PREFIX)" = "/usr/local" ]; then \
	  echo "Installing system-wide to $(PREFIX)"; \
	else \
	  echo "Installing to $(PREFIX) (user)"; \
	fi
	install -d "$(DESTDIR)$(BIN_DIR)"
	install -m 755 "$(BINARY)" "$(DESTDIR)$(BIN_DIR)/$(APP)"
	install -d "$(DESTDIR)$(APPLICATIONS_DIR)"
	install -m 644 "$(DESKTOP)" "$(DESTDIR)$(APPLICATIONS_DIR)/com.$(APP).desktop"
	install -d "$(DESTDIR)$(ICON_DIR)"
	install -m 644 "$(ICON_SRC)" "$(DESTDIR)$(ICON_DIR)/$(ICON_NAME)"

	# Update caches for system installs (skip for user installs unless tools exist)
	@if [ "$(DESTDIR)" = "" ] && [ "$(PREFIX)" = "/usr/local" ]; then \
	  if command -v $(ICON_CACHE_CMD) >/dev/null 2>&1; then \
	    $(ICON_CACHE_CMD) -f "$(PREFIX)/share/icons/hicolor" >/dev/null 2>&1 || true; \
	  fi; \
	  if command -v $(DESKTOP_DB_CMD) >/dev/null 2>&1; then \
	    $(DESKTOP_DB_CMD) "$(PREFIX)/share/applications" >/dev/null 2>&1 || true; \
	  fi; \
	fi

# Convenience target for a per-user install
install-user:
	$(MAKE) install PREFIX=$(HOME)/.local

# Convenience target for system install (may require sudo)
install-system:
	sudo $(MAKE) install PREFIX=/usr/local

uninstall:
	-rm -f "$(DESTDIR)$(BIN_DIR)/$(APP)"
	-rm -f "$(DESTDIR)$(APPLICATIONS_DIR)/$(APP).desktop"
	-rm -f "$(DESTDIR)$(ICON_DIR)/$(ICON_NAME)"
	@if [ "$(DESTDIR)" = "" ] && [ "$(PREFIX)" = "/usr/local" ]; then \
	  if command -v $(DESKTOP_DB_CMD) >/dev/null 2>&1; then \
	    $(DESKTOP_DB_CMD) "$(PREFIX)/share/applications" >/dev/null 2>&1 || true; \
	  fi; \
	  if command -v $(ICON_CACHE_CMD) >/dev/null 2>&1; then \
	    $(ICON_CACHE_CMD) -f "$(PREFIX)/share/icons/hicolor" >/dev/null 2>&1 || true; \
	  fi; \
	fi

clean:
	cargo clean
