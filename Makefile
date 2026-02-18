PREFIX := $(HOME)/.local/bin
SERVICE_DIR := $(HOME)/.config/systemd/user

build:
	cargo build --release

install: build
	mkdir -p $(PREFIX)
	cp target/release/splash-damage $(PREFIX)/splash-damage
	sudo setcap cap_dac_override+ep $(PREFIX)/splash-damage

enable: install
	mkdir -p $(SERVICE_DIR)
	cp splash-damage.service $(SERVICE_DIR)/splash-damage.service
	systemctl --user daemon-reload
	systemctl --user enable splash-damage.service
	systemctl --user start splash-damage.service

update: build
	-systemctl --user stop splash-damage.service
	$(MAKE) install
	systemctl --user start splash-damage.service

disable:
	systemctl --user stop splash-damage.service
	systemctl --user disable splash-damage.service
